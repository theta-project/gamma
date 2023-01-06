use crate::database;
use actix_web::{
    get, post,
    web::{Buf, Bytes, BytesMut, Payload},
    Error, HttpRequest, HttpResponse, HttpResponseBuilder, Responder,
};
use bancho_packets::{
    packets::{reader::*, structures::*, writer::*}, buffer::serialization::BytesMutExt,
};
use log::debug;
use uuid::Uuid;

#[get("/")]
pub async fn index() -> impl Responder {
    "theta! Gamma Server\n"
}

#[post("/")]
pub async fn bancho_server(
    req: HttpRequest,
    body: Bytes,
) -> Result<HttpResponse, Error> {
    match req.headers().get("osu-token") {
        Some(token) => handle_packets(req.clone(), token.to_str().unwrap(), body).await,
        None => do_auth(req, body),
    }
}

fn do_auth(req: HttpRequest, mut body: Bytes) -> Result<HttpResponse, Error> {
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
        format!("Welcome to gamma, {}!", &login_cloned.clone().username).as_str(),
    );

    bancho_channel_listing_complete(&mut buffer);

    res.append_header(("cho-token", uuid.to_string()));
    database::add_buffer(uuid.to_string(), BytesMut::new());

    Ok(res.body(buffer))
}

async fn handle_packets(
    req: HttpRequest,
    token: &str,
    mut body: Bytes,
) -> Result<HttpResponse, Error> {
    let mut res = HttpResponse::Ok();
    // get the players buffer
    let mut player_buffer = database::get_buffer(token.to_string());
    let binding = body.to_vec();
    let body_vec = binding.as_slice();

    let mut in_buf = BytesMut::from(body_vec);

    let mut length = 0;
    while length <= in_buf.len() {
        let id = in_buf.get_i16_le();
        let _compression = in_buf.get_u8() == 1;
        let packet_length = in_buf.get_u32_le();

        match id {
            4 => (), // update last pinged... maybe should have something to destroy it on no ping for n amount of time
            63 => {
                let channel_name = in_buf.get_string();
                println!("{} has joined channel: {}", token, channel_name);
                bancho_channel_join_success(&mut player_buffer, channel_name.as_str());
            },
            _ => {
                println!("Unhandled packet: {} (length: {})", id, packet_length);
                in_buf.advance(packet_length as usize);
            },
        }

        
        length += packet_length as usize;
        length += 1;
    }

    // flush the buffer
    database::add_buffer(token.to_string(), BytesMut::new());
    Ok(res.body(player_buffer))
}
