<script lang="ts">
	import api from '$lib/api';
	import { sortedEntries } from '$lib/utils';
	import { Crown } from 'lucide-svelte';

	interface Props {
		ws: WebSocket | undefined;
		lobby_state: api.LobbyState | undefined;
		own_id: api.Uuid;
	}

	let { ws, lobby_state, own_id }: Props = $props();

	function reset_game() {
		api.send_ws(ws!, { type: 'RestartRound' });
	}
</script>

<div class="mx-auto flex max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	<p>The game is over. Wait for your host to restart the lobby!</p>

	<div class="flex w-full flex-col gap-2 px-2 py-1">
		{#each sortedEntries(lobby_state?.players).toSorted((a, b) => b[1].points - a[1].points) as [id, player]}
			<span
				class="card {id === own_id
					? 'preset-filled-tertiary-500'
					: 'preset-filled'}  pointer-events-none px-4 py-2"
			>
				<span class="flex h-full w-full flex-col items-center justify-center">
					<span class="flex items-center space-x-1.5 text-lg">
						{#if player.is_host}
							<Crown size={16} />
						{/if}
						<span>{player.name}</span>
					</span>
					<span class="text-lg">
						{player.points}
					</span>
				</span>
			</span>
		{/each}
	</div>

	{#if lobby_state?.players[own_id]?.is_host}
		<button class="btn preset-filled-primary-500" onclick={reset_game}>Restart</button>
	{/if}
</div>
