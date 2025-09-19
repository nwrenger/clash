<script lang="ts" module>
	export interface Connection {
		ws?: WebSocket;
		connected: boolean;
	}

	export interface Own {
		credentials: api.Credentials;
		logged_in: boolean;
		cards: api.WhiteCard[];
		selected_cards: number[];
	}

	export type Lobby =
		| {
				id: string;
				joined: false;
				players?: Record<api.Uuid, api.PlayerInfo>;
				settings?: api.Settings;
				phase?: api.GamePhase;
		  }
		| {
				id: string;
				joined: true;
				players: Record<api.Uuid, api.PlayerInfo>;
				settings: api.Settings;
				phase: api.GamePhase;
		  };

	export interface Round {
		count: number;
		black_card?: api.BlackCard;
		revealed_cards: api.WhiteCard[][];
		result?: { player_id?: api.Uuid; winning_card_index?: number };
		time: { self?: number };
	}
</script>

<script lang="ts">
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';

	import api from '$lib/api';
	import { session } from '$lib/state';
	import { sortedEntries } from '$lib/utils';
	import { show_error, toaster } from '$lib/toaster';

	import Joining from './Joining.svelte';
	import LobbyOpen from './LobbyOpen.svelte';
	import TopBar from './TopBar.svelte';
	import Hand from './Hand.svelte';
	import Board from './Board.svelte';
	import GameOver from './GameOver.svelte';
	import Loading from './Loading.svelte';

	let connection: Connection = $state({
		ws: undefined,
		connected: false
	});

	let own: Own = $state({
		credentials: { name: '', id: crypto.randomUUID(), secret: crypto.randomUUID() },
		logged_in: false,
		cards: [],
		selected_cards: []
	});

	let lobby: Lobby = $state({
		id: '',
		joined: false,
		players: undefined,
		settings: undefined,
		phase: undefined
	});

	let round: Round = $state({
		count: 0,
		black_card: undefined,
		revealed_cards: [],
		result: undefined,
		time: {}
	});

	let joining = $derived(!lobby.joined && connection.connected);
	let open = $derived(lobby?.phase == 'LobbyOpen');
	let submitting = $derived(lobby?.phase == 'Submitting');
	let judging = $derived(lobby?.phase == 'Judging');
	let finished = $derived(lobby?.phase == 'RoundFinished');
	let gaming = $derived(submitting || judging || finished);
	let over = $derived(lobby?.phase == 'GameOver');

	onDestroy(() => {
		// Gracefully close and mark disconnected, but don't auto-rejoin
		try {
			connection.ws?.close();
		} finally {
			connection.connected = false;
		}
	});

	onMount(() => {
		lobby.id = page.url.searchParams.get('id') || '';

		// Get stored data if already logged in
		own.logged_in = $session?.lobby_id == lobby.id;
		if ($session && own.logged_in) {
			own.credentials = $session.credentials;
		}

		connect();
	});

	function connect() {
		// Avoid duplicate sockets
		if (
			connection.ws &&
			(connection.ws.readyState === WebSocket.OPEN ||
				connection.ws.readyState === WebSocket.CONNECTING)
		) {
			console.log(connection);
			return;
		}

		connection.ws = api.connect_ws(lobby.id as api.Uuid);

		connection.ws.onopen = () => {
			connection.connected = true;
			// Join lobby if player did already join once
			if (own.logged_in) join_lobby();
		};

		connection.ws.onerror = () => {
			// If lobby truly doesn't exist, show error. Otherwise, onclose will handle cleanup
			show_error({ kind: 'LobbyNotFound' });
		};

		connection.ws.onmessage = (event) => {
			const msg: api.IncommingEvent = JSON.parse(event.data);
			handleIncommingEvent(msg);
		};

		connection.ws.onclose = () => {
			// Land in Loading screen (disconnected). Keep credentials/session unless kicked
			connection.connected = false;
			removeState();
		};
	}

	function handleIncommingEvent(msg: api.IncommingEvent) {
		switch (msg.type) {
			case 'PlayerJoin':
				return onPlayerJoin(msg);
			case 'PlayerRemove':
				return onPlayerRemove(msg);
			case 'AssignHost':
				return onAssignHost(msg);
			case 'StartRound':
				return onStartRound(msg);
			case 'CardsSubmitted':
				return onCardsSubmitted(msg);
			case 'UpdateDecks':
				return onUpdateDecks(msg);
			case 'UpdateSettings':
				return onUpdateSettings(msg);
			case 'RevealCards':
				return onRevealCards(msg);
			case 'RoundSkip':
				return onRoundSkip();
			case 'RoundResult':
				return onRoundResult(msg);
			case 'GameOver':
				return onGameOver();
			case 'LobbyReset':
				return onLobbyReset();
			case 'ClientLobby':
				return onClientLobby(msg);
			case 'UpdateHand':
				return onUpdateHand(msg);
			case 'Timeout':
				return onTimeout();
			case 'Kick':
				return onKick();
			case 'Error':
				return onError(msg);
		}
	}

	function onPlayerJoin(msg: Extract<api.IncommingEvent, { type: 'PlayerJoin' }>) {
		if (!lobby.joined) return;

		let joined_id = msg.data.player_id;
		let joined_info = msg.data.player_info;
		lobby.players[joined_id] = joined_info;
	}

	function onPlayerRemove(msg: Extract<api.IncommingEvent, { type: 'PlayerRemove' }>) {
		if (!lobby.joined) return;

		let kicked_id = msg.data.player_id;
		delete lobby.players[kicked_id];

		if (gaming) {
			toaster.error({
				title: 'Game Interrupted',
				description: 'A player left the lobby during the game, so it could not continue.'
			});
		}
	}

	function onAssignHost(msg: Extract<api.IncommingEvent, { type: 'AssignHost' }>) {
		if (!lobby.joined) return;

		let host_id = msg.data.player_id;
		let new_host = lobby.players[host_id];
		new_host.is_host = true;
		if (host_id === own.credentials.id) {
			toaster.warning({
				title: 'You Are Now the Host',
				description:
					'The previous host has left the lobby, so you have been assigned as the new host.'
			});
		}
	}

	function onStartRound(msg: Extract<api.IncommingEvent, { type: 'StartRound' }>) {
		if (!lobby.joined) return;

		setPhase('Submitting');
		round.count += 1;

		resetRound();

		let czar_id = msg.data.czar_id;
		let czar = lobby.players[czar_id];
		if (czar) {
			czar.is_czar = true;
			if (czar_id == own.credentials.id) {
				toaster.info({ title: `You are the Czar!` });
			} else {
				toaster.info({ title: `${czar.name} is the Czar!` });
			}
		}

		round.black_card = msg.data.black_card;
	}

	function onCardsSubmitted(msg: Extract<api.IncommingEvent, { type: 'CardsSubmitted' }>) {
		if (!lobby.joined || !round.black_card) return;

		let id = msg.data.player_id;
		let player_submit = lobby.players[id];
		let placeholders = [];
		for (let i = 0; i < round.black_card.fields; i++) {
			if (id === own.credentials.id) {
				placeholders.push({ text: 'Your Card' });
			} else {
				placeholders.push({ text: `Card by ${player_submit.name}` });
			}
		}
		round.revealed_cards.push(placeholders);
	}

	function onUpdateDecks(msg: Extract<api.IncommingEvent, { type: 'UpdateDecks' }>) {
		if (!lobby.joined) return;
		lobby.settings.decks = msg.data.decks;
	}

	function onUpdateSettings(msg: Extract<api.IncommingEvent, { type: 'UpdateSettings' }>) {
		if (!lobby.joined) return;
		lobby.settings = msg.data.settings;
	}

	function onRevealCards(msg: Extract<api.IncommingEvent, { type: 'RevealCards' }>) {
		setPhase('Judging');
		round.revealed_cards = msg.data.selected_cards;
	}

	function onRoundSkip() {
		setPhase('RoundFinished');
		toaster.info({
			title: 'Skipped',
			description:
				'The current round was skipped due to the players or the Czar not selecting any cards.'
		});
	}

	function onRoundResult(msg: Extract<api.IncommingEvent, { type: 'RoundResult' }>) {
		if (!lobby.joined) return;

		setPhase('RoundFinished');
		round.result = msg.data;

		if (round.result.player_id) {
			let winner = lobby.players[round.result.player_id];
			winner.points += 1;
			if (round.result.player_id == own.credentials.id) {
				toaster.info({ title: `You are the winner of this round!` });
			} else {
				toaster.info({ title: `${winner.name} is the winner of this round!` });
			}
		}
	}

	function onGameOver() {
		setPhase('GameOver');
	}

	function onLobbyReset() {
		if (!lobby.joined) return;

		setPhase('LobbyOpen');
		for (const [_, player] of sortedEntries(lobby.players)) {
			player.points = 0;
		}
		round.count = 0;
	}

	function onClientLobby(msg: Extract<api.IncommingEvent, { type: 'ClientLobby' }>) {
		lobby.joined = true;
		lobby.players = msg.data.players;
		lobby.settings = msg.data.settings;
		lobby.phase = msg.data.phase;

		round.count = msg.data.round;
		round.black_card = msg.data.black_card;
		if (msg.data.revealed_cards) round.revealed_cards = msg.data.revealed_cards;
		if (msg.data.czar_pick != null) round.result = { winning_card_index: msg.data.czar_pick };
		if (msg.data.winner != null) round.result = { player_id: msg.data.winner, ...round.result };

		if (msg.data.submitted_players)
			for (const submit_id of msg.data.submitted_players) {
				onCardsSubmitted({ type: 'CardsSubmitted', data: { player_id: submit_id } });
			}

		if (msg.data.selected_cards) own.selected_cards = msg.data.selected_cards;
		if (msg.data.hand) own.cards = msg.data.hand;

		// We joined, but we still keep explicit-join flow for future reconnects
		own.logged_in = true;
	}

	function onUpdateHand(msg: Extract<api.IncommingEvent, { type: 'UpdateHand' }>) {
		own.cards = msg.data.cards;
	}

	function onTimeout() {
		const was_joined = lobby.joined;

		toaster.warning({
			title: 'Timed Out',
			description: was_joined
				? 'You were removed due to inactivity. Use the Loading screen to reconnect.'
				: 'Connection timed out. Use the Loading screen to reconnect.'
		});

		// Keep credentials/session so the user can reconnect manually
		closeWs();
		removeState();
	}

	function onKick() {
		toaster.warning({
			title: 'Kicked',
			description: 'You have been removed from the lobby by the host.'
		});

		// Kick: fully clear state and credentials; user may reconnect manually and rejoin
		disconnect();
	}

	function onError(msg: Extract<api.IncommingEvent, { type: 'Error' }>) {
		show_error(msg.data);
	}

	function resetRound() {
		own.selected_cards = [];
		round.black_card = undefined;
		round.revealed_cards = [];
		round.result = undefined;
		for (const [_, player] of sortedEntries(lobby?.players)) {
			player.is_czar = false;
		}
	}

	function resetLogin() {
		own.credentials = { name: '', id: crypto.randomUUID(), secret: crypto.randomUUID() };
		own.logged_in = false;
		$session = null;
	}

	function removeState() {
		lobby.joined = false;
		lobby.phase = undefined;
		lobby.players = undefined;
		lobby.settings = undefined;
	}

	function closeWs() {
		connection.connected = false;
		if (connection.ws) {
			try {
				connection.ws.onopen = null;
				connection.ws.onmessage = null;
				connection.ws.onclose = null;
				connection.ws.onerror = null;
				if (
					connection.ws.readyState === WebSocket.OPEN ||
					connection.ws.readyState === WebSocket.CONNECTING
				) {
					connection.ws.close();
				}
			} catch {}
			connection.ws = undefined;
		}
	}

	function disconnect() {
		closeWs();
		removeState();
		resetLogin();
	}

	function join_lobby(e?: Event) {
		if (!own.credentials.name.trim()) return;
		if (e) e.preventDefault();

		// Ensure we have a live connection; do not auto-join on connect
		if (!connection.ws || connection.ws.readyState !== WebSocket.OPEN) {
			connect();
			return;
		}

		// Send join only when the user explicitly presses Join
		api.send_ws(connection.ws, { type: 'JoinLobby', data: { credentials: own.credentials } });
		$session = { lobby_id: lobby.id as api.Uuid, credentials: own.credentials };
	}

	function setPhase(phase: api.GamePhase) {
		if (!lobby.joined) return;
		lobby.phase = phase;

		let change: number | undefined;
		if (phase == 'RoundFinished') change = lobby.settings!.wait_time_secs || undefined;
		if (phase == 'Submitting') change = to_seconds(lobby.settings!.max_submitting_time_secs, lobby);
		if (phase == 'Judging') change = lobby.settings!.max_judging_time_secs || undefined;
		round.time = { self: change };
	}

	function to_seconds(scaling: api.Scaling | null, lobby: Lobby) {
		if (scaling) {
			switch (scaling.type) {
				case 'Player':
					let player_count = Object.keys(lobby.players || {}).length;
					return scaling.seconds * player_count;
				case 'Constant':
					return scaling.seconds;
			}
		}
	}
