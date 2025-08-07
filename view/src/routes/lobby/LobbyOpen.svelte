<script lang="ts">
	import { page } from '$app/state';
	import api from '$lib/api';
	import CopyButton from '$lib/components/CopyButton.svelte';
	import { areObjectsEqual, deepClone, sortedEntries } from '$lib/utils';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import { Crown, Eye, EyeOff, Play, Settings2 } from 'lucide-svelte';
	import AddDeck from './AddDeck.svelte';
	import type { Connection, Lobby, Own } from './+page.svelte';

	interface Props {
		connection: Connection;
		lobby: Lobby;
		own: Own;
	}

	let tabs = $state('lobby');
	let hide_url = $state(true);
	let changable_settings: api.Settings | undefined = $state();
	$effect(() => {
		changable_settings = deepClone(lobby.state?.settings);
	});

	let { connection, lobby, own }: Props = $props();

	let isHost = $derived(lobby.state?.players[own.id]?.is_host || false);
	let lobbyUrl = $derived(`${page.url.origin}/lobby?id=${lobby.id}`);
	let applyable = $derived(
		!areObjectsEqual(changable_settings, lobby.state?.settings) && changable_settings?.max_players
	);

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
		api.send_ws(connection.ws!, { type: 'GetDecks' });
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
				<div class="w-8 sm:w-full">
					<div class="truncate">Lobby</div>
				</div>
			</Tabs.Control>
			<Tabs.Control value="settings" labelBase="btn hover:filter-none!">
				{#snippet lead()}<Settings2 size="18" />{/snippet}
				<div class="w-8 sm:w-full">
					<div class="truncate">Settings</div>
				</div>
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
						<input class="ig-input {hide_url ? 'blur-sm' : ''}" value={lobbyUrl} readonly />
						<CopyButton class="ig-btn preset-filled-secondary-500" id="cp-button" text={lobbyUrl}>
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
						{#if isHost}
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

				{#if isHost}
					<hr class="hr" />
					<div class="flex w-full items-center justify-center space-x-2">
						<button
							class="btn preset-filled-primary-500 flex w-full items-center justify-center sm:w-auto"
							onclick={start_game}>Start Game</button
						>
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
											disabled={!isHost}
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
										</span>
									</label>
								{/each}
							</div>

							{#if isHost}
								<div class="label">
									<span class="label-text">Manage Decks</span>
									<div class="grid gap-1.5 sm:grid-cols-2">
										<button
											class="preset-filled-primary-500 btn"
											title="Update Decks"
											onclick={get_decks}>Update</button
										>
										<AddDeck {connection} />
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
									disabled={!isHost}
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
									disabled={!isHost}
								/>
							</label>
						</div>
						<div class="w-full">
							<label class="label">
								<span class="label-text">Wait Time</span>

								<select
									class="select"
									bind:value={changable_settings.wait_time_secs}
									disabled={!isHost}
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
									disabled={!isHost}
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
									disabled={!isHost}
								>
									<option value={null}>None</option>
									{#each [15, 30, 45, 60, 75, 90, 105, 120] as s}
										<option value={s}>{s}s</option>
									{/each}
								</select>
							</label>
						</div>
					</div>

					{#if isHost}
						<div class="grid gap-1.5 sm:grid-cols-2">
							<button
								class="btn preset-filled-error-500"
								disabled={!applyable}
								onclick={() => (changable_settings = deepClone(lobby.state?.settings))}
								>Reset</button
							>
							<button
								class="btn preset-filled-success-500"
								disabled={!applyable}
								onclick={update_settings}>Apply</button
							>
						</div>
					{/if}
				{/if}
			</Tabs.Panel>
		{/snippet}
	</Tabs>
</div>
