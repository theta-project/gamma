use actix_web::web::BytesMut;
use bancho_packet::{buffer::serialization::Buffer, packets::structures};
use sqlx::{mysql::MySqlRow, Row};

pub struct Session {
    pub token: String,
    pub buffer: Buffer,
    pub presence: structures::BanchoPresence,
    pub stats: structures::BanchoStats,
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
        token: uuid,
        buffer: BytesMut::new(),
        presence,
        stats,
    }
}
