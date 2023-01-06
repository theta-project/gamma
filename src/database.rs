use actix_web::web::BytesMut;
use bancho_packets::buffer::serialization::Buffer;
use redis::Commands;

fn redis_error() {
    println!("Error in connecting to redis")
}
fn set_redis<K: redis::ToRedisArgs, V: redis::ToRedisArgs>(key: K, value: V) {
    let mut connection = redis::Client::open("redis://127.0.0.1/").unwrap().get_connection().unwrap();
    let _ : () = connection.set(key, value).unwrap();
}

fn get_redis<K: redis::ToRedisArgs>(key: K) {
    
}


pub fn add_buffer(token: String, buf: Buffer) {
    set_redis(format!("gamma::buffers::{}", token), buf.to_vec());
}

pub fn get_buffer(token: String) -> Buffer {
    let mut connection = redis::Client::open("redis://127.0.0.1/").unwrap().get_connection().unwrap();
    let x: Vec<u8>  = connection.get(format!("gamma::buffers::{}", token)).unwrap();
    BytesMut::from(x.as_slice())
}