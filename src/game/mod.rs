use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::Error,
    game::{
        deck::{BlackCard, DeckInfo, WhiteCard},
        lobby::GamePhase,
    },
};

pub mod deck;
pub mod lobby;

/// Sent by the client to the server
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ClientEvent {
    /// Client wants to join a lobby
    JoinLobby { name: String, id: Uuid },
    /// Client updates game settings (only host allowed)
    UpdateSettings { settings: Settings },
    /// Add a deck (only host allowed)
    AddDeck { deckcode: String },
    /// Clients kicks a player (usually the host)
    Kick { kicked: Uuid },
    /// Client starts a new round (usually the host)
    StartRound,
    /// Client restarts the round (usually the host)
    RestartRound,
    /// A player submits a card during the submission phase
    SubmitOwnCards { indexes: Vec<usize> },
    /// Czar picks a winning card
    CzarPick { index: usize },
}

/// Sent by the server to clients (broadcasted to all clients)
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ServerEvent {
    /// A player has joined the lobby
    PlayerJoin {
        player_id: Uuid,
        player_info: PlayerInfo,
    },
    /// A player was removed, either by the host or due to a timeout
    PlayerRemove { player_id: Uuid },
    /// The host left the lobby, assigns a new player to be one
    AssignHost { player_id: Uuid },
    /// The Round starts, a player has been selected as the czar with their card
    StartRound {
        czar_id: Uuid,
        black_card: BlackCard,
    },
    /// A card has been submitted by a player
    CardsSubmitted { player_id: Uuid },
    /// The Decks has been updated by the host, via a kick
    UpdateDecks { decks: Vec<DeckInfo> },
    /// The Settings has been updated by the host
    UpdateSettings { settings: Settings },
    /// Reveal all submitted cards to all players
    RevealCards { selected_cards: Vec<Vec<WhiteCard>> },
    /// The round was skipped
    RoundSkip,
    /// The round result after czar picks
    RoundResult {
        player_id: Uuid,
        winning_card_index: usize,
    },
    /// The game is over due to reaching max rounds
    GameOver,
    /// The lobby has been reset
    LobbyReset,
}

/// Sent by the server privately (to one client)
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum PrivateServerEvent {
    /// New state of the lobby
    LobbyState(LobbyState),
    /// Updates a player's hand (e.g., after submission or round start)
    UpdateHand { cards: Vec<WhiteCard> },
    /// A Player times out
    Timeout,
    /// A Player gets kicked by the host
    Kick,
    /// An error occurred during communication
    Error(Error),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LobbyState {
    players: HashMap<Uuid, PlayerInfo>,
    settings: Settings,
    phase: GamePhase,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerInfo {
    name: String,
    is_host: bool,
    is_czar: bool,
    points: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    info: PlayerInfo,
    cards: Vec<WhiteCard>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub max_rounds: u32,
    pub max_submitting_time_secs: Option<u64>,
    pub max_judging_time_secs: Option<u64>,
    pub wait_time_secs: Option<u64>,
    pub max_players: u32,
    pub decks: Vec<DeckInfo>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_rounds: 10,
            max_submitting_time_secs: Some(90),
            max_judging_time_secs: Some(30),
            wait_time_secs: Some(5),
            max_players: 20,
            decks: Vec::new(),
        }
    }
}
