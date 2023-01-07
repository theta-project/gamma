use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use std::env;
mod server;

#[derive(Clone)]
struct Databases {
    redis: redis::Client,
    mysql: mysql::Pool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    println!("> theta! Gamma Server");
    println!("> CTRL+C to exit");

    HttpServer::new(|| {
        let redis_client = redis::Client::open(
            env::var_os("REDIS_URL")
                .expect("Please add the URL to your redis instance")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let mysql_client = mysql::Pool::new(
            env::var_os("MYSQL_URL")
                .expect("Please add the URL to your mysql instance")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let databases = Databases {
            redis: redis_client.clone(),
            mysql: mysql_client.clone(),
        };
        App::new()
            .app_data(web::Data::new(databases.clone()))
            .wrap(Logger::default())
            .service(server::index)
            .service(server::bancho_server)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
