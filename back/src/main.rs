// src/main.rs
use actix_web::{web, App, HttpServer, Error, HttpResponse, HttpRequest};
use actix_web_actors::ws;
use actix_cors::Cors;
use std::env;

mod handlers;
mod models;
mod game;

use handlers::ws::GameWebSocket;
use game::manager::create_shared_game_manager;

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    game_manager: web::Data<game::manager::SharedGameManager>,
) -> Result<HttpResponse, Error> {
    ws::start(GameWebSocket::new(game_manager.get_ref().clone()), &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("127.0.0.1:{}", port);
    
    println!("Starting server at: {}", addr);

    let game_manager = create_shared_game_manager();
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(game_manager.clone()))
            .route("/ws", web::get().to(ws_route))
    })
    .bind(&addr)?
    .run()
    .await
}