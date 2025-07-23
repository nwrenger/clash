<script lang="ts">
	import { page } from '$app/state';
	import api from '$lib/api';
	import { own } from '$lib/state';
	import { error_toast, show_error, toaster } from '$lib/toaster';
	import { onDestroy, onMount } from 'svelte';
	import LobbyOpen from './LobbyOpen.svelte';
	import TopBar from './TopBar.svelte';
	import Hand from './Hand.svelte';
	import BlackCard from './BlackCard.svelte';
	import { sortedEntries } from '$lib/utils';
	import Joining from './Joining.svelte';
	import GameOver from './GameOver.svelte';

	let lobby_id: string = $state('');
	let logged_in: boolean = $state(false);
	let own_id: api.Uuid = $state(crypto.randomUUID());
	let own_name = $state('');

	let ws: WebSocket | undefined = $state();
	let connected = $state(false);

	let lobby_state: api.LobbyState | undefined = $state();
	let cards: api.WhiteCard[] = $state([]);
	let self_selected_cards: number[] = $state([]);
	let revealed_cards: api.WhiteCard[][] = $state([]);
	let black_card: api.BlackCard | undefined = $state();
	let round = $state(0);
	let round_result: { player_id: api.Uuid; winning_card_index: number } | undefined = $state();

	let isJoining = $derived(!lobby_state && connected);
	let isOpen = $derived(lobby_state?.phase == 'LobbyOpen');
	let isSubmitting = $derived(lobby_state?.phase == 'Submitting');
	let isJudging = $derived(lobby_state?.phase == 'Judging');
	let isFinished = $derived(lobby_state?.phase == 'RoundFinished');
	let isGaming = $derived(isSubmitting || isJudging || isFinished);
	let isOver = $derived(lobby_state?.phase == 'GameOver');
	let time: { self?: number } = $state({});

	onDestroy(() => {
		ws?.close();
	});

	onMount(() => {
		lobby_id = page.url.searchParams.get('id') || '';
		logged_in = $own?.lobby_id == lobby_id;
		own_id = $own && logged_in ? $own.id : crypto.randomUUID();
		own_name = $own && logged_in ? $own.name : '';

		connect();
	});

	function connect() {
		ws = api.connect_ws(lobby_id as api.Uuid);

		ws.onopen = () => {
			connected = true;
			// joins lobby if player did already join once
			if (logged_in) join_lobby();
		};

		ws.onerror = () => {
			show_error({ kind: 'LobbyNotFound' });
		};

		ws.onmessage = (event) => {
			const msg: api.IncommingEvent = JSON.parse(event.data);
			switch (msg.type) {
				case 'LobbyState':
					lobby_state = msg.data;
					break;
				case 'StartRound':
					set_phase('Submitting');
					round += 1;

					// reset everything
					self_selected_cards = [];
					black_card = undefined;
					revealed_cards = [];
					round_result = undefined;
					for (const [_, player] of sortedEntries(lobby_state?.players)) {
						player.is_czar = false;
					}

					// apply the send state
					if (lobby_state) {
						let id = msg.data.player_id;
						let czar = lobby_state.players[id];
						if (czar) {
							czar.is_czar = true;
							if (id == own_id) {
								toaster.info({ title: `You are the Czar!` });
							} else {
								toaster.info({ title: `${czar.name} is the Czar!` });
							}
						}
					}
					black_card = msg.data.black_card;
					break;
				case 'CardsSubmitted':
					let id = msg.data.player_id;
					let player_submit = lobby_state?.players[id];
					let placeholders = [];
					if (player_submit && black_card) {
						for (let i = 0; i < black_card.fields; i++) {
							placeholders.push({ text: `Card by ${player_submit.name}` });
						}
						revealed_cards.push(placeholders);
					}
					break;
				case 'UpdatePlayers':
					if (lobby_state) lobby_state.players = msg.data.players;
					break;
				case 'UpdateDecks':
					if (lobby_state) lobby_state.settings.decks = msg.data.decks;
					break;
				case 'UpdateSettings':
					if (lobby_state) lobby_state.settings = msg.data.settings;
					break;
				case 'RevealCards':
					set_phase('Judging');
					revealed_cards = msg.data.selected_cards;
					break;
				case 'RoundResult':
					set_phase('RoundFinished');
					round_result = msg.data;
					let winner = lobby_state?.players[round_result.player_id];
					if (winner) {
						winner.points += 1;
						if (round_result.player_id == own_id) {
							toaster.info({ title: `You are the winner of this round!` });
						} else {
							toaster.info({ title: `${winner.name} is the winner of this round!` });
						}
					}
					break;
				case 'RoundSkip':
					toaster.info({
						title: 'Skipped',
						description: 'The current round was skipped due to nobody selecting any card.'
					});
					break;
				case 'GameOver':
					set_phase('GameOver');
					break;
				case 'LobbyReset':
					set_phase('LobbyOpen');
					round = 0;
					break;
				case 'Kick':
					toaster.warning({
						title: 'Kicked',
						description: 'You have been removed from the lobby by the host.'
					});

					// close ws
					ws?.close();

					// remove remembered credentials
					logged_in = false;
					$own = null;
					own_name = '';
					own_id = crypto.randomUUID();

					// reconnect
					connect();

					break;
				case 'UpdateHand':
					cards = msg.data.cards;
					break;
				case 'Error':
					show_error(msg.data);
			}
		};

		ws.onclose = () => {
			lobby_state = undefined;
			connected = false;
		};
	}

	function join_lobby() {
		// make sure to reconnect if a error closed the connection
		if (!connected) connect();
		api.send_ws(ws!, { type: 'JoinLobby', data: { name: own_name, id: own_id } });
		$own = { lobby_id: lobby_id as api.Uuid, id: own_id, name: own_name };
	}

	function set_phase(phase: api.GamePhase) {
		if (lobby_state) lobby_state.phase = phase;

		let change: number | undefined;
		if (phase == 'RoundFinished') change = lobby_state?.settings.wait_time_secs;
		if (phase == 'Submitting') change = lobby_state?.settings.max_submitting_time_secs;
		if (phase == 'Judging') change = lobby_state?.settings.max_judging_time_secs;

		time = { self: change };
	}
