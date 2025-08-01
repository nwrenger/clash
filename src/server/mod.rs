use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::{extract::State, Json};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::game::lobby::Lobby;

pub mod ws;

pub struct ServerState {
    pub lobbies: DashMap<Uuid, Arc<Lobby>>,
    pub cache: PathBuf,
}

impl ServerState {
    pub fn new(cache: PathBuf) -> Self {
        Self {
            lobbies: DashMap::new(),
            cache,
        }
    }

    /// Remove all lobbies in `LobbyOpen` or `GameOver` _only_ if they've
    /// been idle for over one hour. Returns how many were dropped.
    pub async fn clean_unused(&self) -> usize {
        // Scan for stale IDs
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for entry in self.lobbies.iter() {
            let id = *entry.key();
            let lobby = entry.value().clone();
            let last = *lobby.last_activity.read().await;
            if now.duration_since(last) > Duration::from_secs(60 * 60) {
                to_remove.push(id);
            }
        }

        // Actually remove them
        let before = self.lobbies.len();
        for id in to_remove {
            self.lobbies.remove(&id);
        }
        before - self.lobbies.len()
    }

    pub fn get_lobby(&self, uuid: Uuid) -> Result<Arc<Lobby>> {
        if let Some(lobby) = self.lobbies.get(&uuid) {
            Ok(lobby.clone())
        } else {
            Err(Error::LobbyNotFound)
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new(PathBuf::new())
    }
}

#[derive(Deserialize)]
pub struct Host {
    name: String,
    id: Uuid,
}

#[derive(Serialize)]
pub struct LobbyId {
    id: Uuid,
}

pub async fn create_lobby(
    State(state): State<Arc<ServerState>>,
    Json(host): Json<Host>,
) -> Result<Json<LobbyId>> {
    let lobby_id = Uuid::new_v4();
    let lobby = Lobby::new(state.cache.clone(), host.name, host.id).await?;
    state.lobbies.insert(lobby_id, lobby);

    Ok(Json(LobbyId { id: lobby_id }))
}
