use actix_web::{App, HttpServer, middleware::Logger};
mod server;
mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    println!("> theta! Gamma Server");
    println!("> CTRL+C to exit");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(server::index)
            .service(server::bancho_server)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
