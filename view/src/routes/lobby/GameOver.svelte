<script lang="ts">
	import confetti from 'canvas-confetti';
	import api from '$lib/api';
	import { sortedEntries } from '$lib/utils';
	import { Crown, Trophy } from 'lucide-svelte';

	interface Props {
		ws: WebSocket | undefined;
		lobby_state: api.LobbyState | undefined;
		own_id: api.Uuid;
	}

	let { ws, lobby_state, own_id }: Props = $props();

	function reset_game() {
		api.send_ws(ws!, { type: 'RestartRound' });
	}

	// Fire confetti when component mounts
	$effect(() => {
		confetti({
			particleCount: 200,
			spread: 70,
			origin: { y: 0.3 }
		});
	});

	// Top three players for podium
	const topThree = () =>
		sortedEntries(lobby_state?.players)
			.toSorted((a, b) => b[1].points - a[1].points)
			.slice(0, 3);
	const others = () =>
		sortedEntries(lobby_state?.players)
			.toSorted((a, b) => b[1].points - a[1].points)
			.slice(3);
</script>

<div class="mx-auto max-w-4xl space-y-8 px-6 py-12 text-center">
	<h1 class="text-5xl font-extrabold">Game Over!</h1>
	<p class="text-lg">Here are your champions:</p>

	<div class="grid grid-cols-3 items-end gap-6">
		{#each [0, 1, 2] as idx}
			{#if topThree()[idx]}
				{@const [id, player] = topThree()[idx]}
				<div class="flex flex-col items-center space-y-2">
					<div
						class="card {idx === 0
							? 'bg-gradient-to-br from-yellow-300 to-yellow-600 text-white'
							: idx === 1
								? 'bg-gradient-to-br from-gray-300 to-gray-500 text-gray-900'
								: 'bg-gradient-to-br from-amber-600 to-orange-800 text-white'} flex w-20 flex-col items-center justify-center space-y-1 rounded-t-2xl py-4 shadow-lg sm:w-32 {id ===
						own_id
							? 'outline-secondary-500 outline-2 outline-offset-2'
							: ''}"
						style="height: {idx === 0 ? '200px' : idx === 1 ? '160px' : '140px'}"
					>
						{#if idx === 0}
							<Trophy size={32} />
						{/if}
						<div class="flex w-16 items-center justify-center space-x-1.5 sm:w-20">
							{#if player.is_host}
								<Crown size={16} />
							{/if}
							<span class="truncate text-xl font-semibold" title={player.name}>{player.name}</span>
						</div>
						<span class="hidden text-2xl sm:block">{player.points} pts</span>
						<span class="text-2xl sm:hidden">{player.points} </span>
					</div>
					<span class="text-surface-800-200 text-sm">
						{idx === 0 ? '1st Place' : idx === 1 ? '2nd Place' : '3rd Place'}
					</span>
				</div>
			{/if}
		{/each}
	</div>

	<!-- Others list -->
	{#if others().length}
		<div class="space-y-2 px-4 pt-4 sm:px-16">
			{#each others() as [id, player]}
				<div
					class="preset-filled grid w-full grid-cols-[1fr_auto] rounded-lg px-5 py-3 shadow-md
					{id === own_id ? 'outline-secondary-500 outline-2 outline-offset-2' : ''}"
				>
					<div class="flex w-full max-w-full min-w-0 flex-1 items-center justify-start space-x-1.5">
						<div class="w-4">
							{#if player.is_host}
								<Crown size={16} />
							{/if}
						</div>
						<span class="truncate text-xl font-semibold" title={player.name}>{player.name}</span>
					</div>
					<span class="text-md flex items-center font-medium">{player.points}</span>
				</div>
			{/each}
		</div>
	{/if}

	{#if lobby_state?.players[own_id]?.is_host}
		<button class="btn preset-filled-primary-500" onclick={reset_game}> Restart Game </button>
	{/if}
</div>
