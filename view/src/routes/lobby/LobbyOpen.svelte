<script lang="ts">
	import { page } from '$app/state';
	import api from '$lib/api';
	import CopyButton from '$lib/components/CopyButton.svelte';
	import { areObjectsEqual, deepClone, sortedEntries } from '$lib/utils';
	import { Tabs, Tooltip } from '@skeletonlabs/skeleton-svelte';
	import { Crown, Download, Eye, EyeOff, LoaderCircle, Play, Settings2 } from 'lucide-svelte';
	import AddDeck from './AddDeck.svelte';
	import type { Connection, Lobby, Own } from './+page.svelte';

	interface Props {
		connection: Connection;
		lobby: Lobby;
		own: Own;
	}

	let { connection, lobby, own }: Props = $props();

	let tabs = $state('lobby');

	let host = $derived(lobby.state?.players[own.id]?.is_host || false);
	let hide_url = $state(true);
	let lobby_url = $derived(`${page.url.origin}/lobby?id=${lobby.id}`);

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
		!areObjectsEqual(changable_settings, lobby.state?.settings) && !!changable_settings?.max_players
	);
	let saving = $derived(changes || updating_decks);

	$effect(() => {
		if (lobby.state?.settings.decks) updating_decks = false;
	});
	$effect(() => {
		changable_settings = deepClone(lobby.state?.settings);
	});
	$effect(applySave);

	function applySave() {
		if (!host || !changes) {
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

	function handleMaxPlayers(e: Event) {
		let target = e.target as HTMLInputElement;
		// Only allow integers and remove leading zeros
		let digits: string = target.value.replace(/[^0-9]/g, '').replace(/^0+/, '');
		// Cap at Rust u32 max
		if (digits.length > 10) {
			digits = digits.slice(0, 10);
		}
		const U32_MAX_STR = '4294967295';
		if (digits.length === 10 && digits > U32_MAX_STR) {
			digits = U32_MAX_STR;
		}
		// Update value and display
		if (changable_settings) changable_settings.max_players = parseInt(digits) || (undefined as any);
		target.value = digits;
	}

	function kick(player_id: api.Uuid) {
		if (own.id != player_id)
			api.send_ws(connection.ws!, { type: 'Kick', data: { kicked: player_id } });
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

	function start_game() {
		api.send_ws(connection.ws!, { type: 'StartRound' });
	}
</script>

<div class="mx-auto flex max-w-3xl flex-col items-center px-4 py-8">
	<Tabs
		value={tabs}
		onValueChange={(e) => (tabs = e.value)}
		fluid
		composite
		listClasses="preset-tonal pt-2 px-2 rounded-md whitespace-nowrap"
		contentClasses=""
		classes="h-full"
	>
		{#snippet list()}
			<Tabs.Control value="lobby" labelBase="btn hover:filter-none!">
				{#snippet lead()}<Play size="18" />{/snippet}
				<span>Lobby</span>
			</Tabs.Control>
			<Tabs.Control value="settings" labelBase="btn hover:filter-none!">
				{#snippet lead()}<Settings2 size="18" />{/snippet}
				<span>Settings</span>
			</Tabs.Control>
		{/snippet}
		{#snippet content()}
			<Tabs.Panel classes="h-full space-y-6" value="lobby">
				<div class="label">
					<span class="label-text">Lobby Url</span>

					<div class="input-group grid-cols-[auto_1fr_auto]">
						<button
							class="ig-btn preset-filled p-2"
							onclick={() => (hide_url = !hide_url)}
							title="Toggle Visibility"
						>
							{#if hide_url}
								<EyeOff size="20" />
							{:else}
								<Eye size="20" />
							{/if}
						</button>
						<input class="ig-input {hide_url ? 'blur-sm' : ''}" value={lobby_url} readonly />
						<CopyButton class="ig-btn preset-filled-secondary-500" id="cp-button" text={lobby_url}>
							{#snippet child({ copied })}
								{#if copied}
									Copied
								{:else}
									Copy
								{/if}
							{/snippet}
						</CopyButton>
					</div>
				</div>
				<hr class="hr" />
				<div class="mx-auto flex w-full max-w-(--breakpoint-xl) flex-wrap justify-center gap-2">
					{#each sortedEntries(lobby.state?.players) as [id, player]}
						{#if host}
							<button
								class="card relative {id === own.id
									? 'preset-filled-tertiary-500'
									: 'preset-filled'} {id !== own.id
									? 'hover:preset-filled-error-500'
									: 'pointer-events-none'} px-4 py-2 transition-colors"
								title="Kick Player?"
								onclick={() => kick(id)}
							>
								<span class="flex h-full w-full items-center justify-center">
									<span class="flex items-center space-x-1.5">
										{#if player.is_host}
											<Crown size={16} />
										{/if}
										<span>{player.name}</span>
									</span>
								</span>
							</button>
						{:else}
							<button
								class="card relative {id === own.id
									? 'preset-filled-tertiary-500'
									: 'preset-filled'} pointer-events-none px-4 py-2"
							>
								<span class="flex h-full w-full items-center justify-center">
									<span class="flex items-center space-x-1.5">
										{#if player.is_host}
											<Crown size={16} />
										{/if}
										<span>{player.name}</span>
									</span>
								</span>
							</button>
						{/if}
					{/each}
				</div>

				{#if host}
					<hr class="hr" />
					<div class="flex w-full items-center justify-center space-x-2">
						<Tooltip
							open={saving}
							positioning={{ placement: 'bottom' }}
							base="flex w-full sm:w-auto items-center justify-center"
							triggerBase="w-full sm:w-auto"
							contentBase="card preset-filled-warning-500 p-4 max-w-[calc(100vw-80px)] text-center"
							openDelay={200}
							closeDelay={200}
							arrowBackground="var(--color-warning-500)"
							arrow
						>
							{#snippet trigger()}
								<button
									class="btn preset-filled-primary-500 w-full sm:w-auto"
									onclick={start_game}
									disabled={saving}
								>
									Start Game
								</button>
							{/snippet}
							{#snippet content()}
								<span class="flex items-center gap-2">
									<LoaderCircle class="animate-spin" size={18} />
									<span>Saving settings...</span>
								</span>
							{/snippet}
						</Tooltip>
					</div>
				{/if}
			</Tabs.Panel>
			<Tabs.Panel classes="h-full space-y-6" value="settings">
				{#if changable_settings}
					<div class="space-y-3">
						<div class="w-full space-y-2">
							<div class="label">
								<span class="label-text">Decks</span>
								{#each changable_settings.decks as deck}
									<label class="flex items-center space-x-2">
										<input
											disabled={!host}
											class="checkbox"
											type="checkbox"
											checked={deck.enabled}
											onclick={() => (deck.enabled = !deck.enabled)}
										/>
										<span>
											<span>{deck.name}</span>
											<a
												class="anchor text-xs"
												href="https://cast.clrtd.com/deck/{deck.deckcode}"
												target="_blank">{deck.deckcode}</a
											>
											<span class="text-surface-800-200 text-xs">
												{new Date(deck.fetched_at * 1000).toLocaleString()}
											</span>
										</span>
									</label>
								{/each}
							</div>

							{#if host}
								<div class="label">
									<span class="label-text">Manage Decks</span>
									<div class="grid gap-1.5 sm:grid-cols-2">
										<button
											class="preset-filled-primary-500 btn"
											title="Update all Decks"
											onclick={get_decks}
											disabled={saving}
										>
											{#if updating_decks}
												<LoaderCircle class="animate-spin" />
												Updating...
											{:else}
												Update All
											{/if}
										</button>
										<AddDeck {connection} disabled={saving} />
									</div>
								</div>
							{/if}
						</div>
						<div class="grid w-full grid-cols-2 gap-1.5">
							<label class="label">
								<span class="label-text">Max Rounds</span>

								<select class="select" bind:value={changable_settings.max_rounds} disabled={!host}>
									{#each Array.from({ length: 69 }) as _, i}
										{@const round = i + 1}
										<option value={round}>{round}</option>
									{/each}
								</select>
							</label>
							<label class="label">
								<span class="label-text">Max Players</span>

								<input
									class="input"
									type="text"
									inputmode="numeric"
									autocomplete="off"
									placeholder="Input max players..."
									oninput={handleMaxPlayers}
									value={changable_settings.max_players}
									disabled={!host}
								/>
							</label>
						</div>
						<div class="w-full">
							<label class="label">
								<span class="label-text">Wait Time</span>

								<select
									class="select"
									bind:value={changable_settings.wait_time_secs}
									disabled={!host}
								>
									<option value={null}>None</option>
									{#each [5, 10, 15, 20] as s}
										<option value={s}>{s}s</option>
									{/each}
								</select>
							</label>
						</div>
						<div class="grid w-full grid-cols-2 gap-1.5">
							<label class="label">
								<span class="label-text">Max Submitting Time</span>

								<select
									class="select"
									bind:value={changable_settings.max_submitting_time_secs}
									disabled={!host}
								>
									<option value={null}>None</option>
									{#each [15, 30, 45, 60, 75, 90, 105, 120] as s}
										<option value={s}>{s}s</option>
									{/each}
								</select>
							</label>
							<label class="label">
								<span class="label-text">Max Judging Time</span>

								<select
									class="select"
									bind:value={changable_settings.max_judging_time_secs}
									disabled={!host}
								>
									<option value={null}>None</option>
									{#each [15, 30, 45, 60, 75, 90, 105, 120] as s}
										<option value={s}>{s}s</option>
									{/each}
								</select>
							</label>
						</div>
						{#if auto_save.active}
							<div class="text-primary-500 flex items-center gap-1 text-xs">
								<LoaderCircle class="animate-spin" size={16} />
								Applying in {(auto_save.remaining / 1000).toFixed(1)}s...
							</div>
						{:else if changes}
							<div class="text-primary-500 flex items-center gap-1 text-xs">
								<Download size={16} />
								Saving...
							</div>
						{/if}
					</div>
				{/if}
			</Tabs.Panel>
		{/snippet}
	</Tabs>
</div>
