use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::StreamExt;
use futures::{stream::SplitSink, SinkExt};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    game::{lobby::Lobby, ClientEvent, PrivateServerEvent, ServerEvent},
    server::ServerState,
};

/// WebSocket handler for the `/ws/{uuid}` route
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Response> {
    if let Ok(lobby) = state.get_lobby(uuid).await {
        Ok(ws.on_upgrade(|socket| handle_socket(socket, lobby)))
    } else {
        Err(Error::LobbyNotFound)
    }
}

async fn send_private_event(
    sender: &mut SplitSink<WebSocket, Message>,
    event: &PrivateServerEvent,
) {
    let txt = serde_json::to_string(event).unwrap();
    sender.send(Message::Text(txt.into())).await.ok();
}

async fn send_event(sender: &mut SplitSink<WebSocket, Message>, event: &ServerEvent) {
    let txt = serde_json::to_string(event).unwrap();
    sender.send(Message::Text(txt.into())).await.ok();
}

/// Handles an individual WebSocket connection
async fn handle_socket(socket: WebSocket, lobby: Arc<Lobby>) {
    let (mut sender, mut receiver) = socket.split();

    // Expect a player join event
    // otherwise close the connection
    let txt = match receiver.next().await {
        Some(Ok(Message::Text(t))) => t,
        _ => {
            send_private_event(&mut sender, &PrivateServerEvent::Error(Error::LobbyLogin)).await;
            return;
        }
    };
    let (player_name, player_id) = match serde_json::from_str::<ClientEvent>(&txt) {
        Ok(ClientEvent::JoinLobby { name, id }) => (name, id),
        _ => {
            send_private_event(&mut sender, &PrivateServerEvent::Error(Error::LobbyLogin)).await;
            return;
        }
    };

    // open global and private receivers and join the lobby
    let (mut global, mut private) = {
        if let Err(msg) = lobby.join(player_name, player_id).await {
            send_private_event(&mut sender, &PrivateServerEvent::Error(msg)).await;
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
                send_private_event(&mut sender, &private_event).await;
              }
              else => {
                  send_private_event(&mut sender, &PrivateServerEvent::Error(Error::WebSocket(String::from("The Broadcasting from lobby to WebSocket didn't work as expected. Closing connection!")))).await;
                  return;
              }
            }
        }
    });

    if let Err(error) = lobby.send_lobby_state().await {
        lobby
            .emit_private(&player_id, PrivateServerEvent::Error(error))
            .await
            .unwrap();
        return;
    }

    // Receive events from websocket
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(txt) = msg {
            if let Ok(event) = serde_json::from_str::<ClientEvent>(&txt) {
                // TODO: Error Handling via sending Error Event
                if let Err(error) = {
                    match event {
                        ClientEvent::JoinLobby { .. } => Ok(()),
                        ClientEvent::UpdateSettings { settings } => {
                            lobby.update_settings(&player_id, settings).await
                        }
                        ClientEvent::AddDeck { deckcode } => {
                            lobby.add_deck(&player_id, deckcode).await
                        }
                        ClientEvent::StartRound => lobby.start_game(&player_id).await,
                        ClientEvent::Kick { kicked } => lobby.kick(&player_id, &kicked).await,
                        ClientEvent::CzarPick { index } => {
                            lobby.submit_czar_choice(&player_id, index).await
                        }
                        ClientEvent::RestartRound => lobby.reset_game(&player_id).await,
                        ClientEvent::SubmitOwnCards { indexes } => {
                            lobby.submit_cards(&player_id, indexes).await
                        }
                    }
                } {
                    // hope this does'nt throw an error otherwise :skull: and we should panic
                    lobby
                        .emit_private(&player_id, PrivateServerEvent::Error(error))
                        .await
                        .unwrap();
                }
            }
        }
    }
}
