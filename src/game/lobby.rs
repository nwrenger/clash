use dashmap::DashMap;
use rand::{rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{
        broadcast::{Receiver, Sender},
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Notify, RwLock,
    },
    time::Instant,
};
use tokio::{task::JoinHandle, time::sleep};
use tracing::error;
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    game::{
        deck::{BlackCard, Deck, DeckInfo, WhiteCard},
        LobbyState, Player, PlayerInfo, PrivateServerEvent, ServerEvent, Settings,
    },
    GRACE_PERIOD,
};

/// Inner mutable state of a lobby, protected by RwLock.
#[derive(Debug, Default)]
pub struct LobbyData {
    pub settings: Settings,
    pub players: HashMap<Uuid, Player>,
    pub czar_order: VecDeque<Uuid>,
    pub round: u32,
    pub black_card: Option<BlackCard>,
    pub submissions: Vec<(Uuid, Vec<WhiteCard>)>,
    pub czar_pick: Option<usize>,
    pub phase: GamePhase,
}

/// The overall lobby/game, separating channels from state.
pub struct Lobby {
    pub game_task: RwLock<Option<JoinHandle<()>>>,
    pub disconnect_timers: DashMap<Uuid, JoinHandle<()>>,
    pub global: Sender<ServerEvent>, // broadcast to all clients
    pub private: DashMap<Uuid, UnboundedSender<PrivateServerEvent>>,
    pub cache: PathBuf,
    pub state: RwLock<LobbyData>, // game state
    pub last_activity: RwLock<Instant>,
    pub submission_notify: Notify,
    pub czar_notify: Notify,
}

/// Discrete phases in a round
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GamePhase {
    #[default]
    LobbyOpen,
    Submitting,
    Judging,
    RoundFinished,
    GameOver,
}

impl Drop for Lobby {
    fn drop(&mut self) {
        tracing::info!("Lobby dropped");
    }
}

impl Lobby {
    /// Create a new lobby with host as first player.
    pub async fn new(cache: PathBuf, host_name: String, host_id: Uuid) -> Result<Arc<Self>> {
        let lobby = Arc::new(Self {
            game_task: RwLock::new(None),
            disconnect_timers: DashMap::new(),
            global: Sender::new(100),
            private: DashMap::new(),
            state: RwLock::new(LobbyData::default()),
            submission_notify: Notify::new(),
            last_activity: RwLock::new(Instant::now()),
            czar_notify: Notify::new(),
            cache: cache.clone(),
        });

        // Initialize host in state
        {
            let mut guard = lobby.state.write().await;
            guard.settings = Settings::default();
            let all_decks = Deck::get_all_cached_info(cache, None).await?;
            guard.settings.decks = all_decks;
            guard.round = 1;
            guard.phase = GamePhase::LobbyOpen;
            let host_player = Player {
                info: PlayerInfo {
                    name: host_name,
                    is_host: true,
                    is_czar: false,
                    points: 0,
                },
                cards: Vec::new(),
            };
            guard.players.insert(host_id, host_player);
            guard.czar_order.push_back(host_id);
        }

        Ok(lobby)
    }

    /// Subscribe to global events
    pub fn subscribe_global(&self) -> Receiver<ServerEvent> {
        self.global.subscribe()
    }

    /// Subscribe to private events for a given player
    pub async fn subscribe_private(
        &self,
        player_id: Uuid,
    ) -> UnboundedReceiver<PrivateServerEvent> {
        let (tx, rx) = unbounded_channel();
        self.private.insert(player_id, tx);
        rx
    }

    /// Emit a global event without locking state
    pub fn emit_global(&self, event: ServerEvent) -> Result<()> {
        self.global.send(event)?;
        Ok(())
    }

    /// Emit a private event
    pub async fn emit_private(&self, player_id: &Uuid, event: PrivateServerEvent) -> Result<()> {
        if let Some(tx) = self.private.get(player_id) {
            tx.send(event)?;
            Ok(())
        } else {
            Err(Error::WebSocket(format!(
                "No private channel for {}",
                player_id
            )))
        }
    }

    /// Remove a player from the private emitter
    fn remove_private(&self, player_id: &Uuid) {
        self.private.remove(player_id);
    }

    /// Call this whenever something happens in the lobby
    async fn touch(&self) {
        let mut guard = self.last_activity.write().await;
        *guard = Instant::now();
    }