</script>

<svelte:head>
	<title>Playing in a Lobby - Cards (Ludicrous ones) Against Humanity</title>
	<meta name="description" content="Playing in a Lobby! Time to create great combinations." />
	<!-- Open Graph -->
	<meta property="og:title" content="Play in a Lobby - Cards (Ludicrous ones) Against Humanity" />
	<meta
		property="og:description"
		content="Join this Cards (Ludicrous ones) Against Humanity (CLASH) lobby, an open‑source Cards Against Humanity game. Jump in, pick your wildest cards, and let the chaos and laughs begin!"
	/>
</svelte:head>

<!-- Prevent Closing when in Game/Game Over -->
<svelte:window
	onbeforeunload={(e) => {
		e.preventDefault();
		return '';
	}}
/>

{#if joining}
	<Joining bind:own_name={own.credentials.name} {join_lobby} />
{:else if open}
	<LobbyOpen {connection} {lobby} {own} {disconnect} />
{:else if gaming}
	{@const is_czar = lobby!.players![own.credentials.id]?.is_czar || false}
	<div class="mx-auto flex max-w-7xl flex-col items-center space-y-6 px-4 py-8">
		<TopBar {connection} {lobby} {own} {round} />

		<Board {connection} {round} selectable={is_czar && judging} />

		<Hand {connection} {round} bind:own selectable={!is_czar && submitting} disabled={is_czar} />
	</div>
{:else if over}
	<GameOver {connection} {lobby} {own} />
{:else}
	<Loading {connect} />
{/if}
