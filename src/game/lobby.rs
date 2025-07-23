use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::time::sleep;
use tokio::{
    sync::{
        broadcast::{Receiver, Sender},
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Notify, RwLock,
    },
    time::Instant,
};
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    game::{
        deck::{BlackCard, Deck, DeckInfo, WhiteCard},
        LobbyState, Player, PlayerInfo, PrivateServerEvent, ServerEvent, Settings,
    },
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
    pub global: Sender<ServerEvent>, // broadcast to all clients
    pub private: RwLock<HashMap<Uuid, UnboundedSender<PrivateServerEvent>>>,
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

impl Lobby {
    /// Create a new lobby with host as first player.
    pub async fn new(cache: PathBuf, host_name: String, host_id: Uuid) -> Result<Arc<Self>> {
        let lobby = Arc::new(Self {
            global: Sender::new(100),
            private: RwLock::new(HashMap::new()),
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
        self.private.write().await.insert(player_id, tx);
        rx
    }

    /// Emit a global event without locking state
    pub fn emit_global(&self, event: ServerEvent) -> Result<()> {
        self.global.send(event)?;
        Ok(())
    }

    /// Emit a private event
    pub async fn emit_private(&self, player_id: &Uuid, event: PrivateServerEvent) -> Result<()> {
        let map = self.private.read().await;
        if let Some(tx) = map.get(player_id) {
            tx.send(event)?;
            Ok(())
        } else {
            Err(Error::WebSocket(format!(
                "No private channel for {}",
                player_id
            )))
        }
    }

    /// Call this whenever something happens in the lobby
    async fn touch(&self) {
        let mut guard = self.last_activity.write().await;
        *guard = Instant::now();
    }

    /// Send the current lobby state globally
    pub async fn send_lobby_state(&self) -> Result<()> {
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

        self.emit_global(ServerEvent::LobbyState(LobbyState {
            players,
            settings,
            phase,
        }))
    }

    /// Player joins the lobby
    pub async fn join(&self, player_name: String, player_id: Uuid) -> Result<()> {
        {
            let guard = self.state.read().await;
            if guard.phase != GamePhase::LobbyOpen {
                return Err(Error::LobbyClosed);
            }
            if guard.players.contains_key(&player_id) {
                return Ok(());
            }
            if guard.players.len() >= guard.settings.max_players as usize {
                return Err(Error::LobbyFull);
            }
        }
        // upgrade to write lock to add player
        {
            let mut guard = self.state.write().await;
            let new_player = Player {
                info: PlayerInfo {
                    name: player_name,
                    is_host: false,
                    is_czar: false,
                    points: 0,
                },
                cards: Vec::new(),
            };
            guard.players.insert(player_id, new_player);
            guard.czar_order.push_back(player_id);
        }

        self.touch().await;

        Ok(())
    }

    pub async fn kick(&self, own_id: &Uuid, player_id: &Uuid) -> Result<()> {
        if self.is_host(own_id).await && own_id != player_id {
            self.remove_player(player_id).await?;
        } else {
            return Err(Error::Unauthorized);
        }

        Ok(())
    }

    async fn remove_player(&self, player_id: &Uuid) -> Result<()> {
        {
            let mut guard = self.state.write().await;
            guard.players.remove(player_id);
            guard.czar_order.retain(|id| id != player_id);
        }
        let players = {
            let guard = self.state.read().await;
            guard
                .players
                .iter()
                .map(|(&id, p)| (id, p.info.clone()))
                .collect()
        };

        self.touch().await;

        self.emit_global(ServerEvent::UpdatePlayers { players })?;
        self.emit_private(player_id, PrivateServerEvent::Kick)
            .await?;

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
                self.remove_player(&id).await?;
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

    /// Start the game on a different thread
    pub async fn start_game(self: &Arc<Self>, player_id: &Uuid) -> Result<()> {
        if self.is_host(player_id).await {
            if self.has_phase(GamePhase::LobbyOpen).await
                && self.min_players().await
                && self.min_decks().await?
            {
                let lobby = self.clone();
                tokio::spawn(async move {
                    let _ = Lobby::run_game(lobby).await;
                });
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
                    player_id,
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

        // wait the normal time
        self.wait_time_secs().await;

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
                let cards =
                    WhiteCard::choose_random(self.cache.to_owned(), count, &settings).await?;
                deals.push((player_id, cards));
            }
        }

        {
            let mut guard = self.state.write().await;
            for (player_id, cards) in deals {
                if let Some(player) = guard.players.get_mut(&player_id) {
                    player.cards.extend(cards);
                }
            }
        }

        let snapshot = {
            let guard = self.state.read().await;
            guard
                .players
                .iter()
                .map(|(&id, p)| (id, p.cards.clone()))
                .collect::<Vec<_>>()
        };
        for (id, hand) in snapshot {
            self.emit_private(&id, PrivateServerEvent::UpdateHand { cards: hand })
                .await?;
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
            for index in &indexes {
                player.cards.remove(*index);
            }

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
            {
                let mut guard = self.state.write().await;
                guard.round = 1;
                guard.phase = GamePhase::LobbyOpen;
                self.global.send(ServerEvent::LobbyReset)?;
                for p in guard.players.values_mut() {
                    p.info.is_czar = false;
                    p.info.points = 0;
                    p.cards.clear();
                }
            }
            self.send_lobby_state().await
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
