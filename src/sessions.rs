use bancho_packet::{buffer::serialization::Buffer, packets::structures};
pub struct Session {
    pub token: String,
    pub buffer: Buffer,
    pub presence: structures::BanchoPresence,
    pub stats: structures::BanchoStats,
}