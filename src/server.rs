use std::sync::Arc;

use actix_web::{
    get, post,
    web::{Buf, Bytes, BytesMut, Data},
    HttpRequest, HttpResponse, Responder,
};
use bancho_packet::{
    buffer::serialization::BytesMutExt,
    packets::{
        reader::{self, *},
        structures::*,
        writer::*,
    },
};

use bcrypt::verify;
use redis::AsyncCommands;
use sqlx::{Executor, Row};
use tracing::{debug, error, info_span, instrument, Instrument};
use uuid::Uuid;

use crate::{
    db::Databases,
    errors::{ExternalError, InternalError, RequestError, Result},
    sessions,
};
extern crate lazy_static;

lazy_static::lazy_static! {
    static ref BOT_PRESENCE: BanchoPresence = BanchoPresence {
        player_id: 5,
        username: "GammaBot".to_string(),
        timezone: 24,
        country_code: 0,
        permissions: 8,
        play_mode: 0,
        longitude: 0.,
        latitude: 0.,
        player_rank: 0
    };

    static ref BOT_STATS: BanchoStats = BanchoStats {
        player_id: 5,
        status: ClientStatus {
            status: 0,
            status_text: "- Helping run Gamma".to_string(),
            beatmap_checksum: "".to_string(),
            current_mods: 0,
            play_mode: 2,
            beatmap_id: 0,
        },
        ranked_score: 0,
        accuracy: 0.,
        play_count: 0,
        total_score: 0,
        rank: 1,
        performance: 0,
    };
}

#[get("/")]
pub async fn index() -> impl Responder {
    "theta! Gamma Server\n"
}

#[post("/")]
pub async fn bancho_server(
    req: HttpRequest,
    body: Bytes,
    data: Data<Arc<Databases>>,
) -> Result<HttpResponse> {
    match req.headers().get("osu-token") {
        Some(token) => {
            let token = token.to_str().map_err(|_| ExternalError::InvalidToken)?;
            handle_regular_req(&req, token, body, &data).await
        }
        None => handle_auth_req(&req, body, &data).await,
    }
}

