use std::sync::Arc;

use crate::sessions;
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
use redis::AsyncCommands;
use sqlx::{Executor, Row};
use tracing::{debug, error, info_span, instrument, Instrument};
use uuid::Uuid;

use crate::{
    db::Databases,
    errors::{ExternalError, InternalError, RequestError, Result},
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
    let user_id: i32 = user_data.get(0 as usize);

    {
        let _span = info_span!("prepare_response", uuid = uuid.to_string()).entered();
        bancho_login_reply(&mut buffer, 69);
        bancho_channel_available(
            &mut buffer,
            BanchoChannel {
                name: "#osu".to_string(),
                topic: "default channel".to_string(),
                connected: 1,
            },
        );
        bancho_protocol_negotiaton(&mut buffer, 19);
        bancho_login_permissions(&mut buffer, 4);

        bancho_channel_join_success(&mut buffer, "#osu");
        bancho_announce(
            &mut buffer,
            format!("Welcome to Gamma, {}!", &login.username).as_str(),
        );

        bancho_channel_listing_complete(&mut buffer);

        let bot_presence = &*BOT_PRESENCE;
        let bot_stats = &*BOT_STATS;
        bancho_user_presence(&mut buffer, bot_presence.clone());
        bancho_handle_osu_update(&mut buffer, bot_stats.clone());

        res.append_header(("cho-token", uuid.to_string()));
    }
    // create a session struct to add to redis as a JSON object with the player id as the key such as "gamma::sessions::player_id"
    /*let sess = sessions::Session {
        token: uuid.to_string(),
        buffer: BytesMut::new(),
        presence: BanchoPresence { player_id: player_id, username: (), timezone: (), country_code: (), play_mode: (), permissions: (), longitude: (), latitude: (), player_rank: () }
        stats: {

        }
    }    */

    data.redis()
        .await?
        .set::<_, _, ()>(
            format!("gamma::sessions::{}", uuid),
            BytesMut::new().to_vec(),
        )
        .instrument(info_span!("add_session_buffer", uuid = uuid.to_string()))
        .await
        .map_err(InternalError::Redis)?;

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
    let buffer: Vec<u8> = data
        .redis()
        .await?
        .get(format!("gamma::buffers::{}", token))
        .instrument(info_span!("get_session_buffer", token = token))
        .await
        .map_err(|_| ExternalError::InvalidToken)?;

    // get the players buffer
    let mut player_buffer = BytesMut::from(buffer.as_slice());
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
                debug!(msg = "received user status", status = &status.status);
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
                let message = reader::client_send_mesage(&mut in_buf);
                debug!(
                    msg = "packet received",
                    typ = "send_message",
                    target = &message.target
                );
                dbg!(&message);
                if message.target == "GammaBot" {
                    bancho_announce(&mut player_buffer, "TODO: add commands.");
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

    // flush the buffer
    if let Err(e) = data
        .redis()
        .await?
        .set::<_, _, ()>(
            format!("gamma::buffers::{}", token),
            BytesMut::new().to_vec(),
        )
        .instrument(info_span!("flush_player_buffer", token = token))
        .await
    {
        // report the error, but still send back the packets
        let _ = RequestError::from(InternalError::Redis(e));
    };

    Ok(res.body(player_buffer))
}