    /// Send the current lobby state globally
    pub async fn send_lobby_state(&self, player_id: &Uuid) -> Result<()> {
        let (players, settings, phase) = {
            let guard = self.state.read().await;
            let players = guard
                .players
                .iter()
                .map(|(&id, p)| (id, p.info.clone()))
                .collect();
            let settings = guard.settings.clone();
            let phase = guard.phase;

            (players, settings, phase)
        };

        self.touch().await;

        self.emit_private(
            player_id,
            PrivateServerEvent::LobbyState(LobbyState {
                players,
                settings,
                phase,
            }),
        )
        .await
    }

    /// Player joins the lobby
    pub async fn join(&self, player_name: String, player_id: Uuid) -> Result<()> {
        if !self.has_phase(GamePhase::LobbyOpen).await {
            return Err(Error::LobbyClosed);
        }

        let mut guard = self.state.write().await;
        let is_rejoining = guard.players.contains_key(&player_id);
        let has_host = guard.players.values().any(|p| p.info.is_host);

        if is_rejoining {
            if let Some((_, handle)) = self.disconnect_timers.remove(&player_id) {
                handle.abort();
            }

            if !has_host {
                if let Some(player) = guard.players.get_mut(&player_id) {
                    player.info.is_host = true;
                }
            }

            return Ok(());
        }

        // Not rejoining, new player join
        if guard.players.len() >= guard.settings.max_players as usize {
            return Err(Error::LobbyFull);
        }

        let is_first_player = guard.players.is_empty();

        let player_info = PlayerInfo {
            name: player_name,
            is_host: is_first_player,
            is_czar: false,
            points: 0,
        };

        guard.players.insert(
            player_id,
            Player {
                info: player_info.clone(),
                cards: Vec::new(),
            },
        );
        guard.czar_order.push_back(player_id);

        self.emit_global(ServerEvent::PlayerJoin {
            player_id,
            player_info,
        })?;

        self.touch().await;

        Ok(())
    }

    pub async fn kick(&self, own_id: &Uuid, player_id: &Uuid) -> Result<()> {
        if self.is_host(own_id).await && own_id != player_id {
            self.remove_player(player_id, Some(PrivateServerEvent::Kick))
                .await?;
        } else {
            return Err(Error::Unauthorized);
        }

        Ok(())
    }

    pub async fn player_disconnected(self: &Arc<Lobby>, player_id: Uuid) {
        // Make sure the player didn't got kicked or removed by anything beforehand
        let still_present = {
            let guard = self.state.read().await;
            guard.players.contains_key(&player_id)
        };
        if !still_present {
            return;
        }

        let in_lobby = self.has_phase(GamePhase::LobbyOpen).await;

        let lobby = self.clone();
        let handle = tokio::spawn(async move {
            // Wait the grace period only when in a lobby
            if in_lobby {
                tokio::time::sleep(GRACE_PERIOD).await;
            }
            // If player hasn't reconnected, remove them (also remove them from the disconnect_timers map)
            lobby.disconnect_timers.remove(&player_id);
            if let Err(e) = lobby
                .remove_player(&player_id, Some(PrivateServerEvent::Timeout))
                .await
            {
                error!("Timed out player '{}' with error: {:?}", player_id, e);
            }
        });
        self.disconnect_timers.insert(player_id, handle);
    }

    /// Use to remove a player from the lobby
    pub async fn remove_player(
        &self,
        player_id: &Uuid,
        event: Option<PrivateServerEvent>,
    ) -> Result<()> {
        let mut new_host_id: Option<Uuid> = None;
        let in_game;

        {
            let mut guard = self.state.write().await;
            let was_host = guard
                .players
                .get(player_id)
                .map(|p| p.info.is_host)
                .unwrap_or(false);

            in_game = !matches!(guard.phase, GamePhase::LobbyOpen | GamePhase::GameOver);
            guard.players.remove(player_id);
            guard.czar_order.retain(|id| id != player_id);

            if was_host {
                if let Some((&new_id, new_player)) = guard.players.iter_mut().next() {
                    new_player.info.is_host = true;
                    new_host_id = Some(new_id);
                }
            }
        }

        self.emit_global(ServerEvent::PlayerRemove {
            player_id: *player_id,
        })?;
        if let Some(new_id) = new_host_id {
            self.emit_global(ServerEvent::AssignHost { player_id: new_id })?;
        }
        if let Some(event) = event {
            self.emit_private(player_id, event).await?;
        }

        self.remove_private(player_id);

        // If a player was removed during a game, abort the game and end it for everyone
        if in_game {
            self.cancel_task().await;
            self.set_phase_and_emit(GamePhase::GameOver, ServerEvent::GameOver)
                .await?;
        }

        Ok(())
    }