#[instrument(skip_all)]
async fn handle_auth_req(
    req: &HttpRequest,
    mut body: Bytes,
    data: &Databases,
) -> Result<HttpResponse> {
    let login = LoginData::from_slice(&mut body).map_err(ExternalError::MalformedPacket)?;
    let mut mysql_pool = data.mysql().await.unwrap();

    debug!(
        "login request for `{}` from `{:?}`",
        &login.username,
        req.connection_info().peer_addr()
    );
    // TODO: Check against db, etc.
    let mut res = HttpResponse::Ok();
    let mut buffer = BytesMut::new();
    let uuid = Uuid::new_v4();

    let username_safe = login.username.replace(' ', "_").to_lowercase();
    let player_query = sqlx::query("SELECT * FROM `users` WHERE username_safe = ?")
        .bind(username_safe)
        .fetch_one(&mut mysql_pool)
        .await;

    if player_query.is_err() {
        bancho_login_reply(&mut buffer, -1);
        res.append_header(("cho-token", "invalid username"));
        return Ok(res.body(buffer));
    }
    let user_data = player_query.unwrap();
    let password: String = user_data.get(5_usize);
    if !verify(login.password_md5, &password).unwrap() {
        bancho_login_reply(&mut buffer, -1);
        res.append_header(("cho-token", "invalid password"));
        return Ok(res.body(buffer));
    }

    let user_id: i32 = user_data.get(0_usize);
    let stats = sqlx::query("SELECT * FROM `user_stats` WHERE user_id = ? AND mode = 0")
        .bind(user_id)
        .fetch_one(&mut mysql_pool)
        .await;

    if stats.is_err() {
        debug!("could not find player in stats table");
        bancho_login_reply(&mut buffer, -5);
        res.append_header(("cho-token", "invalid stats"));
        return Ok(res.body(buffer));
    }
    {
        let _span = info_span!("prepare_response", uuid = uuid.to_string()).entered();
        // Write all of the necessary login packets, similar to that of the official osu! server
        let user_stats = stats.unwrap();
        let session = sessions::build_session(user_data, user_stats, uuid.to_string());

        bancho_login_reply(&mut buffer, 69);
        bancho_protocol_negotiaton(&mut buffer, 19);
        bancho_announce(
            &mut buffer,
            format!("Welcome to Gamma, {}!", &login.username).as_str(),
        );
        bancho_login_permissions(&mut buffer, 4);
        bancho_channel_listing_complete(&mut buffer);
        let channels_query = sqlx::query("SELECT * FROM `channels`")
            .fetch_all(&mut mysql_pool)
            .await;
        if channels_query.is_err() {
            debug!("could not find channels in db");
            bancho_login_reply(&mut buffer, -5);
            res.append_header(("cho-token", "invalid channels"));
            return Ok(res.body(buffer));
        }
        let channels = channels_query.unwrap();
        for channel in channels {
            let name: String = channel.get(1_usize);
            let topic: String = channel.get(2_usize);
            let autojoin: i8 = channel.get(4_usize);
            println!("{} {}", name, topic);
            bancho_channel_available(
                &mut buffer,
                BanchoChannel {
                    name: name.clone(),
                    topic: topic.clone(),
                    connected: 0,
                },
            );
            if autojoin == 1 {
                bancho_channel_join_success(&mut buffer, &name);
            }
        }
        bancho_ban_info(&mut buffer, 0);

        bancho_user_presence(&mut buffer, session.presence.clone());
        bancho_handle_osu_update(&mut buffer, session.stats.clone());

        bancho_channel_join_success(&mut buffer, "#osu");

        let bot_presence = &*BOT_PRESENCE;
        let bot_stats = &*BOT_STATS;
        bancho_user_presence(&mut buffer, bot_presence.clone());
        bancho_handle_osu_update(&mut buffer, bot_stats.clone());
        sessions::all_online_status(&mut buffer, data.clone()).await;
        sessions::announce_online(session.clone(), data.clone()).await;

        res.append_header(("cho-token", uuid.to_string()));
        let session_string = serde_json::to_string(&session).unwrap();
        data.redis()
            .await?
            .set::<_, _, ()>(format!("gamma::sessions::{}", uuid), session_string)
            .instrument(info_span!("add_session", uuid = uuid.to_string()))
            .await
            .map_err(InternalError::Redis)?;
        data.redis()
            .await?
            .set::<_, _, ()>(format!("gamma::buffers::{}", uuid), BytesMut::new().to_vec())
            .instrument(info_span!("add_session", uuid = uuid.to_string()))
            .await
            .map_err(InternalError::Redis)?;
    }

    Ok(res.body(buffer))
}

