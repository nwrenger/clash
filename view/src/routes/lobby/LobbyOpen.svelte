<script lang="ts">
	import { page } from '$app/state';
	import api from '$lib/api';
	import ShareButton from '$lib/components/ShareButton.svelte';
	import { areObjectsEqual, colorFromUUID, deepClone, sortedEntries } from '$lib/utils';
	import { Avatar, Tabs } from '@skeletonlabs/skeleton-svelte';
	import {
		Ban,
		Check,
		UserRound,
		ClipboardCopy,
		Crown,
		Download,
		House,
		Link,
		LoaderCircle,
		Play,
		Settings2,
		User
	} from 'lucide-svelte';
	import AddDeck from './AddDeck.svelte';
	import type { Connection, Lobby, Own } from './+page.svelte';

	interface Props {
		connection: Connection;
		lobby: Lobby;
		own: Own;
	}

	let { connection, lobby, own }: Props = $props();

	let tabs = $state('lobby');

	let is_host = $derived(lobby!.players![own.id]?.is_host || false);
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
		!areObjectsEqual(changable_settings, lobby?.settings) && !!changable_settings?.max_players
	);
	let saving = $derived(changes || updating_decks);

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

<div class="mx-auto flex h-full max-w-3xl flex-col items-center px-4 py-8">
	<Tabs
		value={tabs}
		onValueChange={(e) => (tabs = e.value)}
		fluid
		composite
		classes="h-full"
		contentClasses="h-[calc(100%-53px)]"
		listClasses="preset-tonal backdrop-blur-lg pt-2 px-2 !mb-0 rounded-md whitespace-nowrap"
	>
		{#snippet list()}
			<Tabs.Control value="lobby" labelBase="btn hover:filter-none!">
				{#snippet lead()}<House size="18" />{/snippet}
				<span>Lobby</span>
			</Tabs.Control>
			<Tabs.Control value="settings" labelBase="btn hover:filter-none!">
				{#snippet lead()}<Settings2 size="18" />{/snippet}
				<span>Settings</span>
			</Tabs.Control>
		{/snippet}
		{#snippet content()}
			<Tabs.Panel classes="h-full" value="lobby">
				<div class="h-full overflow-y-auto">
					<div class="min-h-0 space-y-2 px-2 py-4">
						{#each sortedEntries(lobby?.players) as [id, player]}
							<div class="preset-filled grid w-full grid-cols-[1fr_auto] rounded-lg px-5 py-3">
								<div
									class="flex w-full max-w-full min-w-0 flex-1 items-center justify-start space-x-1.5 {id ===
									own.id
										? 'text-primary-500'
										: ''}"
								>
									<div
										class="w-7 rounded-sm p-1"
										style="background-color: {colorFromUUID(id).background};"
									>
										{#if player.is_host}
											<Crown color={colorFromUUID(id).text} size={20} />
										{:else}
											<UserRound color={colorFromUUID(id).text} size={20} />
										{/if}
									</div>
									<span class="truncate text-xl font-semibold" title={player.name}
										>{player.name}</span
									>
								</div>

								{#if id !== own.id && is_host}
									<button
										class="btn-icon text-error-500 h-full w-fit p-0"
										title="Kick Player?"
										onclick={() => kick(id)}
									>
										<Ban size={20} />
									</button>
								{/if}
							</div>
						{/each}
					</div>

					<div class="sticky bottom-0 z-50 flex w-full flex-col items-center justify-center">
						<div
							class="preset-tonal grid w-full gap-1.5 rounded-md p-2 backdrop-blur-lg {is_host
								? 'sm:grid-cols-2'
								: 'sm:w-auto'}"
						>
							<ShareButton class="btn preset-filled-primary-500 h-fit w-full" url={lobby_url}>
								{#snippet child({ copied })}
									{#if copied}
										<ClipboardCopy size={22} />
										Copied Invite
									{:else}
										<Link size={22} />
										Invite Players
									{/if}
								{/snippet}
							</ShareButton>

							{#if is_host}
								<div class="flex w-full flex-col space-y-2">
									<button
										class="btn preset-filled-primary-500"
										onclick={start_game}
										disabled={saving}
									>
										<Play size={22} />
										Start Game
									</button>

									{#if saving}
										<div class="flex items-center justify-center gap-1 text-xs">
											<LoaderCircle class="animate-spin" size={16} />
											<span>Saving settings...</span>
										</div>
									{/if}
								</div>
							{/if}
						</div>
					</div>
				</div>
			</Tabs.Panel>
			<Tabs.Panel classes="h-full space-y-6 overflow-y-auto pt-4" value="settings">
				{#if changable_settings}
					<div class="space-y-3 px-2">
						<div class="w-full space-y-2">
							<div class="label">
								<span class="label-text">Decks</span>
								{#each changable_settings.decks as deck}
									<label class="flex items-center space-x-2">
										<input
											disabled={!is_host}
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

							{#if is_host}
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

								<select
									class="select"
									bind:value={changable_settings.max_rounds}
									disabled={!is_host}
								>
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
									disabled={!is_host}
								/>
							</label>
						</div>
						<div class="w-full">
							<label class="label">
								<span class="label-text">Wait Time</span>

								<select
									class="select"
									bind:value={changable_settings.wait_time_secs}
									disabled={!is_host}
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
									disabled={!is_host}
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
							<div class="sticky bottom-0 z-50 flex w-full justify-center">
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
			</Tabs.Panel>
		{/snippet}
	</Tabs>
</div>
