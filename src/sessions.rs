use actix_web::web::BytesMut;
use bancho_packet::{buffer::serialization::Buffer, packets::structures, packets::writer::*};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, Row};
use tracing_subscriber::registry::Data;

use crate::{db::Databases, errors::InternalError};

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: i32,
    pub token: String,
    pub presence: structures::BanchoPresence,
    pub stats: structures::BanchoStats,
    pub relax: bool,
    pub autopilot: bool,
}
const COUNTRY_CODES: [&str; 252] = [
    "oc", "eu", "ad", "ae", "af", "ag", "ai", "al", "am", "an", "ao", "aq", "ar", "as", "at", "au",
    "aw", "az", "ba", "bb", "bd", "be", "bf", "bg", "bh", "bi", "bj", "bm", "bn", "bo", "br", "bs",
    "bt", "bv", "bw", "by", "bz", "ca", "cc", "cd", "cf", "cg", "ch", "ci", "ck", "cl", "cm", "cn",
    "co", "cr", "cu", "cv", "cx", "cy", "cz", "de", "dj", "dk", "dm", "do", "dz", "ec", "ee", "eg",
    "eh", "er", "es", "et", "fi", "fj", "fk", "fm", "fo", "fr", "fx", "ga", "gb", "gd", "ge", "gf",
    "gh", "gi", "gl", "gm", "gn", "gp", "gq", "gr", "gs", "gt", "gu", "gw", "gy", "hk", "hm", "hn",
    "hr", "ht", "hu", "id", "ie", "il", "in", "io", "iq", "ir", "is", "it", "jm", "jo", "jp", "ke",
    "kg", "kh", "ki", "km", "kn", "kp", "kr", "kw", "ky", "kz", "la", "lb", "lc", "li", "lk", "lr",
    "ls", "lt", "lu", "lv", "ly", "ma", "mc", "md", "mg", "mh", "mk", "ml", "mm", "mn", "mo", "mp",
    "mq", "mr", "ms", "mt", "mu", "mv", "mw", "mx", "my", "mz", "na", "nc", "ne", "nf", "ng", "ni",
    "nl", "no", "np", "nr", "nu", "nz", "om", "pa", "pe", "pf", "pg", "ph", "pk", "pl", "pm", "pn",
    "pr", "ps", "pt", "pw", "py", "qa", "re", "ro", "ru", "rw", "sa", "sb", "sc", "sd", "se", "sg",
    "sh", "si", "sj", "sk", "sl", "sm", "sn", "so", "sr", "st", "sv", "sy", "sz", "tc", "td", "tf",
    "tg", "th", "tj", "tk", "tm", "tn", "to", "tl", "tr", "tt", "tv", "tw", "tz", "ua", "ug", "um",
    "us", "uy", "uz", "va", "vc", "ve", "vg", "vi", "vn", "vu", "wf", "ws", "ye", "yt", "rs", "za",
    "zm", "me", "zw", "xx", "a2", "o1", "ax", "gg", "im", "je", "bl", "mf",
];

/*
    Builds the session struct based on the information within the database
*/
pub fn build_session(user_data: MySqlRow, stats: MySqlRow, uuid: String) -> Session {
    let id = user_data.get(0_usize);
    let username = user_data.get(1_usize);
    let country: String = user_data.get(6_usize);
    let country_code = COUNTRY_CODES
        .iter()
        .position(|&r| r == country.as_str())
        .unwrap_or(0);

    let presence = structures::BanchoPresence {
        player_id: id,
        username,
        timezone: 0,
        country_code: (country_code as u8) + 1,
        play_mode: 0,
        permissions: 4,
        longitude: 0.,
        latitude: 0.,
        player_rank: 0,
    };

    let ranked_score: i64 = stats.try_get(2).unwrap_or(0);
    let total_score: i64 = stats.try_get(3).unwrap_or(0);
    let avg_accuracy: f32 = stats.try_get(6).unwrap_or(0.);
    let play_count = 0;
    let rank = 0;
    let performance: i16 = stats.try_get(8).unwrap_or(0);

    let status = structures::ClientStatus {
        status: 0,
        status_text: "".to_string(),
        beatmap_checksum: "".to_string(),
        current_mods: 0,
        play_mode: 0,
        beatmap_id: 0,
    };

    let stats = structures::BanchoStats {
        player_id: id,
        status,
        ranked_score,
        total_score,
        play_count,
        accuracy: avg_accuracy,
        rank,
        performance,
    };
    Session {
        id,
        token: uuid,
        presence,
        stats,
        relax: false,
        autopilot: false,
    }
}

