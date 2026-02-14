use crate::game::types::*;
use crate::messages::{ActionType, ClientMessage, ServerMessage};
use rand::seq::SliceRandom;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

pub struct GameActor {
    game_id: String,
    settings: GameSettings,
    phase: GamePhase,
    deck: Vec<Card>,
    players: Vec<Player>, // List of all players
    dealer_hand: Vec<Card>,
    turn_index: usize, // Who is playing right now?

    // Channels
    receiver: mpsc::Receiver<(Uuid, ClientMessage)>, // We need to know WHO sent the msg
    sender: broadcast::Sender<ServerMessage>,
}

impl GameActor {
    pub fn new(
        game_id: String,
        settings: GameSettings,
    ) -> (
        Self,
        mpsc::Sender<(Uuid, ClientMessage)>,
        broadcast::Receiver<ServerMessage>,
    ) {
        let (sender, receiver) = mpsc::channel(100);
        let (broadcast_sender, broadcast_receiver) = broadcast::channel(100);
        (
            Self {
                game_id,
                settings,
                phase: GamePhase::Lobby,
                deck: Vec::new(),
                players: Vec::new(),
                dealer_hand: Vec::new(),
                turn_index: 0,
                receiver,
                sender: broadcast_sender,
            },
            sender,
            broadcast_receiver,
        )
    }

    pub async fn run(&mut self) {
        while let Some((player_id, msg)) = self.receiver.recv().await {
            match msg {
                ClientMessage::CreateGame { settings, username } => {}
                ClientMessage::JoinGame { username, .. } => {
                    self.handle_join(player_id, username);
                }

                ClientMessage::StartGame => {
                    if self.is_admin(player_id) && self.phase == GamePhase::Lobby {
                        self.start_betting_phase();
                    }
                }

                ClientMessage::PlaceBet { amount } => {
                    self.handle_bet(player_id, amount);
                }

                ClientMessage::GameAction { action_type } => {
                    self.handle_action(player_id, action_type);
                }

                ClientMessage::ApprovePlayer { player_id } => {
                    if self.is_admin(player_id) {
                        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
                            p.status = PlayerStatus::Spectating;
                        }
                        self.broadcast_state();
                    }
                }
                _ => {}
            }
        }
    }

    fn is_admin(&self, player_id: Uuid) -> bool {
        self.players
            .iter()
            .find(|p| p.id == player_id)
            .map(|p| p.is_admin)
            .unwrap_or(false)
    }

    fn handle_join(&mut self, player_id: Uuid, username: String) {
        let is_first = self.players.is_empty();
        let status = if self.settings.approval_required && !is_first {
            PlayerStatus::PendingApproval
        } else {
            PlayerStatus::Spectating
        };

        self.players.push(Player {
            id: player_id,
            name: username,
            chips: self.settings.initial_chips,
            current_bet: 0,
            hand: vec![],
            status,
            is_admin: is_first,
        });

        self.broadcast_state();
    }

    fn handle_bet(&mut self, player_id: Uuid, amount: u32) {
        if self.phase != GamePhase::Betting {
            return;
        }

        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if player.chips >= amount {
                player.current_bet = amount;
                player.chips -= amount;
                player.status = PlayerStatus::Playing;
            }
        }

        if self
            .players
            .iter()
            .all(|p| p.current_bet > 0 || p.status == PlayerStatus::Spectating)
        {
            self.start_action_phase();
        }

        self.broadcast_state();
    }

    fn handle_action(&mut self, player_id: Uuid, action: ActionType) {
        if self.phase != GamePhase::Playing {
            return;
        }

        if let Some(current_player) = self.get_current_player() {
            if current_player.id != player_id {
                return;
            }
        } else {
            return;
        }

        match action {
            ActionType::Hit => {}
            ActionType::Stand => {}
            ActionType::Double => {}
            ActionType::Split => {}
        }

        self.broadcast_state();
    }

    fn advance_turn(&mut self) {
        self.turn_index += 1;
        if self.turn_index >= self.players.len() {
            self.play_dealer_turn();
        }
    }

    fn play_dealer_turn(&mut self) {
        self.phase = GamePhase::DealerTurn;
        // TODO: Implement dealer logic (hit until 17 or higher)

        self.broadcast_state();
    }

    fn get_current_player(&self) -> Option<&Player> {
        self.players.get(self.turn_index)
    }

    fn start_betting_phase(&mut self) {
        self.phase = GamePhase::Betting;
        for player in self.players.iter_mut() {
            if player.status != PlayerStatus::Spectating {
                player.status = PlayerStatus::Betting;
            }
        }
        self.broadcast_state();
    }

    fn start_action_phase(&mut self) {
        self.phase = GamePhase::Playing;
        self.turn_index = 0;
        self.broadcast_state();
    }

    fn broadcast_state(&self) {
        let mut sanitized_dealer = self.dealer_hand.clone();
        if self.phase == GamePhase::Playing && sanitized_dealer.len() >= 2 {
            sanitized_dealer.truncate(1);
        }

        let msg = ServerMessage::GameStateSnapshot {
            phase: self.phase.clone(),
            dealer_hand: sanitized_dealer,
            players: self.players.clone(),
            deck_remaining: self.deck.len(),
            current_turn_player_id: self.players.get(self.turn_index).map(|p| p.id),
        };

        let _ = self.sender.send(msg);
    }
}
