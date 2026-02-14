use crate::{
    game::{actor::GameActor, types::GameSettings},
    messages::{ClientMessage, ServerMessage},
};
use std::{collections::HashMap, sync::Mutex};
use rand::RngExt;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

#[derive(Clone)]
pub struct GameHandle {
    pub sender: mpsc::Sender<(Uuid, ClientMessage)>,
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

    pub async fn get_game_handle(&self, game_id: &str) -> Option<GameHandle> {
        let games = self.games.lock().unwrap();
        games.get(game_id).cloned()
    }

    pub fn create_game(&self, settings: GameSettings) -> String {
        let mut games = self.games.lock().unwrap();
        let mut rng = rand::rng();
        let id = loop {
            let id = format!("{:06}", rng.random_range(0..999999));
            if !games.contains_key(&id) {
                break id;
            }
        };

        let (tx, rx) = mpsc::channel(100);
        let (tx_state, _) = broadcast::channel(100);

        let mut actor = GameActor::new(id.clone(), settings, rx, tx_state.clone());

        let game_id_owned = id.clone();
        tokio::spawn(async move {
            actor.run().await;
            tracing::info!("Game {} ended", game_id_owned);
        });

        let handle = GameHandle {
            sender: tx,
            state_sender: tx_state,
        };

        games.insert(id.clone(), handle);
        id
    }


    pub async fn get_game_sender(&self, game_id: &str) -> Option<mpsc::Sender<(Uuid, ClientMessage)>> {
        self.get_game_handle(game_id).await.map(|h| h.sender)
    }

    pub async fn subscribe_to_game(&self, game_id: &str) -> Option<broadcast::Receiver<ServerMessage>> {
        self.get_game_handle(game_id).await.map(|h| h.state_sender.subscribe())
    }
}
