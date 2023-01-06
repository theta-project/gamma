use bytes::BufMut;

use crate::buffer::serialization::Buffer;
use crate::buffer::serialization::BytesMutExt;
use crate::packets::packet_ids::PacketIDs;
use crate::packets::structures;

pub fn bancho_login_reply(buf: &mut Buffer, player_id: i32) {
    buf.with_header(PacketIDs::BanchoLoginReply as i16, |buf| {
        buf.put_i32_le(player_id)
    });
}

pub fn bancho_send_message(buf: &mut Buffer, message: structures::BanchoMessage) {
    buf.with_header(PacketIDs::BanchoSendMessage as i16, |buf| {
        buf.put_string(&message.sending_client);
        buf.put_string(&message.message);
        buf.put_string(&message.target);
        buf.put_i32_le(message.sender_id);
    });
}
pub fn bancho_ping(buf: &mut Buffer) {
    buf.with_header(PacketIDs::BanchoPing as i16, |_| {})
}

pub fn bancho_handle_osu_update(buf: &mut Buffer, stats: structures::BanchoStats) {
    buf.with_header(PacketIDs::BanchoHandleOsuUpdate as i16, |buf| {
        buf.put_i32_le(stats.player_id);
        buf.put_u8(stats.status.status);
        buf.put_string(&stats.status.status_text);
        buf.put_string(&stats.status.beatmap_checksum);
        buf.put_u32_le(stats.status.current_mods);
        buf.put_u8(stats.status.play_mode);
        buf.put_i32_le(stats.status.beatmap_id);
    });
}

pub fn bancho_handle_user_quit(buf: &mut Buffer, player_id: i32) {
    buf.with_header(PacketIDs::BanchoHandleUserQuit as i16, |buf| {
        buf.put_i32_le(player_id);
        buf.put_bool(false);
    })
}

pub fn bancho_spectator_join(buf: &mut Buffer, player_id: i32) {
    buf.with_header(PacketIDs::BanchoSpectatorJoined as i16, |buf| {
        buf.put_i32_le(player_id);
    })
}

pub fn bancho_spectator_left(buf: &mut Buffer, player_id: i32) {
    buf.with_header(PacketIDs::BanchoSpectatorLeft as i16, |buf| {
        buf.put_i32_le(player_id);
    })
}

pub fn bancho_spectator_cant_spectate(buf: &mut Buffer, player_id: i32) {
    buf.with_header(PacketIDs::BanchoSpectatorCantSpectate as i16, |buf| {
        buf.put_i32_le(player_id);
    })
}

pub fn bancho_announce(buf: &mut Buffer, announcement: &str) {
    buf.with_header(PacketIDs::BanchoAnnounce as i16, |buf| {
        buf.put_string(announcement)
    })
}

pub fn bancho_channel_join_success(buf: &mut Buffer, channel_name: &str) {
    buf.with_header(PacketIDs::BanchoChannelJoinSuccess as i16, |buf| {
        buf.put_string(channel_name)
    })
}

pub fn bancho_channel_available(buf: &mut Buffer, channel: structures::BanchoChannel) {
    buf.with_header(PacketIDs::BanchoChannelAvailable as i16, |buf| {
        buf.put_string(&channel.name);
        buf.put_string(&channel.topic);
        buf.put_i16_le(channel.connected);
    })
}

pub fn bancho_channel_revoked(buf: &mut Buffer, channel_name: &str) {
    buf.with_header(PacketIDs::BanchoChannelRevoked as i16, |buf| {
        buf.put_string(channel_name)
    })
}

pub fn bancho_login_permissions(buf: &mut Buffer, permissions: u8) {
    buf.with_header(PacketIDs::BanchoLoginPermissions as i16, |buf| {
        buf.put_u8(permissions)
    })
}

pub fn bancho_protocol_negotiaton(buf: &mut Buffer, version: i32) {
    buf.with_header(PacketIDs::BanchoProtocolNegotiation as i16, |buf| {
        buf.put_i32_le(version);
    })
}

pub fn bancho_user_presence(buf: &mut Buffer, presence: structures::BanchoPresence) {
    buf.with_header(PacketIDs::BanchoUserPresence as i16, |buf| {
        buf.put_i32_le(presence.player_id);
        buf.put_string(&presence.username);
        buf.put_u8(presence.timezone);
        buf.put_u8(presence.country_code);
        buf.put_u8(presence.play_mode);
        buf.put_f32_le(presence.longitude);
        buf.put_f32_le(presence.latitude);
        buf.put_i32_le(presence.player_rank);
    })
}

pub fn bancho_channel_listing_complete(buf: &mut Buffer) {
    buf.with_header(PacketIDs::BanchoChannelListingComplete as i16, |buf| {
        buf.put_i32_le(0);
    })
}

pub fn bancho_user_pm_blocked(buf: &mut Buffer, message: structures::BanchoMessage) {
    buf.with_header(PacketIDs::BanchoUserPmBlocked as i16, |buf| {
        buf.put_string(&message.sending_client);
        buf.put_string(&message.message);
        buf.put_string(&message.target);
        buf.put_i32_le(message.sender_id);
    });
}

pub fn bacnho_target_is_silenced(buf: &mut Buffer, message: structures::BanchoMessage) {
    buf.with_header(PacketIDs::BanchoTargetIsSilenced as i16, |buf| {
        buf.put_string(&message.sending_client);
        buf.put_string(&message.message);
        buf.put_string(&message.target);
        buf.put_i32_le(message.sender_id);
    });
}

pub fn bancho_version_update_forced(buf: &mut Buffer) {
    buf.with_header(PacketIDs::BanchoVersionUpdateForced as i16, |buf| {})
}

pub fn bancho_switch_server(buf: &mut Buffer, server: &str) {
    buf.with_header(PacketIDs::BanchoSwitchServer as i16, |buf| {
        buf.put_string(server);
    })
}

pub fn bancho_account_restricted(buf: &mut Buffer) {
    buf.with_header(PacketIDs::BanchoAccountRestricted as i16, |buf| {})
}

pub fn bancho_rtx(buf: &mut Buffer, rtx: &str) {
    buf.with_header(PacketIDs::BanchoRTX as i16, |buf| {
        buf.put_string(rtx);
    })
}