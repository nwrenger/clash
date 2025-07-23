use std::path::PathBuf;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::Instant;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::game::lobby::Lobby;

pub mod ws;

pub struct ServerState {
    pub lobbies: RwLock<HashMap<Uuid, Arc<Lobby>>>,
    pub cache: PathBuf,
}

impl ServerState {
    pub fn new(cache: PathBuf) -> Self {
        Self {
            lobbies: RwLock::new(HashMap::new()),
            cache,
        }
    }

    /// Remove all lobbies in `LobbyOpen` or `GameOver` _only_ if they've
    /// been idle for over one hour. Returns how many were dropped.
    pub async fn clean_unused(&self) -> usize {
        // Scan for stale IDs
        let now = Instant::now();
        let read_guard = self.lobbies.read().await;
        let mut to_remove = Vec::new();

        for (&id, lobby) in read_guard.iter() {
            let last = *lobby.last_activity.read().await;
            if now.duration_since(last) > Duration::from_secs(60 * 60) {
                to_remove.push(id);
            }
        }
        drop(read_guard);

        // Actually remove them
        let mut write_guard = self.lobbies.write().await;
        let before = write_guard.len();
        for id in to_remove {
            write_guard.remove(&id);
        }
        before - write_guard.len()
    }

    pub async fn get_lobby(&self, uuid: Uuid) -> Result<Arc<Lobby>> {
        let guard = self.lobbies.read().await;

        if let Some(lobby) = guard.get(&uuid) {
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
    let mut guard = state.lobbies.write().await;

    let lobby = Lobby::new(state.cache.clone(), host.name, host.id).await?;
    guard.insert(lobby_id, lobby);

    Ok(Json(LobbyId { id: lobby_id }))
}
