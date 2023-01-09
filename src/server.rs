use std::sync::Arc;

use actix_web::{
    get, post,
    web::{Buf, Bytes, BytesMut, Data},
    Error, HttpRequest, HttpResponse, Responder,
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
use tracing::{debug, info_span, instrument, Instrument};
use uuid::Uuid;

use crate::db::Databases;
extern crate lazy_static;

lazy_static::lazy_static! {
    static ref BOT_PRESENCE: BanchoPresence = BanchoPresence {
        player_id: 3,
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
        player_id: 3,
        status: ClientStatus {
            status: 0,
            status_text: "Helping run Gamma".to_string(),
            beatmap_checksum: "".to_string(),
            current_mods: 0,
            play_mode: 0,
            beatmap_id: 0,
        },
        ranked_score: 0,
        accuracy: 0.,
        play_count: 0,
        total_score: 1337,
        rank: 0,
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
) -> Result<HttpResponse, Error> {
    match req.headers().get("osu-token") {
        Some(token) => handle_regular_packet(&req, token.to_str().unwrap(), body, &data).await,
        None => handle_auth_packet(&req, body, &data).await,
    }
}

#[instrument(skip(body, data))]
async fn handle_auth_packet(
    req: &HttpRequest,
    mut body: Bytes,
    data: &Databases,
) -> Result<HttpResponse, Error> {
    let login = LoginData::from_slice(&mut body)
        .map_err(|_| actix_web::error::PayloadError::EncodingCorrupted);

    let login_cloned = &login?.clone();
    debug!(
        "login request for `{}` from `{:?}`",
        &login_cloned.username,
        req.connection_info().peer_addr()
    );
    // TODO: Check against db, etc.
    let mut res = HttpResponse::Ok();
    let mut buffer = BytesMut::new();
    let uuid = Uuid::new_v4();

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
        format!("Welcome to Gamma, {}!", &login_cloned.clone().username).as_str(),
    );

    bancho_channel_listing_complete(&mut buffer);
    let bot_presence = &*BOT_PRESENCE;
    bancho_user_presence(&mut buffer, bot_presence.clone());

    res.append_header(("cho-token", uuid.to_string()));

    let _: () = data
        .redis()
        .set(
            format!("gamma::buffers::{}", uuid),
            BytesMut::new().to_vec(),
        )
        .instrument(info_span!("add_session_buffer", uuid = uuid.to_string()))
        .await
        .unwrap();

    Ok(res.body(buffer))
}

#[instrument(skip(body, data))]
async fn handle_regular_packet(
    _req: &HttpRequest,
    token: &str,
    body: Bytes,
    data: &Databases,
) -> Result<HttpResponse, Error> {
    let mut res = HttpResponse::Ok();
    let buffer: Vec<u8> = data
        .redis()
        .get(format!("gamma::buffers::{}", token))
        .instrument(info_span!("get_session_buffer", token = token))
        .await
        .unwrap();

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
                println!("{:?}", status);
            }
            4 => (), // update last pinged... maybe should have something to destroy it on no ping for n amount of time
            1 => {
                let message = reader::client_send_mesage(&mut in_buf);
                println!("{:?}", message);
            }
            63 => {
                let channel_name = in_buf.get_string();
                println!("{} has joined channel: {}", token, channel_name);
                bancho_channel_join_success(&mut player_buffer, channel_name.as_str());
            }
            _ => {
                println!("Unhandled packet: {} (length: {})", id, packet_length);
                in_buf.advance(packet_length as usize);
            }
        }

        length += packet_length as usize;
        length += 1;
    }

    // flush the buffer
    let _: () = data
        .redis()
        .set(
            format!("gamma::buffers::{}", token),
            BytesMut::new().to_vec(),
        )
        .instrument(info_span!("flush_player_buffer", token = token))
        .await
        .unwrap();
    Ok(res.body(player_buffer))
}