    /// Update settings (host only)
    pub async fn update_settings(&self, player_id: &Uuid, new_settings: Settings) -> Result<()> {
        if self.is_host(player_id).await && self.has_phase(GamePhase::LobbyOpen).await {
            let to_remove: Vec<Uuid> = {
                let guard = self.state.read().await;
                let excess = guard
                    .players
                    .len()
                    .saturating_sub(new_settings.max_players as usize);
                guard
                    .players
                    .keys()
                    .filter(|&&id| id != *player_id) // never kick out the current player
                    .take(excess)
                    .cloned()
                    .collect()
            };

            for id in to_remove {
                self.remove_player(&id, Some(PrivateServerEvent::Kick))
                    .await?;
            }

            {
                let mut guard = self.state.write().await;
                guard.settings = new_settings.clone();
            }

            self.emit_global(ServerEvent::UpdateSettings {
                settings: new_settings,
            })
        } else {
            Err(Error::Unauthorized)
        }
    }

    pub async fn add_deck(&self, player_id: &Uuid, deckcode: String) -> Result<()> {
        if self.is_host(player_id).await && self.has_phase(GamePhase::LobbyOpen).await {
            let fetched = Deck::fetch(&deckcode).await?;
            fetched.save(self.cache.clone()).await?;

            let settings = {
                let guard = self.state.read().await;
                guard.settings.clone()
            };

            let decks: Vec<DeckInfo> =
                Deck::get_all_cached_info(self.cache.to_owned(), Some(settings.decks)).await?;

            // update settings
            {
                let mut guard = self.state.write().await;
                guard.settings.decks = decks.clone();
            }

            // send update
            self.touch().await;
            self.emit_global(ServerEvent::UpdateDecks { decks })
        } else {
            Err(Error::Unauthorized)
        }
    }

    /// Assigns the task handle to the game task
    pub async fn assign_task(&self, handle: JoinHandle<()>) {
        *self.game_task.write().await = Some(handle);
    }

    /// Cancels the game task if any exist
    pub async fn cancel_task(&self) {
        if let Some(handle) = self.game_task.write().await.take() {
            handle.abort();
        }
    }

    /// Start the game on a different thread
    pub async fn start_game(self: &Arc<Self>, player_id: &Uuid) -> Result<()> {
        if self.is_host(player_id).await {
            if self.has_phase(GamePhase::LobbyOpen).await
                && self.min_players().await
                && self.min_decks().await?
            {
                let lobby = self.clone();
                let handle = tokio::spawn(async move {
                    if let Err(e) = Lobby::run_game(lobby).await {
                        error!("Game loop exited with error: {:?}", e);
                    }
                });
                *self.game_task.write().await = Some(handle);

                Ok(())
            } else {
                Err(Error::LobbyStart)
            }
        } else {
            Err(Error::Unauthorized)
        }
    }

    /// Main game loop
    pub async fn run_game(self: Arc<Self>) -> Result<()> {
        self.reset_round().await?;

        loop {
            self.assign_czar().await?;
            self.submitting().await;

            let no_subs = { self.state.read().await.submissions.is_empty() };
            if no_subs {
                self.emit_global(ServerEvent::RoundSkip)?;
            } else {
                self.judging().await?;
            }

            // wait the normal time
            self.wait_time_secs().await;

            self.increment_round().await;

            let done = {
                let guard = self.state.read().await;
                guard.round > guard.settings.max_rounds
            };
            if done {
                break;
            }

            self.reset_round().await?;
        }

        self.set_phase_and_emit(GamePhase::GameOver, ServerEvent::GameOver)
            .await?;
        Ok(())
    }

    /// Refill cards and clear state
    async fn reset_round(&self) -> Result<()> {
        {
            let mut guard = self.state.write().await;
            guard.submissions.clear();
            guard.czar_pick = None;
            guard.black_card = None;
        }

        self.fill_white_cards().await?;

        Ok(())
    }

    /// Sleep according to settings.wait_time_secs
    async fn wait_time_secs(&self) {
        if let Some(secs) = self.state.read().await.settings.wait_time_secs {
            sleep(Duration::from_secs(secs)).await;
        }
    }

    /// Assigns the czar and deals black card
    async fn assign_czar(&self) -> Result<()> {
        let next = {
            let mut guard = self.state.write().await;
            guard.czar_order.pop_front()
        };

        if let Some(player_id) = next {
            {
                {
                    let mut guard = self.state.write().await;
                    guard
                        .players
                        .values_mut()
                        .for_each(|p| p.info.is_czar = false);
                    if let Some(p) = guard.players.get_mut(&player_id) {
                        p.info.is_czar = true;
                    }
                }

                let black_card = self.fill_black_card().await?;

                self.emit_global(ServerEvent::StartRound {
                    czar_id: player_id,
                    black_card,
                })
                .unwrap();
            }

            // re-queue
            {
                let mut guard = self.state.write().await;
                guard.czar_order.push_back(player_id);
            }
        }
        Ok(())
    }

