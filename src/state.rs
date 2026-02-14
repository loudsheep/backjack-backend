use std::{collections::HashMap, sync::Mutex};

use tokio::sync::{broadcast, mpsc};

use crate::messages::{ClientMessage, ServerMessage};

#[derive(Clone)]
pub struct GameHandle {
    pub sender: mpsc::Sender<ClientMessage>,
    pub state_sender: broadcast::Sender<ServerMessage>,
}

pub struct AppState {
    pub games: Mutex<HashMap<String, GameHandle>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            games: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_game_handle(&self, game_id: &str) -> GameHandle {
        let mut games = self.games.lock().unwrap();

        if let Some(handle) = games.get(game_id) {
            return handle.clone();
        }

        let (lx, rx) = mpsc::channel(32);
        let (tx_state, _) = broadcast::channel(32);

        let mut actor = GameActor::new(rx, tx_state.clone());

        tokio::spawn(async move {
            actor.run().await;
            tracing::info!("Game actor for game_id {} has finished", game_id);
        });

        let handle = GameHandle {
            sender: lx,
            state_sender: tx_state,
        };

        games.insert(game_id.to_string(), handle.clone());
        handle
    }

    pub async fn get_game_sender(&self, game_id: &str) -> mpsc::Sender<ClientMessage> {
        self.get_game_handle(game_id).await.sender
    }

    pub async fn subscribe_to_game(&self, game_id: &str) -> broadcast::Receiver<ServerMessage> {
        self.get_game_handle(game_id).await.state_sender.subscribe()
    }
}
