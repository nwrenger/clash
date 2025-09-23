<script lang="ts">
	import api from '$lib/api';
	import { areObjectsEqual, deepClone, relativeTime } from '$lib/utils';
	import { Check, Download, LoaderCircle, ExternalLink } from 'lucide-svelte';
	import AddDeck from './AddDeck.svelte';
	import type { Connection, Lobby, Own } from './+page.svelte';
	import CahIcon from '$lib/components/CahIcon.svelte';
	import { Tween } from 'svelte/motion';
	import NumberInput from '$lib/components/NumberInput.svelte';
	import type { Shared } from './LobbyOpen.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';

	interface Props {
		connection: Connection;
		lobby: Lobby;
		own: Own;
		shared: Shared;
	}

	let { connection, lobby, own, shared = $bindable() }: Props = $props();

	let is_host = $derived(lobby!.players![own.credentials.id]?.is_host || false);

	let auto_save = $state({
		delay: 2_000,
		remaining: 0,
		active: false,
		interval: undefined as number | undefined,
		last_settings: undefined as api.Settings | undefined
	});

	let updating_decks = $state(false);
	let changable_settings: api.Settings | undefined = $state();
	let changes = $derived(
		!areObjectsEqual(changable_settings, lobby?.settings) && !!changable_settings?.max_players
	);
	let sorted_decks = $derived(
		changable_settings?.decks.toSorted((a, b) => a.meta.name.localeCompare(b.meta.name)) || []
	);
	$effect(() => {
		shared.saving = changes || updating_decks;
	});

	let card_count = Tween.of(getCardCount);

	$effect(() => {
		shared.valid_config =
			Object.keys(lobby.players || {}).length >= 2 &&
			card_count.target.blacks > 0 &&
			card_count.target.whites > 0;
	});

	function getCardCount() {
		let enabled_decks = changable_settings?.decks.filter((d) => d.enabled) || [];

		let blacks = 0;
		let whites = 0;
		for (const deck of enabled_decks) {
			blacks += deck.meta.blacks_count;
			whites += deck.meta.whites_count;
		}

		return { blacks, whites };
	}

	const defaultSeconds = 90;
	const secondsOptions = [5, 10, 15, 30, 45, 60, 75, 90, 105, 120, 135, 150];

	function onSubmittingTypeChange(e: Event) {
		if (!changable_settings) return;
		const v = (e.target as HTMLSelectElement).value; // "" | "Constant" | "Player"
		if (v === '') {
			changable_settings.max_submitting_time_secs = null;
		} else {
			const currentSecs = changable_settings.max_submitting_time_secs?.seconds ?? defaultSeconds;
			changable_settings.max_submitting_time_secs = {
				type: v as 'Constant' | 'Player',
				seconds: currentSecs
			};
		}
	}

	function onSubmittingSecondsChange(e: Event) {
		if (!changable_settings) return;
		const v = (e.target as HTMLSelectElement).value; // string number
		const cur = changable_settings.max_submitting_time_secs;
		if (!cur) return; // seconds only applies if a type is chosen
		changable_settings.max_submitting_time_secs = {
			type: cur.type,
			seconds: +v
		};
	}

	$effect(() => {
		if (lobby?.settings?.decks) updating_decks = false;
	});
	$effect(() => {
		changable_settings = deepClone(lobby?.settings);
	});
	$effect(applySave);

	function applySave() {
		if (!is_host || !changes) {
			clearAutoSave();
			auto_save.last_settings = undefined;
			return;
		}

		if (!areObjectsEqual(changable_settings, auto_save.last_settings)) {
			auto_save.last_settings = deepClone(changable_settings);

			clearAutoSave();

			auto_save.remaining = auto_save.delay;
			auto_save.active = true;

			// Setup interval, used instead of a timeout for display
			auto_save.interval = setInterval(() => {
				auto_save.remaining -= 100;
				if (auto_save.remaining <= 0) {
					update_settings();
					clearAutoSave();
				}
			}, 100);
		}
	}

	function clearAutoSave() {
		if (auto_save.interval) clearInterval(auto_save.interval);
		auto_save.active = false;
		auto_save.remaining = 0;
	}

	function get_decks() {
		api.send_ws(connection.ws!, { type: 'FetchDecks' });
		updating_decks = true;
	}

	function update_settings() {
		if (changable_settings)
			api.send_ws(connection.ws!, {
				type: 'UpdateSettings',
				data: { settings: changable_settings }
			});
	}
