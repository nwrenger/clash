use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::StreamExt;
use futures::{stream::SplitSink, SinkExt};
use serde::Serialize;
use std::sync::Arc;
use tokio::time::timeout;
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    game::{lobby::Lobby, ClientEvent, PrivateServerEvent},
    server::ServerState,
    GRACE_PERIOD, TIMEOUT_INTERVAL,
};

/// WebSocket handler for the `/ws/{uuid}` route
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Response> {
    if let Ok(lobby) = state.get_lobby(uuid) {
        Ok(ws.on_upgrade(|socket| handle_socket(socket, lobby)))
    } else {
        Err(Error::LobbyNotFound)
    }
}

async fn send_event<T>(sender: &mut SplitSink<WebSocket, Message>, event: &T)
where
    T: ?Sized + Serialize,
{
    if let Ok(txt) = serde_json::to_string(event) {
        sender.send(Message::Text(txt.into())).await.ok();
    }
}

/// Handles an individual WebSocket connection
async fn handle_socket(socket: WebSocket, lobby: Arc<Lobby>) {
    let (mut sender, mut receiver) = socket.split();

    // Expect a player join event otherwise close the connection
    // Also close after the `GRACE_PERIOD`
    let txt = match timeout(GRACE_PERIOD, receiver.next()).await {
        Ok(Some(Ok(Message::Text(t)))) => t,
        _ => {
            send_event(&mut sender, &PrivateServerEvent::Error(Error::LobbyLogin)).await;
            return;
        }
    };
    let (player_name, player_id) = match serde_json::from_str::<ClientEvent>(&txt) {
        Ok(ClientEvent::JoinLobby { name, id }) => (name, id),
        _ => {
            send_event(&mut sender, &PrivateServerEvent::Error(Error::LobbyLogin)).await;
            return;
        }
    };

    // Open global and private receivers and join the lobby
    let (mut global, mut private) = {
        // When join errors it's maken sure that no players got added beforehand
        if let Err(msg) = lobby.join(player_name, player_id).await {
            send_event(&mut sender, &PrivateServerEvent::Error(msg)).await;
            return;
        }
        (
            lobby.subscribe_global(),
            lobby.subscribe_private(player_id).await,
        )
    };

    // Send broadcasted events over to the websocket
    tokio::spawn(async move {
        loop {
            tokio::select! {
              Ok(event) = global.recv() => {
                send_event(&mut sender, &event).await;
              }

              Some(private_event) = private.recv() => {
                send_event(&mut sender, &private_event).await;
              }

              else => {
                  return;
              }
            }
        }
    });

    // Send lobby state on succesfull setup
    lobby.send_lobby_state(&player_id).await;

    // Receive events from websocket, timeout after the `TIMEOUT_INTERVAL`
    while let Some(Ok(Ok(msg))) = timeout(TIMEOUT_INTERVAL, receiver.next()).await.transpose() {
        if let Message::Text(txt) = msg {
            if let Ok(event) = serde_json::from_str::<ClientEvent>(&txt) {
                if let Err(error) = {
                    match event {
                        ClientEvent::JoinLobby { .. } => Ok(()),
                        ClientEvent::UpdateSettings { settings } => {
                            lobby.update_settings(&player_id, settings).await
                        }
                        ClientEvent::AddDeck { deckcode } => {
                            lobby.add_deck(&player_id, deckcode).await
                        }
                        ClientEvent::FetchDecks => lobby.fetch_decks(&player_id).await,
                        ClientEvent::Kick { kicked } => lobby.kick(&player_id, &kicked).await,
                        ClientEvent::StartRound => lobby.start_game(&player_id).await,
                        ClientEvent::RestartRound => lobby.reset_game(&player_id).await,
                        ClientEvent::SubmitOwnCards { indexes } => {
                            lobby.submit_cards(&player_id, indexes).await
                        }
                        ClientEvent::CzarPick { index } => {
                            lobby.submit_czar_choice(&player_id, index).await
                        }
                        ClientEvent::LeaveLobby => lobby.leave(&player_id).await,
                    }
                } {
                    // hope this does'nt throw an error otherwise :skull:
                    lobby
                        .emit_private(&player_id, PrivateServerEvent::Error(error))
                        .await;
                }
            }
        }
    }

    // Mark the player as disconnected
    lobby.player_disconnected(player_id).await;
}
