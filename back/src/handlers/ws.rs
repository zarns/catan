// src/handlers/ws.rs
use actix::{Actor, StreamHandler, ActorContext};
use actix_web_actors::ws;
use log::{info, error};
use serde_json;
use uuid::Uuid;

use crate::game::{GameCommand, GameResponse, GameState, SharedGameManager};

pub struct GameWebSocket {
    pub player_id: Option<String>,
    pub game_id: Option<String>,
    pub game_manager: SharedGameManager,
}

impl Actor for GameWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket connection established");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket connection closed");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<GameCommand>(&text) {
                    Ok(cmd) => self.handle_command(cmd, ctx),
                    Err(e) => {
                        error!("Failed to parse command: {}", e);
                        let error = GameResponse::Error {
                            message: "Invalid command format".to_string(),
                        };
                        self.send_response(error, ctx);
                    }
                }
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            },
            _ => (),
        }
    }
}

impl GameWebSocket {
    pub fn new(game_manager: SharedGameManager) -> Self {
        Self {
            player_id: None,
            game_id: None,
            game_manager,
        }
    }

    fn send_response(&self, response: GameResponse, ctx: &mut ws::WebsocketContext<Self>) {
        if let Ok(response_json) = serde_json::to_string(&response) {
            ctx.text(response_json);
        }
    }

    fn process_ai_turns(&self, ctx: &mut ws::WebsocketContext<Self>) {
        if let Some(game_id) = &self.game_id {
            let mut game_manager = self.game_manager.lock().unwrap();
            if let Ok(ai_responses) = game_manager.process_ai_turns(game_id) {
                for response in ai_responses {
                    self.send_response(response, ctx);
                }
            }
        }
    }

    fn handle_command(&mut self, cmd: GameCommand, ctx: &mut ws::WebsocketContext<Self>) {
        let response = match cmd {
            GameCommand::JoinGame { player_name } => {
                let mut game_manager = self.game_manager.lock().unwrap();
                
                // Generate player ID if none exists
                if self.player_id.is_none() {
                    self.player_id = Some(Uuid::new_v4().to_string());
                }
                
                // Create a new game with one AI opponent
                match game_manager.create_game_with_ai(self.player_id.clone().unwrap(), 1) {
                    Ok(game_id) => {
                        self.game_id = Some(game_id.clone());
                        if let Some(state) = game_manager.get_game_state(&game_id) {
                            GameResponse::GameJoined {
                                player_id: self.player_id.clone().unwrap(),
                                game_state: state.clone(),
                            }
                        } else {
                            GameResponse::Error {
                                message: "Failed to get game state".to_string(),
                            }
                        }
                    }
                    Err(e) => GameResponse::Error { message: e },
                }
            },
            GameCommand::RollDice => {
                if let (Some(game_id), Some(player_id)) = (&self.game_id, &self.player_id) {
                    let mut game_manager = self.game_manager.lock().unwrap();
                    match game_manager.roll_dice(game_id, player_id) {
                        Ok(dice) => {
                            let response = GameResponse::DiceRolled {
                                player_id: player_id.clone(),
                                values: dice,
                            };
                            // Send the dice roll response
                            self.send_response(response.clone(), ctx);
                            
                            // Get updated game state
                            if let Some(state) = game_manager.get_game_state(game_id) {
                                GameResponse::GameStateUpdate {
                                    game_state: state.clone(),
                                }
                            } else {
                                GameResponse::Error {
                                    message: "Failed to get game state".to_string(),
                                }
                            }
                        },
                        Err(e) => GameResponse::Error { message: e },
                    }
                } else {
                    GameResponse::Error {
                        message: "Not in a game".to_string(),
                    }
                }
            },
            GameCommand::BuildSettlement { position } => {
                if let (Some(game_id), Some(player_id)) = (&self.game_id, &self.player_id) {
                    let mut game_manager = self.game_manager.lock().unwrap();
                    match game_manager.place_settlement(game_id, player_id, position.clone()) {
                        Ok(()) => {
                            if let Some(state) = game_manager.get_game_state(game_id) {
                                GameResponse::GameStateUpdate {
                                    game_state: state.clone(),
                                }
                            } else {
                                GameResponse::Error {
                                    message: "Failed to get game state".to_string(),
                                }
                            }
                        },
                        Err(e) => GameResponse::Error { message: e },
                    }
                } else {
                    GameResponse::Error {
                        message: "Not in a game".to_string(),
                    }
                }
            },
            GameCommand::BuildRoad { start, end } => {
                if let (Some(game_id), Some(player_id)) = (&self.game_id, &self.player_id) {
                    let mut game_manager = self.game_manager.lock().unwrap();
                    match game_manager.place_road(game_id, player_id, start, end) {
                        Ok(()) => {
                            if let Some(state) = game_manager.get_game_state(game_id) {
                                GameResponse::GameStateUpdate {
                                    game_state: state.clone(),
                                }
                            } else {
                                GameResponse::Error {
                                    message: "Failed to get game state".to_string(),
                                }
                            }
                        },
                        Err(e) => GameResponse::Error { message: e },
                    }
                } else {
                    GameResponse::Error {
                        message: "Not in a game".to_string(),
                    }
                }
            },
            _ => GameResponse::Error {
                message: "Command not implemented yet".to_string(),
            },
        };

        // Send the response to the human player
        self.send_response(response, ctx);

        // Process AI turns after the human player's move
        self.process_ai_turns(ctx);
    }
}