    /// Collect submissions
    async fn submitting(&self) {
        self.set_phase(GamePhase::Submitting).await;

        let max_submitting_time_secs = {
            let guard = self.state.read().await;
            guard.settings.max_submitting_time_secs
        };
        if let Some(max) = max_submitting_time_secs {
            let timeout = sleep(Duration::from_secs(max));
            tokio::pin!(timeout);

            loop {
                if self.all_player_submitted().await {
                    break;
                }
                tokio::select! {
                    _ = &mut timeout => break,
                    _ = self.submission_notify.notified() => {}
                }
            }
        } else {
            while !self.all_player_submitted().await {
                self.submission_notify.notified().await;
            }
        }

        // now shuffle the submission array
        {
            let mut guard = self.state.write().await;
            let mut rng = rng();
            guard.submissions.shuffle(&mut rng);
        }
    }

    /// Czar picks winner
    async fn judging(&self) -> Result<()> {
        let cards = {
            let guard = self.state.read().await;
            guard.submissions.iter().map(|(_, c)| c.clone()).collect()
        };

        self.set_phase_and_emit(
            GamePhase::Judging,
            ServerEvent::RevealCards {
                selected_cards: cards,
            },
        )
        .await?;

        let max_judging_time_secs = {
            let guard = self.state.read().await;
            guard.settings.max_judging_time_secs
        };
        if let Some(max) = max_judging_time_secs {
            let timeout = sleep(Duration::from_secs(max));
            tokio::pin!(timeout);

            loop {
                if self.czar_submitted().await {
                    break;
                }
                tokio::select! {
                    _ = &mut timeout => break,
                    _ = self.czar_notify.notified() => {}
                }
            }
        } else {
            while !self.czar_submitted().await {
                self.czar_notify.notified().await;
            }
        }

        // award
        let czar_pick = {
            let guard = self.state.read().await;
            guard.czar_pick
        };
        if let Some(idx) = czar_pick {
            let player_id = {
                let mut guard = self.state.write().await;
                let (player_id, _) = guard.submissions[idx].clone();
                if let Some(p) = guard.players.get_mut(&player_id) {
                    p.info.points += 1;
                }
                player_id
            };

            self.set_phase_and_emit(
                GamePhase::RoundFinished,
                ServerEvent::RoundResult {
                    player_id,
                    winning_card_index: idx,
                },
            )
            .await?;
        } else {
            self.emit_global(ServerEvent::RoundSkip)?;
        }

        Ok(())
    }

    /// Fill a single black card
    async fn fill_black_card(&self) -> Result<BlackCard> {
        let settings = {
            let guard = self.state.read().await;
            guard.settings.clone()
        };
        let black = BlackCard::choose_random(self.cache.to_owned(), &settings).await?;
        {
            let mut guard = self.state.write().await;
            guard.black_card = Some(black.clone());
        }
        Ok(black)
    }

    /// Fill players' white hands
    async fn fill_white_cards(&self) -> Result<()> {
        let needs: Vec<(Uuid, usize)> = {
            let guard = self.state.read().await;
            guard
                .players
                .iter()
                .map(|(&id, p)| (id, 10usize.saturating_sub(p.cards.len())))
                .collect()
        };

        let settings = {
            let guard = self.state.read().await;
            guard.settings.clone()
        };

        let mut deals: Vec<(Uuid, Vec<WhiteCard>)> = Vec::with_capacity(needs.len());
        for (player_id, count) in needs {
            if count > 0 {
                let new_cards =
                    WhiteCard::choose_random(self.cache.to_owned(), count, &settings).await?;
                deals.push((player_id, new_cards));
            }
        }

        {
            let mut guard = self.state.write().await;
            for (player_id, new_cards) in &deals {
                if let Some(player) = guard.players.get_mut(player_id) {
                    player.cards.extend(new_cards.clone());
                }
            }
        }

        for (player_id, _new_cards) in deals {
            let hand = {
                let guard = self.state.read().await;
                guard.players.get(&player_id).map(|p| p.cards.clone())
            };
            if let Some(hand) = hand {
                self.emit_private(&player_id, PrivateServerEvent::UpdateHand { cards: hand })
                    .await?;
            }
        }

        Ok(())
    }

