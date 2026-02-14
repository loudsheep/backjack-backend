use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GamePhase {
    Lobby,
    Betting,
    Playing,
    DealerTurn,
    Payout,
    GameOver,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub chips: u32,
    pub current_bet: u32,
    pub hand: Vec<Card>,
    pub status: PlayerStatus,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PlayerStatus {
    Spectating, // Just joined or sitting out
    Betting,    // Needs to place bet
    Playing,    // Waiting for action
    Stood,      // Finished turn
    Busted,     // > 21
    PendingApproval, // Waiting for admin to let them in
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GameSettings {
    pub initial_chips: u32,
    pub max_players: usize,
    pub deck_count: usize,
    pub approval_required: bool,
    pub chat_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn value(&self) -> u8 {
        match self.rank {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            Rank::Ace => 11, // ace can be 1 or 11, but we'll handle that in the game logic
        }
    }
}