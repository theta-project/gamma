# bancho-packets
Packet Serializer and Deserializer for osu!

example writing usage

```rs
use bancho_packets::*;
...
let mut buffer = BytesMut::new();
bancho_packet::bancho_channel_join_success(&mut buffer, "#osu");
...
```
example reading usage (assuming in_buf is a BytesMut with packets)
```rs
use bancho_packets::*;
let status = bancho_packet::reader::client_user_status(&mut in_buf);
```