    /// Submit white cards
    pub async fn submit_cards(&self, player_id: &Uuid, indexes: Vec<usize>) -> Result<()> {
        let in_phase = {
            let guard = self.state.read().await;
            guard.phase == GamePhase::Submitting
        };
        let can_submit =
            { !self.is_czar(player_id).await && !self.has_submitted(player_id).await && in_phase };
        if !can_submit {
            return Err(Error::CardSubmission);
        }
        let black_card = {
            let guard = self.state.read().await;
            guard.black_card.clone().unwrap_or_default()
        };
        if black_card.fields != indexes.len() {
            return Err(Error::CardSubmission);
        }

        // perform submission
        {
            let mut guard = self.state.write().await;
            let player = guard
                .players
                .get_mut(player_id)
                .ok_or(Error::CardSubmission)?;

            // get selected cards
            let mut cards = Vec::new();
            for index in &indexes {
                cards.push(player.cards[*index].clone());
            }

            // remove selected indexes
            player.cards = player
                .cards
                .iter() // &Card
                .enumerate() // (usize, &Card)
                .filter(|&(i, _)| !indexes.contains(&i)) // keep those NOT in `indexes`
                .map(|(_, card)| card.clone()) // clone the Card out
                .collect();

            guard.submissions.push((*player_id, cards));
        };

        self.submission_notify.notify_one();
        self.emit_global(ServerEvent::CardsSubmitted {
            player_id: *player_id,
        })
    }

    /// Czar submits choice
    pub async fn submit_czar_choice(&self, player_id: &Uuid, index: usize) -> Result<()> {
        let is_czar = self.is_czar(player_id).await;
        if !is_czar
            || self.czar_submitted().await
            || self.state.read().await.phase != GamePhase::Judging
        {
            return Err(Error::CzarChoice);
        }
        {
            let mut guard = self.state.write().await;
            guard.czar_pick = Some(index);
        }
        self.czar_notify.notify_one();
        Ok(())
    }

    /// Helpers
    async fn increment_round(&self) {
        self.state.write().await.round += 1;
    }

    async fn set_phase(&self, phase: GamePhase) {
        let mut guard = self.state.write().await;
        guard.phase = phase;

        self.touch().await;
    }

    async fn set_phase_and_emit(&self, phase: GamePhase, evt: ServerEvent) -> Result<()> {
        self.set_phase(phase).await;
        self.emit_global(evt)
    }

    pub async fn reset_game(&self, player_id: &Uuid) -> Result<()> {
        if self.is_host(player_id).await && self.has_phase(GamePhase::GameOver).await {
            let mut guard = self.state.write().await;
            guard.round = 1;
            guard.phase = GamePhase::LobbyOpen;
            for p in guard.players.values_mut() {
                p.info.is_czar = false;
                p.info.points = 0;
                p.cards.clear();
            }

            self.emit_global(ServerEvent::LobbyReset)
        } else {
            Err(Error::Unauthorized)
        }
    }

    pub async fn min_players(&self) -> bool {
        let guard = self.state.read().await;
        guard.players.len() >= 2
    }

    pub async fn min_decks(&self) -> Result<bool> {
        let guard = self.state.read().await;
        let decks_enabled = guard.settings.decks.iter().any(|f| f.enabled);

        let settings = &guard.settings;
        let decks = Deck::get_enabled(self.cache.clone(), settings).await?;

        let has_all_kinds = decks.iter().any(|f| !f.blacks.is_empty())
            && decks.iter().any(|f| !f.whites.is_empty());

        Ok(decks_enabled && has_all_kinds)
    }

    pub async fn has_phase(&self, phase: GamePhase) -> bool {
        let guard = self.state.read().await;
        guard.phase == phase
    }

    pub async fn is_czar(&self, player_id: &Uuid) -> bool {
        let guard = self.state.read().await;
        guard
            .players
            .get(player_id)
            .map(|p| p.info.is_czar)
            .unwrap_or(false)
    }

    pub async fn is_host(&self, player_id: &Uuid) -> bool {
        let guard = self.state.read().await;
        guard
            .players
            .get(player_id)
            .map(|p| p.info.is_host)
            .unwrap_or(false)
    }

    pub async fn has_submitted(&self, player_id: &Uuid) -> bool {
        let guard = self.state.read().await;
        guard.submissions.iter().any(|(id, _)| id == player_id)
    }

    pub async fn czar_submitted(&self) -> bool {
        let guard = self.state.read().await;
        guard.czar_pick.is_some()
    }

    pub async fn all_player_submitted(&self) -> bool {
        let guard = self.state.read().await;
        guard.submissions.len() == guard.players.len() - 1
    }
}
