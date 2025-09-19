use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::Error,
    game::{
        deck::{BlackCard, DeckInfo, WhiteCard},
        lobby::{GamePhase, LobbyData},
    },
};

pub mod deck;
pub mod lobby;

/// Sent by the client to the server
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ClientEvent {
    /// Client wants to join a lobby
    JoinLobby { credentials: Credentials },
    /// Client updates game settings (only host allowed)
    UpdateSettings { settings: Settings },
    /// Add a deck (only host allowed)
    AddDeck { deckcode: String },
    /// Fetches all current decks from the api, use for a force update (only host allowed)
    FetchDecks,
    /// Client kicks a player (usually the host)
    Kick { kicked: Uuid },
    /// Client ends a running game (usually the host)
    EndGame,
    /// Client starts a new round (usually the host)
    StartRound,
    /// Client restarts the round (usually the host)
    RestartRound,
    /// A player submits a card during the submission phase
    SubmitOwnCards { indexes: Vec<usize> },
    /// Czar picks a winning card
    CzarPick { index: usize },
    /// Client leaves the current lobby
    LeaveLobby,
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
    /// Client lobby state
    ClientLobby(Box<ClientLobby>),
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
pub struct ClientLobby {
    players: HashMap<Uuid, PlayerInfo>,
    settings: Settings,
    phase: GamePhase,
    round: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    hand: Option<Vec<WhiteCard>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    revealed_cards: Vec<Vec<WhiteCard>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    submitted_players: Vec<Uuid>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    selected_cards: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    czar_pick: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    winner: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    black_card: Option<BlackCard>,
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
    secret: Uuid,
    info: PlayerInfo,
    cards: Vec<WhiteCard>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub name: String,
    pub id: Uuid,
    pub secret: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub max_rounds: Option<u32>,
    pub max_points: Option<u32>,
    pub max_submitting_time_secs: Option<Scaling>,
    pub max_judging_time_secs: Option<u64>,
    pub wait_time_secs: Option<u64>,
    pub max_players: u32,
    pub decks: Vec<DeckInfo>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_rounds: Some(10),
            max_points: Some(4),
            max_submitting_time_secs: Some(Scaling::default()),
            max_judging_time_secs: Some(30),
            wait_time_secs: Some(5),
            max_players: 20,
            decks: Vec::new(),
        }
    }
}

impl Settings {
    pub fn end_condition_reached(&self, lobby_data: &LobbyData) -> bool {
        let max_rounds_reached = match self.max_points {
            Some(limit) => lobby_data.round >= limit,
            None => false,
        };
        let max_points_reached = match self.max_points {
            Some(limit) => lobby_data.players.iter().any(|p| p.1.info.points >= limit),
            None => false,
        };

        max_rounds_reached || max_points_reached
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "seconds")]
pub enum Scaling {
    Player(u64),
    Constant(u64),
}

impl Default for Scaling {
    fn default() -> Self {
        Self::Constant(90)
    }
}

impl Scaling {
    pub fn to_seconds(&self, player_count: u64) -> u64 {
        match self {
            Scaling::Player(seconds_each) => seconds_each * player_count,
            Scaling::Constant(total) => *total,
        }
    }
}