</script>

<svelte:head>
	<title>Playing In a Lobby - Cards (Ludicrous ones) Against Humanity</title>
	<meta name="description" content="You're currently playing in a lobby." />
</svelte:head>

{#if isJoining}
	<Joining bind:own_name {join_lobby} />
{:else if isOpen}
	<LobbyOpen {ws} {lobby_state} {lobby_id} {own_id} />
{:else if isGaming}
	{@const isCzar = lobby_state?.players[own_id]?.is_czar || false}
	<div class="mx-auto flex max-w-7xl flex-col items-center space-y-6 px-4 py-8">
		{#key time}
			<TopBar {lobby_state} {own_id} {round} winner={round_result?.player_id} time={time.self} />
		{/key}

		{#if black_card}
			<BlackCard
				{ws}
				selectable={isCzar && isJudging}
				{black_card}
				{revealed_cards}
				selectedIndex={round_result?.winning_card_index}
			/>

			<Hand
				{ws}
				{black_card}
				bind:selectedIndexes={self_selected_cards}
				selectable={!isCzar && isSubmitting}
				disabled={isCzar}
				{cards}
			/>
		{/if}
	</div>
{:else if isOver}
	<GameOver {ws} {lobby_state} {own_id} />
{:else}
	<div class="mx-auto flex max-w-3xl flex-col items-center space-y-6 px-4 py-8">
		<p>Loading...</p>
		<a href="/" class="btn preset-filled-secondary-500">Close Connection</a>
	</div>
{/if}
