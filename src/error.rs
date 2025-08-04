use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "value")]
pub enum Error {
    /// During the login sequenze malformed data was send
    LobbyLogin,
    /// The lobby is closed (a game is going on), joining is not possible
    LobbyClosed,
    /// The lobby is full (from maxplayers), joining is not possible
    LobbyFull,
    /// The lobby cannot be started, this happens because certain criteria are hit which will break the game
    LobbyStart,
    /// The lobby is full (from maxplayers), joining is not possible
    LobbyNotFound,
    /// Card couldn't be submitted due to Game Phase missmatch
    CardSubmission,
    /// Czar couldn't choose due to Game Phase missmatch
    CzarChoice,
    /// Event send from player or source which is not authorized to do that action
    Unauthorized,
    /// Deck related errors
    Deck(String),
    /// Reqwest related Errors
    Reqwest(String),
    /// File System Error
    FileSystem(String),
    /// Json Serialzing/Desializing Error
    Json(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::FileSystem(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Error::LobbyClosed
            | Error::LobbyLogin
            | Error::LobbyFull
            | Error::Json(_)
            | Error::Deck(_) => StatusCode::BAD_REQUEST,
            Error::LobbyNotFound => StatusCode::NOT_FOUND,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::CardSubmission
            | Error::LobbyStart
            | Error::CzarChoice
            | Error::FileSystem(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Reqwest(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        (status, Json(self)).into_response()
    }
}
