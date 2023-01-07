# bancho-packets
Packet Serializer and Deserializer for osu!

example writing usage

```
...
let mut buffer = BytesMut::new();
bancho_packets::bancho_channel_join_success(&mut buffer, "#osu");
...
```
example reading usage (assuming in_buf is a BytesMut with packets)
```
let status = bancho_packets::reader::client_user_status(&mut in_buf);
```
