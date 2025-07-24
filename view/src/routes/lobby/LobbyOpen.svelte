<script lang="ts">
	import { page } from '$app/state';
	import api from '$lib/api';
	import CopyButton from '$lib/components/CopyButton.svelte';
	import { areObjectsEqual, deepClone, sortedEntries } from '$lib/utils';
	import { Switch, Tabs } from '@skeletonlabs/skeleton-svelte';
	import {
		ClipboardCheck,
		ClipboardCopy,
		Crown,
		Eye,
		EyeOff,
		Play,
		Settings2
	} from 'lucide-svelte';
	import AddDeck from './AddDeck.svelte';

	interface Props {
		ws: WebSocket | undefined;
		lobby_state: api.LobbyState | undefined;
		lobby_id: string;
		own_id: api.Uuid;
	}

	let tabs = $state('lobby');
	let hide_url = $state(true);
	let changable_settings: api.Settings | undefined = $state();
	$effect(() => {
		changable_settings = deepClone(lobby_state?.settings);
	});

	let { ws, lobby_state, lobby_id, own_id }: Props = $props();

	let isHost = $derived(lobby_state?.players[own_id]?.is_host || false);
	let lobbyUrl = $derived(`${page.url.origin}/lobby?id=${lobby_id}`);

	function kick(own_id: api.Uuid, player_id: api.Uuid) {
		if (own_id != player_id) api.send_ws(ws!, { type: 'Kick', data: { kicked: player_id } });
	}

	function update_settings() {
		if (changable_settings)
			api.send_ws(ws!, { type: 'UpdateSettings', data: { settings: changable_settings } });
	}

	function start_game() {
		api.send_ws(ws!, { type: 'StartRound' });
	}
</script>

<div class="mx-auto flex max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	<Tabs
		value={tabs}
		onValueChange={(e) => (tabs = e.value)}
		fluid
		composite
		listClasses="preset-tonal pt-2 px-2 rounded-md whitespace-nowrap"
		contentClasses="h-[calc(100%-61px)]"
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
				<label class="label">
					<span class="label-text">Lobby Url</span>

					<div class="grid grid-cols-[auto_1fr_auto] gap-2">
						<Switch
							checked={hide_url}
							onCheckedChange={(e) => (hide_url = e.checked)}
							name="compact"
							controlWidth="w-9"
							controlActive="preset-filled"
							compact
						>
							{#snippet inactiveChild()}<Eye size="20" />{/snippet}
							{#snippet activeChild()}<EyeOff size="20" />{/snippet}
						</Switch>
						<input class="input {hide_url ? 'blur-sm' : ''}" value={lobbyUrl} readonly />
						<CopyButton class="btn preset-filled-secondary-500" text={lobbyUrl}>
							{#snippet child({ copied })}
								{#if copied}
									Copied
								{:else}
									Copy
								{/if}
							{/snippet}
						</CopyButton>
					</div>
				</label>
				<hr class="hr" />
				<div class="mx-auto flex w-full max-w-(--breakpoint-xl) flex-wrap justify-center gap-2">
					{#each sortedEntries(lobby_state?.players) as [id, player]}
						{#if isHost}
							<button
								class="card relative {id === own_id
									? 'preset-filled-tertiary-500'
									: 'preset-filled'} {id !== own_id
									? 'hover:preset-filled-error-500'
									: 'pointer-events-none'} px-4 py-2 transition-colors"
								title="Kick Player?"
								onclick={() => kick(own_id, id)}
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
								class="card relative {id === own_id
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
						<div class="grid w-full grid-cols-[1fr_auto] gap-1.5">
							<label class="label">
								<span class="label-text">Card Sets</span>
								{#each changable_settings.decks as deck}
									<label class="flex items-center space-x-2">
										<input
											disabled={!isHost}
											class="checkbox"
											type="checkbox"
											checked={deck.enabled}
											onclick={() => (deck.enabled = !deck.enabled)}
										/>
										<p>{deck.name}</p>
									</label>
								{/each}
							</label>
							<div class="pt-[22px]">
								<AddDeck {ws} disabled={!isHost} />
							</div>
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

								<select
									class="select"
									bind:value={changable_settings.max_players}
									disabled={!isHost}
								>
									{#each Array.from({ length: 69 }) as _, i}
										{@const round = i + 1}
										<option value={round}>{round}</option>
									{/each}
								</select>
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
								disabled={areObjectsEqual(changable_settings, lobby_state?.settings)}
								onclick={() => (changable_settings = deepClone(lobby_state?.settings))}
								>Reset</button
							>
							<button
								class="btn preset-filled-success-500"
								disabled={areObjectsEqual(changable_settings, lobby_state?.settings)}
								onclick={update_settings}>Apply</button
							>
						</div>
					{/if}
				{/if}
			</Tabs.Panel>
		{/snippet}
	</Tabs>
</div>
