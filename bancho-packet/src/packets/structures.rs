#[derive(Debug)]
pub struct BanchoMessage {
    pub sending_client: String,
    pub message: String,
    pub target: String,
    pub sender_id: i32,
}

pub struct BanchoChannel {
    pub name: String,
    pub topic: String,
    pub connected: i16,
}
#[derive(Clone)]
pub struct BanchoPresence {
    pub player_id: i32,
    pub username: String,
    pub timezone: u8,
    pub country_code: u8,
    pub play_mode: u8,
    pub permissions: u8,
    pub longitude: f32,
    pub latitude: f32,
    pub player_rank: i32,
}
#[derive(Clone, Debug)]
pub struct ClientStatus {
    pub status: u8,
    pub status_text: String,
    pub beatmap_checksum: String,
    pub current_mods: u32,
    pub play_mode: u8,
    pub beatmap_id: i32,
}
#[derive(Clone, Debug)]
pub struct BanchoStats {
    pub player_id: i32,
    pub status: ClientStatus,
    pub ranked_score: i64,
    pub accuracy: f32,
    pub play_count: i32,
    pub total_score: i64,
    pub rank: i32,
    pub performance: i16,
}

pub struct MatchSlot {
    pub status: u8,
    pub team: u8,
    pub player_id: i32,
    pub slot_mods: u32,

    pub skipped: bool,
    pub completed: bool,
    pub loaded: bool,
}

pub struct ScoreFrame {
    pub time: i32,
    pub id: i32,
    pub count_300: i16,
    pub count_100: i16,
    pub count_50: i16,
    pub count_geki: i16,
    pub count_katu: i16,
    pub count_miss: i16,
    pub total_score: i32,
    pub max_combo: i16,
    pub current_combo: i16,
    pub perfect: bool,
    pub current_hp: f32,
    pub tag_byte: u8,
    pub using_score_v2: bool,
    pub combo_portion: Option<f32>,
    pub bonus_portion: Option<f32>,
}

pub struct Match {
    pub match_id: i32,
    pub in_progress: bool,
    pub match_type: u8,
    pub active_mods: u32,
    pub game_name: String,
    pub game_password: String,
    pub beatmap_name: String,
    pub beatmap_id: i32,
    pub beatmap_checksum: String,
    pub slot_status: Vec<u8>,
    pub slot_team: Vec<u8>,
    pub slot_id: Vec<i32>,
    pub host_id: i32,
    pub play_mode: u8,
    pub match_scoring_type: u8,
    pub match_team_type: u8,
    pub free_mod: bool,
    pub slot_mods: Vec<u32>,
    pub seed: i32,
}

pub struct ReplayFrame {
    pub button_state: u8,
    pub mouse_x: i16,
    pub mouse_y: i16,
    pub time: i32,
}

pub struct ReplayFrameBundle {
    pub extra: i32,
    pub frames: Vec<ReplayFrame>,
    pub action: i32,
    pub score_frame: ScoreFrame,
}