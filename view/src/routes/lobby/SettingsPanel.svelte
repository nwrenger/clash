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
	import InfoTooltip from '$lib/components/InfoTooltip.svelte';

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
			blacks += deck.blacks_count;
			whites += deck.whites_count;
		}

		return { blacks, whites };
	}

	// ---------- NEW: string UI state for all selects ----------
	const defaultSeconds = 90;
	const secondsOptions = [5, 10, 15, 30, 45, 60, 75, 90, 105, 120, 135, 150];

	let maxRoundsStr = $state('');
	let maxPointsStr = $state('');
	let waitTimeStr = $state('');
	let maxJudgingStr = $state('');

	let submittingTypeStr = $state(''); // "" | "Constant" | "Player"
	let submittingSecondsStr = $state(String(defaultSeconds)); // string number

	// Reflect server state -> string UI state
	$effect(() => {
		const s = changable_settings;
		maxRoundsStr = s?.max_rounds != null ? String(s.max_rounds) : '';
		maxPointsStr = s?.max_points != null ? String(s.max_points) : '';
		waitTimeStr = s?.wait_time_secs != null ? String(s.wait_time_secs) : '';
		maxJudgingStr = s?.max_judging_time_secs != null ? String(s.max_judging_time_secs) : '';

		const sub = s?.max_submitting_time_secs ?? null;
		submittingTypeStr = sub?.type ?? '';
		submittingSecondsStr = String(sub?.seconds ?? defaultSeconds);
	});

	// Handlers: convert to number|null and write to changable_settings
	function onMaxRoundsChange(e: Event) {
		const v = (e.target as HTMLSelectElement).value;
		maxRoundsStr = v;
		if (!changable_settings) return;
		changable_settings.max_rounds = v ? +v : null;
	}
	function onMaxPointsChange(e: Event) {
		const v = (e.target as HTMLSelectElement).value;
		maxPointsStr = v;
		if (!changable_settings) return;
		changable_settings.max_points = v ? +v : null;
	}
	function onWaitTimeChange(e: Event) {
		const v = (e.target as HTMLSelectElement).value;
		waitTimeStr = v;
		if (!changable_settings) return;
		changable_settings.wait_time_secs = v ? +v : null;
	}
	function onMaxJudgingChange(e: Event) {
		const v = (e.target as HTMLSelectElement).value;
		maxJudgingStr = v;
		if (!changable_settings) return;
		changable_settings.max_judging_time_secs = v ? +v : null;
	}

	function onSubmittingTypeChange(e: Event) {
		const v = (e.target as HTMLSelectElement).value; // "" | "Constant" | "Player"
		submittingTypeStr = v;
		if (!changable_settings) return;

		if (v === '') {
			changable_settings.max_submitting_time_secs = null;
		} else {
			const secs = Number.isFinite(+submittingSecondsStr) ? +submittingSecondsStr : defaultSeconds;
			changable_settings.max_submitting_time_secs = {
				type: v as 'Constant' | 'Player',
				seconds: secs
			};
		}
	}
	function onSubmittingSecondsChange(e: Event) {
		const v = (e.target as HTMLSelectElement).value; // string number
		submittingSecondsStr = v;
		if (!changable_settings) return;
		if (submittingTypeStr === '') return;

		changable_settings.max_submitting_time_secs = {
			type: submittingTypeStr as 'Constant' | 'Player',
			seconds: +v
		};
	}
	// ----------------------------------------------------------

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
				<span class="label-text flex items-center justify-start">
					<span> Decks </span>
					<InfoTooltip description="Enable the card sets you want to include in the game" />
				</span>

				<!-- deck list -->
				<div class="grid grid-cols-1 gap-1 sm:grid-cols-2">
					{#each changable_settings.decks as deck (deck.deckcode)}
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
									title="{deck.enabled ? 'Disable' : 'Enable'} {deck.name}"
								/>

								<div class="min-w-0 flex-1">
									<div class="flex">
										<span class="truncate" title={deck.name}>{deck.name}</span>
									</div>

									<div
										class="text-surface-800-200 mt-0.5 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs"
									>
										<a
											class="anchor flex w-fit items-center text-xs"
											href={'https://cast.clrtd.com/deck/' + deck.deckcode}
											target="_blank"
											rel="noreferrer"
											onclick={(e) => e.stopPropagation()}
											title="Open deck in browser"
										>
											<span>{deck.deckcode}</span>
											<ExternalLink size={18} class="pb-[2px] pl-1 opacity-60" />
										</a>
										<span title={new Date(deck.fetched_at * 1000).toLocaleString()}>
											Updated {relativeTime(deck.fetched_at)}
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
						<span class="label-text flex items-center justify-start">
							<span>Manage Decks</span>
							<InfoTooltip description="Update or Add card sets made by the community" />
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
					<span class="label-text flex items-center justify-start">
						<span>Max Rounds</span>
						<InfoTooltip description="The game ends once the round count reaches this maximum" />
					</span>

					<!-- CHANGED -->
					<select
						class="select"
						bind:value={maxRoundsStr}
						onchange={onMaxRoundsChange}
						disabled={!is_host}
					>
						<option value="">None</option>
						{#each Array.from({ length: 10 }) as _, i}
							{@const round = (i + 1) * 5}
							<option value={String(round)}>{round}</option>
						{/each}
					</select>
				</label>

				<label class="label">
					<span class="label-text flex items-center justify-start">
						<span>Max Points</span>
						<InfoTooltip description="The game ends once a player's points reach this maximum" />
					</span>

					<select
						class="select"
						bind:value={maxPointsStr}
						onchange={onMaxPointsChange}
						disabled={!is_host}
					>
						<option value="">None</option>
						{#each Array.from({ length: 10 }) as _, i}
							{@const points = (i + 1) * 2}
							<option value={String(points)}>{points}</option>
						{/each}
					</select>
				</label>
			</div>

			<div class="grid w-full space-y-3 sm:grid-cols-2 sm:gap-1.5 sm:space-y-0">
				<label class="label">
					<span class="label-text flex items-center justify-start">
						<span>Max Players</span>
						<InfoTooltip description="The maximum number of players allowed to join this lobby" />
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
					<span class="label-text flex items-center justify-start">
						<span>Wait Time</span>
						<InfoTooltip
							description="The time between each round; avoid setting this too high or it may become annoying"
						/>
					</span>

					<select
						class="select"
						bind:value={waitTimeStr}
						onchange={onWaitTimeChange}
						disabled={!is_host}
					>
						<option value="">None</option>
						{#each [5, 10, 15, 20] as s}
							<option value={String(s)}>{s}s</option>
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
					<span class="label-text flex items-center justify-start">
						<span>Max Submitting Time</span>
						<InfoTooltip
							description="The maximum time players have to submit their cards; can be set to a fixed duration or scale with the number of players"
						/>
					</span>

					<div class="input-group grid-cols-[auto_1fr_auto] {!is_host ? 'opacity-50' : ''}">
						<!-- CHANGED -->
						<select
							class="ig-select opacity-100!"
							bind:value={submittingTypeStr}
							onchange={onSubmittingTypeChange}
							disabled={!is_host}
						>
							<option value="">None</option>
							<option value="Constant">Constant</option>
							<option value="Player">Each Player &nbsp;</option>
						</select>

						<!-- CHANGED -->
						<select
							class="ig-select opacity-100!"
							bind:value={submittingSecondsStr}
							onchange={onSubmittingSecondsChange}
							disabled={!is_host || submittingTypeStr === ''}
						>
							{#each secondsOptions as seconds}
								<option value={String(seconds)}>{seconds}s</option>
							{/each}
						</select>
					</div>
				</label>

				<label class="label">
					<span class="label-text flex items-center justify-start">
						<span>Max Judging Time</span>
						<InfoTooltip description="The maximum time the czar has to pick a winning card" />
					</span>

					<select
						class="select"
						bind:value={maxJudgingStr}
						onchange={onMaxJudgingChange}
						disabled={!is_host}
					>
						<option value="">None</option>
						{#each [15, 30, 45, 60, 75, 90, 105, 120] as s}
							<option value={String(s)}>{s}s</option>
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
