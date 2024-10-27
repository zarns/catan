use actix_web::{web, Error, HttpResponse, HttpRequest, get};
use actix_web::web::ServiceConfig;
use actix_cors::Cors;
use actix_web_actors::ws;
use shuttle_actix_web::ShuttleActixWeb;
use log::info;

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

#[get("/")]
async fn hello() -> &'static str {
    "Catan Game Server"
}

#[shuttle_runtime::main]
async fn actix_web() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    info!("Starting Catan game server");
    
    let game_manager = create_shared_game_manager();
    
    let config = move |cfg: &mut ServiceConfig| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        cfg.service(
            web::scope("")
                .wrap(cors)
                .app_data(web::Data::new(game_manager.clone()))
                .service(hello)
                .route("/ws", web::get().to(ws_route))
        );
    };

    Ok(config.into())
}