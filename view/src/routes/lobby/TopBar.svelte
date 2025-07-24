<script lang="ts">
	import type api from '$lib/api';
	import Countdown from '$lib/components/Countdown.svelte';
	import { sortedEntries } from '$lib/utils';
	import { Crown } from 'lucide-svelte';

	interface Props {
		lobby_state: api.LobbyState | undefined;
		own_id: api.Uuid;
		round: number;
		time: { self?: number };
		winner?: api.Uuid;
	}

	let { lobby_state, own_id, round, time, winner }: Props = $props();
</script>

<div class="grid w-full grid-cols-[1fr_auto]">
	<div class="flex max-w-full gap-2 overflow-x-auto px-2 py-1">
		{#each sortedEntries(lobby_state?.players) as [id, player]}
			<span
				class="card {id === own_id ? 'preset-filled-tertiary-500' : 'preset-filled'} {winner === id
					? 'outline-primary-500 outline-2 outline-offset-2'
					: ''} pointer-events-none px-4 py-2"
			>
				<span class="flex h-full w-full flex-col items-center justify-center">
					<span class="flex items-center space-x-1.5">
						{#if player.is_host}
							<Crown size={16} />
						{/if}
						<span class="text-nowrap">{player.name}</span>
					</span>
					<span>
						{player.points}
					</span>
				</span>
			</span>
		{/each}
	</div>

	<div class="flex w-full items-center justify-center space-x-2">
		{#key time}
			{#if time.self}
				<Countdown time={time.self} />
			{/if}
		{/key}
		<div class="card preset-filled-secondary-500 w-fit px-4 py-2">
			{round}
		</div>
	</div>
</div>
