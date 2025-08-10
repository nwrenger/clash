<script lang="ts" module>
	export interface Connection {
		ws?: WebSocket;
		connected: boolean;
	}

	export interface Own {
		id: api.Uuid;
		name: string;
		logged_in: boolean;
		cards: api.WhiteCard[];
		selected_cards: number[];
	}

	export interface Lobby {
		id: string;
		state?: api.LobbyState;
	}

	export interface Round {
		count: number;
		black_card?: api.BlackCard;
		revealed_cards: api.WhiteCard[][];
		result?: { player_id: api.Uuid; winning_card_index: number };
		time: { self?: number };
	}
</script>

<script lang="ts">
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import { beforeNavigate } from '$app/navigation';

	import api from '$lib/api';
	import { credentials } from '$lib/state';
	import { sortedEntries } from '$lib/utils';
	import { show_error, toaster } from '$lib/toaster';

	import Joining from './Joining.svelte';
	import LobbyOpen from './LobbyOpen.svelte';
	import TopBar from './TopBar.svelte';
	import Hand from './Hand.svelte';
	import Board from './Board.svelte';
	import GameOver from './GameOver.svelte';

	let connection: Connection = $state({
		ws: undefined,
		connected: false
	});

	let own: Own = $state({
		id: crypto.randomUUID(),
		name: '',
		logged_in: false,
		cards: [],
		selected_cards: []
	});

	let lobby: Lobby = $state({
		id: '',
		state: undefined
	});

	let round: Round = $state({
		count: 0,
		black_card: undefined,
		revealed_cards: [],
		result: undefined,
		time: {}
	});

	let joining = $derived(!lobby.state && connection.connected);
	let open = $derived(lobby.state?.phase == 'LobbyOpen');
	let submitting = $derived(lobby.state?.phase == 'Submitting');
	let judging = $derived(lobby.state?.phase == 'Judging');
	let finished = $derived(lobby.state?.phase == 'RoundFinished');
	let gaming = $derived(submitting || judging || finished);
	let over = $derived(lobby.state?.phase == 'GameOver');

	// Prevent Navigation when in Game/Game Over
	beforeNavigate(({ cancel, type }) => {
		if ((gaming || over) && type != 'leave') {
			toaster.warning({
				title: 'Navigation Cancelled',
				description: 'Your Navigation was cancelled due to a running game!'
			});
			cancel();
		}
	});

	onDestroy(() => {
		connection.ws?.close();
	});

	onMount(() => {
		lobby.id = page.url.searchParams.get('id') || '';

		// get stored data if already logged in
		own.logged_in = $credentials?.lobby_id == lobby.id;
		if ($credentials && own.logged_in) {
			own.id = $credentials.id;
			own.name = $credentials.name;
		}

		connect();
	});

	function connect() {
		connection.ws = api.connect_ws(lobby.id as api.Uuid);

		connection.ws.onopen = () => {
			connection.connected = true;
			// Join lobby if player did already join once
			if (own.logged_in) join_lobby();
		};

		connection.ws.onerror = () => {
			show_error({ kind: 'LobbyNotFound' });
		};

		connection.ws.onmessage = (event) => {
			const msg: api.IncommingEvent = JSON.parse(event.data);
			handleIncommingEvent(msg);
		};

		connection.ws.onclose = disconnect;
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
			case 'LobbyState':
				return onLobbyState(msg);
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
		if (!lobby.state) return;

		let joined_id = msg.data.player_id;
		let joined_info = msg.data.player_info;
		lobby.state.players[joined_id] = joined_info;
	}

	function onPlayerRemove(msg: Extract<api.IncommingEvent, { type: 'PlayerRemove' }>) {
		if (!lobby.state) return;

		let kicked_id = msg.data.player_id;
		delete lobby.state.players[kicked_id];

		if (gaming) {
			toaster.error({
				title: 'Game Interrupted',
				description: 'A player left the lobby during the game, so it could not continue.'
			});
		}
	}

	function onAssignHost(msg: Extract<api.IncommingEvent, { type: 'AssignHost' }>) {
		if (!lobby.state) return;

		let host_id = msg.data.player_id;
		let new_host = lobby.state.players[host_id];
		new_host.is_host = true;
		if (host_id === own.id) {
			toaster.warning({
				title: 'You Are Now the Host',
				description:
					'The previous host has left the lobby, so you have been assigned as the new host.'
			});
		}
	}

	function onStartRound(msg: Extract<api.IncommingEvent, { type: 'StartRound' }>) {
		if (!lobby.state) return;

		setPhase('Submitting');
		round.count += 1;

		resetRound();

		let czar_id = msg.data.czar_id;
		let czar = lobby.state.players[czar_id];
		if (czar) {
			czar.is_czar = true;
			if (czar_id == own.id) {
				toaster.info({ title: `You are the Czar!` });
			} else {
				toaster.info({ title: `${czar.name} is the Czar!` });
			}
		}

		round.black_card = msg.data.black_card;
	}

	function onCardsSubmitted(msg: Extract<api.IncommingEvent, { type: 'CardsSubmitted' }>) {
		if (!lobby.state || !round.black_card) return;

		let id = msg.data.player_id;
		let player_submit = lobby.state.players[id];
		let placeholders = [];
		for (let i = 0; i < round.black_card.fields; i++) {
			if (id === own.id) {
				placeholders.push({ text: 'Your Card' });
			} else {
				placeholders.push({ text: `Card by ${player_submit.name}` });
			}
		}
		round.revealed_cards.push(placeholders);
	}

	function onUpdateDecks(msg: Extract<api.IncommingEvent, { type: 'UpdateDecks' }>) {
		if (!lobby.state) return;
		lobby.state.settings.decks = msg.data.decks;
	}

	function onUpdateSettings(msg: Extract<api.IncommingEvent, { type: 'UpdateSettings' }>) {
		if (!lobby.state) return;
		lobby.state.settings = msg.data.settings;
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
		if (!lobby.state) return;

		setPhase('RoundFinished');
		round.result = msg.data;

		let winner = lobby.state.players[round.result.player_id];
		winner.points += 1;
		if (round.result.player_id == own.id) {
			toaster.info({ title: `You are the winner of this round!` });
		} else {
			toaster.info({ title: `${winner.name} is the winner of this round!` });
		}
	}

	function onGameOver() {
		setPhase('GameOver');
	}

	function onLobbyReset() {
		if (!lobby.state) return;

		setPhase('LobbyOpen');
		for (const [_, player] of sortedEntries(lobby.state.players)) {
			player.points = 0;
		}
		round.count = 0;
	}

	function onLobbyState(msg: Extract<api.IncommingEvent, { type: 'LobbyState' }>) {
		lobby.state = msg.data;
	}

	function onUpdateHand(msg: Extract<api.IncommingEvent, { type: 'UpdateHand' }>) {
		own.cards = msg.data.cards;
	}

	function onTimeout() {
		toaster.warning({
			title: 'Timed Out',
			description: 'You have been removed from the lobby due to inactivity. Reload to reconnect!'
		});

		// Clear state directly to improve ux
		disconnect();
	}

	function onKick() {
		toaster.warning({
			title: 'Kicked',
			description: 'You have been removed from the lobby by the host.'
		});

		// Clear state directly to improve ux
		disconnect();

		// Close ws
		connection.ws?.close();

		// Remove remembered credentials
		resetLogin();

		// reconnect
		connect();
	}

	function onError(msg: Extract<api.IncommingEvent, { type: 'Error' }>) {
		show_error(msg.data);
	}

	function resetRound() {
		own.selected_cards = [];
		round.black_card = undefined;
		round.revealed_cards = [];
		round.result = undefined;
		for (const [_, player] of sortedEntries(lobby.state?.players)) {
			player.is_czar = false;
		}
	}

	function resetLogin() {
		own.name = '';
		own.id = crypto.randomUUID();
		own.logged_in = false;
		$credentials = null;
	}

	function disconnect() {
		lobby.state = undefined;
		connection.connected = false;
	}

	function join_lobby() {
		// Make sure to reconnect if an error closed the connection
		// Like when trying to join a full or closed lobby
		if (!connection.connected) connect();
		// Connect and update credentials
		api.send_ws(connection.ws!, { type: 'JoinLobby', data: { name: own.name, id: own.id } });
		$credentials = { lobby_id: lobby.id as api.Uuid, id: own.id, name: own.name };
	}

	function setPhase(phase: api.GamePhase) {
		if (!lobby.state) return;
		lobby.state.phase = phase;

		let change: number | undefined;
		if (phase == 'RoundFinished') change = lobby.state.settings.wait_time_secs;
		if (phase == 'Submitting') change = lobby.state.settings.max_submitting_time_secs;
		if (phase == 'Judging') change = lobby.state.settings.max_judging_time_secs;
		round.time = { self: change };
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
		if (gaming || over) {
			e.preventDefault();
			return '';
		}
	}}
/>

{#if joining}
	<Joining bind:own_name={own.name} {join_lobby} />
{:else if open}
	<LobbyOpen {connection} {lobby} {own} />
{:else if gaming}
	{@const is_czar = lobby.state?.players[own.id]?.is_czar || false}
	<div class="mx-auto flex max-w-7xl flex-col items-center space-y-6 px-4 py-8">
		<TopBar {lobby} {own} {round} />

		<Board {connection} {round} selectable={is_czar && judging} />

		<Hand {connection} {round} bind:own selectable={!is_czar && submitting} disabled={is_czar} />
	</div>
{:else if over}
	<GameOver {connection} {lobby} {own} />
{:else}
	<div class="mx-auto flex max-w-3xl flex-col items-center space-y-6 px-4 py-8">
		<p>Loading...</p>
		<a href="/" class="btn preset-filled-secondary-500">Close Connection</a>
	</div>
{/if}