#[instrument(skip_all)]
async fn handle_regular_req(
    _req: &HttpRequest,
    token: &str,
    body: Bytes,
    data: &Databases,
) -> Result<HttpResponse> {
    let mut res = HttpResponse::Ok();
    // get session object from redis
    let session_redis: String = data
        .redis()
        .await?
        .get(format!("gamma::sessions::{}", token))
        .instrument(info_span!("get_session", token = token))
        .await
        .map_err(|_| ExternalError::InvalidToken)?;
    let buffer_redis: Vec<u8> = data
        .redis()
        .await?
        .get(format!("gamma::buffers::{}", token))
        .instrument(info_span!("get_buffer", token = token))
        .await
        .map_err(|_| ExternalError::InvalidToken)?;

    let mut session: sessions::Session = serde_json::from_str(&session_redis).unwrap();
    // get the players buffer
    let mut player_buffer = BytesMut::from(buffer_redis.as_slice());
    let binding = body.to_vec();
    let body_vec = binding.as_slice();

    let mut in_buf = BytesMut::from(body_vec);

    let mut length = 0;
    while length <= in_buf.len() {
        let id = in_buf.get_i16_le();
        let _compression = in_buf.get_u8() == 1;
        let packet_length = in_buf.get_u32_le();

        match id {
            0 => {
                let status = reader::client_user_status(&mut in_buf);
                session.stats.status = status;
                bancho_handle_osu_update(&mut player_buffer, session.stats.clone());
                sessions::update_stats(session.stats.clone(), data.clone()).await;
                // TODO: maybe check if a player is already in relax mode and if they are, don't announce it
                if (session.stats.status.current_mods & 128 == 128) && (!session.relax) {
                    bancho_announce(
                        &mut player_buffer,
                        format!(
                            "Relax leaderboards have now been enabled, {}",
                            session.presence.username
                        )
                        .as_str(),
                    );
                    session.relax = true;
                } else if (session.stats.status.current_mods & 8192 == 8192) && (!session.autopilot)
                {
                    bancho_announce(
                        &mut player_buffer,
                        format!(
                            "Autopilot leaderboards have now been enabled, {}",
                            session.presence.username
                        )
                        .as_str(),
                    );
                    session.autopilot = true;
                } else if (session.relax) && (session.stats.status.current_mods & 128 != 128) {
                    bancho_announce(
                        &mut player_buffer,
                        format!(
                            "Relax leaderboards have now been disabled, {}",
                            session.presence.username
                        )
                        .as_str(),
                    );
                    session.relax = false;
                } else if (session.autopilot) && (session.stats.status.current_mods & 8192 != 8192)
                {
                    bancho_announce(
                        &mut player_buffer,
                        format!(
                            "Autopilot leaderboards have now been disabled, {}",
                            session.presence.username
                        )
                        .as_str(),
                    );
                    session.autopilot = false;
                }
            }
            1 => {
                let message = reader::client_send_mesage(&mut in_buf);
                debug!(
                    msg = "packet received",
                    typ = "send_message",
                    target = &message.target
                );
            }
            4 => (), // update last pinged... maybe should have something to destroy it on no ping for n amount of time
            25 => {
                let mut message = reader::client_send_mesage(&mut in_buf);
                debug!(
                    msg = "packet received",
                    typ = "send_message",
                    target = &message.target
                );
                if message.target == "GammaBot" {
                } else {
                    let mut message_cloned = message.clone();

                    message_cloned.sender_id = session.presence.player_id;
                    message_cloned.sending_client = session.presence.username.clone();

                    sessions::send_pm(message_cloned, data.clone()).await;
                }
            }
            63 => {
                let channel_name = in_buf.get_string();
                debug!(
                    msg = "packet received",
                    typ = "join_channel",
                    channel_name = &channel_name
                );
                bancho_channel_join_success(&mut player_buffer, channel_name.as_str());
            }
            id => {
                error!(
                    msg = "unrecognised packet received",
                    id = id,
                    length = packet_length
                );
                in_buf.advance(packet_length as usize);
            }
        }

        length += packet_length as usize;
        length += 1;
    }
    let session_string = serde_json::to_string(&session).unwrap();
    // flush the buffer
    if let Err(e) = data
        .redis()
        .await?
        .set::<_, _, ()>(format!("gamma::sessions::{}", token), session_string)
        .instrument(info_span!("update_session", token = token))
        .await
    {
        // report the error, but still send back the packets
        let _ = RequestError::from(InternalError::Redis(e));
    };
    flush(buffer_redis, token, data.clone()).await?;
    Ok(res.body(player_buffer))
}


async fn flush(original_buf: Vec<u8>, token: &str, data: Databases)  -> Result<bool> {
    let mut buffer_redis: Vec<u8> = data
        .redis()
        .await?
        .get(format!("gamma::buffers::{}", token))
        .instrument(info_span!("get_buffer", token = token))
        .await
        .map_err(|_| ExternalError::InvalidToken)?;

    let remove_to = original_buf.len();
    buffer_redis.drain(0..remove_to);

    data.redis()
            .await?
            .set::<_, _, ()>(format!("gamma::buffers::{}", token), BytesMut::new().to_vec())
            .instrument(info_span!("update_buffer", uuid = token))
            .await
            .map_err(InternalError::Redis)?;
    Ok(true)
}