</script>

<div class="h-full space-y-6 overflow-y-auto pt-4">
	{#if changable_settings}
		<div class="space-y-3 px-2">
			<div class="label">
				<span class="label-text flex items-center justify-between">
					<span class="flex items-center">
						<span>Decks</span>
						<Tooltip description="Enable the card sets you want to include in the game" />
					</span>
					<span class="text-error-500 flex items-center">
						<span>User Generated Content</span>
						<Tooltip
							preset="preset-filled-error-500"
							description="These decks are created and shared by other users; I do not review or endorse them"
						/>
					</span>
				</span>

				<!-- deck list -->
				<div class="grid grid-cols-1 gap-1 sm:grid-cols-2">
					{#each sorted_decks as deck (deck.meta.deckcode)}
						<div class="preset-tonal rounded-base">
							<label
								class="rounded-base {is_host
									? 'hover:preset-tonal cursor-pointer'
									: ''} flex items-start gap-3 p-3 transition select-none"
							>
								<input
									disabled={!is_host}
									class="checkbox checkbox-sm mt-[2px]"
									type="checkbox"
									checked={deck.enabled}
									onchange={() => (deck.enabled = !deck.enabled)}
									title="{deck.enabled ? 'Disable' : 'Enable'} {deck.meta.name}"
								/>

								<div class="min-w-0 flex-1">
									<div class="flex justify-between space-x-1.5">
										<span class="truncate" title={deck.meta.name}>{deck.meta.name}</span>
										<span class="flex space-x-1.5">
											{#if deck.meta.language && deck.meta.language != '-'}
												{@const upper = deck.meta.language.toUpperCase()}
												<span
													class="badge preset-filled-surface-300-700 px-1.5 py-0.5 text-[10px] font-bold"
													title="Language: {upper}"
												>
													{upper}
												</span>
											{/if}
											{#if deck.meta.nsfw}
												<span
													class="badge preset-filled-error-500 px-1.5 py-0.5 text-[10px] font-bold"
													title="Not Safe For Work"
												>
													NSFW
												</span>
											{/if}
										</span>
									</div>

									<div
										class="text-surface-800-200 mt-0.5 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs"
									>
										<a
											class="anchor flex w-fit items-center text-xs"
											href={'https://cast.clrtd.com/deck/' + deck.meta.deckcode}
											target="_blank"
											rel="noreferrer"
											onclick={(e) => e.stopPropagation()}
											title="Open deck in browser"
										>
											<span>{deck.meta.deckcode}</span>
											<ExternalLink size={18} class="pb-[2px] pl-1 opacity-60" />
										</a>
										<span title={new Date(deck.meta.fetched_at * 1000).toLocaleString()}>
											Updated {relativeTime(deck.meta.fetched_at)}
										</span>
									</div>
								</div>
							</label>
						</div>
					{/each}
				</div>

				<span
					class="badge preset-filled-surface-300-700"
					title="Blacks: {card_count.target.blacks}"
				>
					<CahIcon class="h-6" />
					{Math.round(card_count.current.blacks)}
				</span>
				<span
					class="badge preset-filled-surface-300-700"
					title="Whites: {card_count.target.whites}"
				>
					<CahIcon class="h-6" fill="#fff" />
					{Math.round(card_count.current.whites)}
				</span>
			</div>

			<div class="w-full space-y-2">
				{#if is_host}
					<div class="label">
						<span class="label-text flex items-center">
							<span>Manage Decks</span>
							<Tooltip description="Update or Add card sets made by the community" />
						</span>
						<div class="grid gap-1.5 sm:grid-cols-2">
							<button
								class="preset-filled-primary-500 btn"
								title="Update all Decks"
								onclick={get_decks}
								disabled={shared.saving}
							>
								{#if updating_decks}
									<LoaderCircle size={20} class="animate-spin" /> Updating...
								{:else}
									<Download size={20} /> Update All
								{/if}
							</button>
							<AddDeck {connection} disabled={shared.saving} />
						</div>
					</div>
				{/if}
			</div>

			<div class="w/full grid space-y-3 sm:grid-cols-2 sm:gap-1.5 sm:space-y-0">
				<label class="label">
					<span class="label-text flex items-center">
						<span>Max Rounds</span>
						<Tooltip description="The game ends once the round count reaches this maximum" />
					</span>

					<select class="select" bind:value={changable_settings.max_rounds} disabled={!is_host}>
						<option value={null}>None</option>
						{#each Array.from({ length: 10 }) as _, i}
							{@const round = (i + 1) * 5}
							<option value={round}>{round}</option>
						{/each}
					</select>
				</label>

				<label class="label">
					<span class="label-text flex items-center">
						<span>Max Points</span>
						<Tooltip description="The game ends once a player's points reach this maximum" />
					</span>

					<select class="select" bind:value={changable_settings.max_points} disabled={!is_host}>
						<option value={null}>None</option>
						{#each Array.from({ length: 10 }) as _, i}
							{@const points = (i + 1) * 2}
							<option value={points}>{points}</option>
						{/each}
					</select>
				</label>
			</div>

			<div class="grid w-full space-y-3 sm:grid-cols-2 sm:gap-1.5 sm:space-y-0">
				<label class="label">
					<span class="label-text flex items-center">
						<span>Max Players</span>
						<Tooltip description="The maximum number of players allowed to join this lobby" />
					</span>

					<NumberInput
						placeholder="Input max players..."
						value={changable_settings.max_players}
						disabled={!is_host}
						update={(value) => {
							if (changable_settings && value) changable_settings.max_players = value;
						}}
					/>
				</label>

				<label class="label">
					<span class="label-text flex items-center">
						<span>Wait Time</span>
						<Tooltip
							description="The time between each round; avoid setting this too high or it may become annoying"
						/>
					</span>

					<select class="select" bind:value={changable_settings.wait_time_secs} disabled={!is_host}>
						<option value={null}>None</option>
						{#each [5, 10, 15, 20] as s}
							<option value={s}>{s}s</option>
						{/each}
					</select>
				</label>
			</div>

			<div
				class="grid w-full space-y-3 sm:grid-cols-2 sm:gap-1.5 sm:space-y-0 {!is_host
					? 'mb-8'
					: ''}"
			>
				<label class="label">
					<span class="label-text flex items-center">
						<span>Max Submitting Time</span>
						<Tooltip
							description="The maximum time players have to submit their cards; can be set to a fixed duration or scale with the number of players"
						/>
					</span>

					<div class="input-group grid-cols-[auto_1fr_auto] {!is_host ? 'opacity-50' : ''}">
						<!-- Type -->
						<select
							class="ig-select {!is_host ? 'opacity-100!' : ''}"
							disabled={!is_host}
							value={changable_settings?.max_submitting_time_secs?.type ?? ''}
							onchange={onSubmittingTypeChange}
						>
							<option value="">None</option>
							<option value="Constant">Constant</option>
							<option value="Player">Each Player &nbsp;</option>
						</select>

						<!-- Seconds -->
						<select
							class="ig-select {!is_host ? 'opacity-100!' : ''}"
							disabled={!is_host || !changable_settings?.max_submitting_time_secs}
							value={String(
								changable_settings?.max_submitting_time_secs?.seconds ?? defaultSeconds
							)}
							onchange={onSubmittingSecondsChange}
						>
							{#each secondsOptions as seconds}
								<option value={String(seconds)}>{seconds}s</option>
							{/each}
						</select>
					</div>
				</label>

				<label class="label">
					<span class="label-text flex items-center">
						<span>Max Judging Time</span>
						<Tooltip description="The maximum time the czar has to pick a winning card" />
					</span>

					<select
						class="select"
						bind:value={changable_settings.max_judging_time_secs}
						disabled={!is_host}
					>
						<option value={null}>None</option>
						{#each [15, 30, 45, 60, 75, 90, 105, 120] as s}
							<option value={s}>{s}s</option>
						{/each}
					</select>
				</label>
			</div>

			{#if is_host}
				<div class="sticky bottom-0 z-50 mb-8 flex w-full justify-center">
					<span class="badge preset-tonal backdrop-blur-lg">
						{#if auto_save.active}
							<LoaderCircle class="animate-spin" size={16} />
							Applying in {(auto_save.remaining / 1000).toFixed(1)}s...
						{:else if changes}
							<Download size={16} />
							Saving...
						{:else}
							<Check size={16} />
							Up to Date
						{/if}
					</span>
				</div>
			{/if}
		</div>
	{/if}
</div>
