use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{State, Path},
    response::IntoResponse,
};
use tokio::sync::mpsc;
use crate::state::AppState;
use crate::messages::ClientMessage;
use std::sync::Arc;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // 1. Get or Create the Game Channel
    let tx = state.get_game_sender(&game_id).await;
    let mut rx = state.subscribe_to_game(&game_id).await;

    // 2. Spawn a task to forward Game Updates -> WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // 3. Main Loop: Forward WebSocket Messages -> Game Actor
    while let Some(Ok(msg)) = receiver.recv().await {
        if let Message::Text(text) = msg {
            if let Ok(cmd) = serde_json::from_str::<ClientMessage>(&text) {
                let _ = tx.send(cmd).await;
            }
        }
    }

    send_task.abort();
}
