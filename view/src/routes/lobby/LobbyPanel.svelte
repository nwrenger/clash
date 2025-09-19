<script lang="ts">
	import { page } from '$app/state';
	import api from '$lib/api';
	import ShareButton from '$lib/components/ShareButton.svelte';
	import { colorFromUUID, sortedEntries } from '$lib/utils';
	import {
		Ban,
		UserRound,
		ClipboardCopy,
		Crown,
		Link,
		LoaderCircle,
		Play,
		LogOut
	} from 'lucide-svelte';
	import type { Connection, Lobby, Own } from './+page.svelte';
	import { goto } from '$app/navigation';
	import type { Shared } from './LobbyOpen.svelte';

	interface Props {
		connection: Connection;
		lobby: Lobby;
		own: Own;
		resetLogin: () => void;
		shared: Shared;
	}

	let { connection, lobby, own, resetLogin, shared }: Props = $props();

	let is_host = $derived(lobby!.players![own.credentials.id]?.is_host || false);
	let lobby_url = $derived(`${page.url.origin}/lobby?id=${lobby.id}`);

	function kick(player_id: api.Uuid) {
		if (own.credentials.id != player_id)
			api.send_ws(connection.ws!, { type: 'Kick', data: { kicked: player_id } });
	}

	async function leave() {
		api.send_ws(connection.ws!, { type: 'LeaveLobby' });
		resetLogin();

		// Return to homepage and making sure that the current page state won't be invalidated
		await goto('/', { replaceState: false, invalidateAll: false });
	}

	function start_game() {
		api.send_ws(connection.ws!, { type: 'StartRound' });
	}
</script>

<div class="h-full overflow-y-auto">
	<div class="min-h-0 space-y-2 px-2 py-4">
		{#each sortedEntries(lobby?.players) as [id, player]}
			<div class="preset-filled grid w-full grid-cols-[1fr_auto] rounded-lg px-5 py-3">
				<div
					class="flex w-full max-w-full min-w-0 flex-1 items-center justify-start space-x-1.5 {id ===
					own.credentials.id
						? 'text-primary-500'
						: ''}"
				>
					<div class="w-7 rounded-sm p-1" style="background-color: {colorFromUUID(id).background};">
						{#if player.is_host}
							<Crown color={colorFromUUID(id).text} size={20} />
						{:else}
							<UserRound color={colorFromUUID(id).text} size={20} />
						{/if}
					</div>
					<span class="truncate text-xl font-semibold" title={player.name}>{player.name}</span>
				</div>

				{#if id !== own.credentials.id && is_host}
					<button
						class="btn-icon text-error-500 h-full w-fit p-0"
						title="Kick {player.name}"
						onclick={() => kick(id)}
					>
						<Ban size={20} />
					</button>
				{/if}
			</div>
		{/each}
	</div>

	<div class="sticky bottom-0 z-50 mb-8 flex w-full flex-col items-center justify-center">
		<div
			class="preset-tonal grid w-full gap-1.5 rounded-md p-2 backdrop-blur-lg {is_host
				? 'sm:grid-cols-3'
				: 'sm:grid-cols-2'}"
		>
			<button class="btn preset-filled-error-500 h-fit w-full" title="Log Out" onclick={leave}>
				<LogOut size={20} />
				Log Out
			</button>
			<ShareButton class="btn preset-filled-primary-500 h-fit w-full" url={lobby_url}>
				{#snippet child({ copied })}
					{#if copied}
						<ClipboardCopy size={20} />
						Copied Invite
					{:else}
						<Link size={20} />
						Invite Players
					{/if}
				{/snippet}
			</ShareButton>

			{#if is_host}
				<div class="flex w-full flex-col space-y-2">
					<button
						class="btn preset-filled-primary-500"
						onclick={start_game}
						disabled={shared.saving || !shared.valid_config}
					>
						<Play size={20} />
						Start Game
					</button>

					{#if shared.saving}
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
