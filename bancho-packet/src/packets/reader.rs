use crate::buffer::serialization::{Buffer, BytesExt, BytesMutExt};
use crate::packets::structures;
use bytes::{Buf, Bytes};
use tracing::instrument;

// TODO: Use `thiserror`  cos macros are fun
#[derive(Debug)]
pub enum ParseError {
    BadUTF8,
}

// TODO: This doesnt need to copy, could just slice into the buf
#[derive(Debug, Clone)]
pub struct LoginData {
    pub username: String,
    pub password_md5: String,
    pub client_version: String,
    pub utc_offset: i32,
    pub show_city: i32,
    pub allow_pms: i32,
    pub path_md5: String,
    pub adapters_string: String,
    pub adapters_md5: String,
    pub uninstall_md5: String,
    pub disk_signature_md5: String,
}

impl LoginData {
    #[instrument(name = "deserialise_login_data")]
    pub fn from_slice(buf: &mut Bytes) -> Result<Self, ParseError> {
        let username = String::from_utf8(buf.take_while(|b| b != b'\n').to_vec())
            .map_err(|_| ParseError::BadUTF8)?;
        buf.advance(1);

        let password_md5 = String::from_utf8(buf.take_while(|b| b != b'\n').to_vec())
            .map_err(|_| ParseError::BadUTF8)?;
        buf.advance(1); // '\n'

        let client_version = String::from_utf8(buf.take_while(|b| b != b'|').to_vec())
            .map_err(|_| ParseError::BadUTF8)?;
        buf.advance(1);

        let utc_offset = String::from_utf8(buf.take_while(|b| b != b'|').to_vec())
            .map_err(|_| ParseError::BadUTF8)?
            .parse::<i32>()
            .unwrap();
        buf.advance(1);

        let show_city = String::from_utf8(buf.take_while(|b| b != b'|').to_vec())
            .map_err(|_| ParseError::BadUTF8)?
            .parse::<i32>()
            .unwrap();
        buf.advance(1);

        let client_hashes = String::from_utf8(buf.take_while(|b| b != b'|').to_vec())
            .map_err(|_| ParseError::BadUTF8)?;
        buf.advance(1);

        let allow_pms = String::from_utf8(buf.take_while(|b| b != b'\n').to_vec())
            .map_err(|_| ParseError::BadUTF8)?
            .parse::<i32>()
            .unwrap();
        buf.advance(1);

        let mut hashes_split = client_hashes.split(':');

        let path_md5 = hashes_split.next().unwrap().to_string();

        let adapters_string = hashes_split.next().unwrap().to_string();

        let adapters_md5 = hashes_split.next().unwrap().to_string();

        let uninstall_md5 = hashes_split.next().unwrap().to_string();

        let disk_signature_md5 = hashes_split.next().unwrap().to_string();

        Ok(LoginData {
            username,
            password_md5,
            client_version,
            utc_offset,
            show_city,
            allow_pms,
            path_md5,
            adapters_string,
            adapters_md5,
            uninstall_md5,
            disk_signature_md5,
        })
    }
}

pub fn client_send_mesage(buf: &mut Buffer) -> structures::BanchoMessage {
    let sending_client = buf.get_string(); //buf.get_string();
    let message = buf.get_string();
    let target = buf.get_string();
    let sender_id = buf.get_i32_le();

    structures::BanchoMessage {
        sending_client,
        message,
        target,
        sender_id,
    }
}

pub fn client_user_status(buf: &mut Buffer) -> structures::ClientStatus {
    let status = buf.get_u8();
    let status_text = buf.get_string();
    let beatmap_checksum = buf.get_string();
    let current_mods = buf.get_u32_le();
    let play_mode = buf.get_u8();
    let beatmap_id = buf.get_i32_le();
    structures::ClientStatus {
        status,
        status_text,
        beatmap_checksum,
        current_mods,
        play_mode,
        beatmap_id,
    }
}