pub async fn find_player_from_username(username: String, data: Databases) -> Session {
    let all_online = data
        .redis()
        .await
        .unwrap()
        .keys::<_, Vec<String>>("gamma::buffers::*")
        .await
        .map_err(InternalError::Redis);

    for player in all_online {
        let p = data
            .redis()
            .await
            .unwrap()
            .get::<_, String>(player)
            .await
            .map_err(InternalError::Redis);
        if (p.is_err()) {
            break;
        }
        let result = p.unwrap();

        let session: Session = serde_json::from_str(result.as_str()).unwrap();

        if session.presence.username == username {
            return session;
        }
    }
    Session {
        id: 0,
        token: "".to_string(),
        presence: structures::BanchoPresence {
            player_id: 0,
            username: "".to_string(),
            timezone: 0,
            country_code: 0,
            play_mode: 0,
            permissions: 0,
            longitude: 0.,
            latitude: 0.,
            player_rank: 0,
        },
        stats: structures::BanchoStats {
            player_id: 0,
            status: structures::ClientStatus {
                status: 0,
                status_text: "".to_string(),
                beatmap_checksum: "".to_string(),
                current_mods: 0,
                play_mode: 0,
                beatmap_id: 0,
            },
            ranked_score: 0,
            total_score: 0,
            play_count: 0,
            accuracy: 0.,
            rank: 0,
            performance: 0,
        },
        relax: false,
        autopilot: false,
    }
}

pub async fn all_online_status(buffer: &mut Buffer, redis: &mut deadpool_redis::Connection) {
    let all_online = redis
        .keys::<_, Vec<String>>("gamma::sessions::*")
        .await
        .map_err(InternalError::Redis);

    if all_online.is_ok() {
        for player in all_online.unwrap() {
            let p = redis
                .get::<_, String>(player)
                .await
                .map_err(InternalError::Redis);
            let session: Session = serde_json::from_str(&p.unwrap()).unwrap();

            bancho_user_presence(buffer, session.presence.clone());
            bancho_handle_osu_update(buffer, session.stats.clone());
        }
    }
}

pub async fn announce_online(session: Session, redis: &mut deadpool_redis::Connection) {
    let all_online = redis
        .keys::<_, Vec<String>>("gamma::sessions::*")
        .await
        .map_err(InternalError::Redis);

    if all_online.is_ok() {
        let all_players = all_online.unwrap();
        let mut b = Buffer::new();
        bancho_user_presence(&mut b, session.clone().presence);
        bancho_handle_osu_update(&mut b, session.clone().stats);

        for player in all_players {
            let token = player.replace("gamma::sessions::", "");

            let mut cmd = redis::cmd("RPUSH");
            cmd.arg(format!("gamma::buffers::{}", token));

            for byte in b.to_vec() {
                cmd.arg(byte as i32);
            }
            let _: () = cmd.query_async(redis).await.unwrap();
        }
    }
}

pub async fn update_stats(stats: structures::BanchoStats, redis: &mut deadpool_redis::Connection) {
    let all_online = redis
        .keys::<_, Vec<String>>("gamma::sessions::*")
        .await
        .map_err(InternalError::Redis);

    if all_online.is_ok() {
        let all_players = all_online.unwrap();
        let mut b = Buffer::new();
        bancho_handle_osu_update(&mut b, stats);

        for player in all_players {
            let token = player.replace("gamma::sessions::", "");

            let mut cmd = redis::cmd("RPUSH");
            cmd.arg(format!("gamma::buffers::{}", token));

            for byte in b.to_vec() {
                cmd.arg(byte as i32);
            }
            let _: () = cmd.query_async(redis).await.unwrap();
        }
    }
}

pub async fn send_pm(message: structures::BanchoMessage, redis: &mut deadpool_redis::Connection) {
    let all_online = redis
        .keys::<_, Vec<String>>("gamma::sessions::*")
        .await
        .map_err(InternalError::Redis);

    if all_online.is_ok() {
        let all_players = all_online.unwrap();
        for player in all_players {;
            let p = redis
                .get::<_, String>(&player)
                .await
                .map_err(InternalError::Redis);

            let session: Session = serde_json::from_str(&p.unwrap()).unwrap();
            if session.presence.username == message.target {
                let token = player.replace("gamma::sessions::", "");
                let mut b = Buffer::new();
                bancho_send_message(&mut b, message.clone());

                let mut cmd = redis::cmd("RPUSH");
                cmd.arg(format!("gamma::buffers::{}", token));

                for byte in b.to_vec() {
                    cmd.arg(byte as i32);
                }
                let _: () = cmd.query_async(redis).await.unwrap();
            }
        }
    }
}
