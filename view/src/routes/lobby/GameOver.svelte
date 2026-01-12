<script lang="ts">
	import confetti from 'canvas-confetti';
	import api from '$lib/api';
	import { colorFromUUID, sortedEntries } from '$lib/utils';
	import { Crown, RotateCw, Trophy, UserRound } from 'lucide-svelte';
	import type { Connection, Lobby, Own } from './+page.svelte';

	interface Props {
		connection: Connection;
		lobby: Lobby;
		own: Own;
	}

	let { connection, lobby, own }: Props = $props();

	function reset_game() {
		api.send_ws(connection.ws!, { type: 'RestartRound' });
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
		sortedEntries(lobby?.players)
			.toSorted((a, b) => b[1].points - a[1].points)
			.slice(0, 3);
	const others = () =>
		sortedEntries(lobby?.players)
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
							? 'bg-linear-to-br from-yellow-300 to-yellow-600 text-white'
							: idx === 1
								? 'bg-linear-to-br from-gray-300 to-gray-500 text-gray-900'
								: 'bg-linear-to-br from-amber-600 to-orange-800 text-white'} flex w-28 flex-col items-center justify-center space-y-1 rounded-t-2xl py-4 shadow-lg sm:w-36 md:w-44"
						style="height: {idx === 0 ? '200px' : idx === 1 ? '160px' : '140px'}"
					>
						{#if idx === 0}
							<Trophy size={32} />
						{/if}
						<div
							class="flex w-full items-center justify-center space-x-1.5 px-1 {id ===
							own.credentials.id
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
							<span class="truncate text-xl font-semibold" title={player.name}>{player.name}</span>
						</div>
						<span class="hidden text-2xl sm:block">{player.points} pts</span>
						<span class="text-2xl sm:hidden">{player.points} </span>
					</div>
					<span class="text-surface-800-200 text-sm text-nowrap">
						{idx === 0 ? '1st Place' : idx === 1 ? '2nd Place' : '3rd Place'}
					</span>
				</div>
			{/if}
		{/each}
	</div>

	<!-- Others list -->
	{#if others().length}
		<div class="min-h-0 space-y-2 px-2 py-4">
			{#each others() as [id, player]}
				<div class="preset-filled grid w-full grid-cols-[1fr_auto] rounded-lg px-5 py-3">
					<div
						class="flex w-full max-w-full min-w-0 flex-1 items-center justify-start space-x-1.5 {id ===
						own.credentials.id
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
						<span class="truncate text-xl font-semibold" title={player.name}>{player.name}</span>
					</div>
					<span class="text-md flex items-center font-medium">{player.points}</span>
				</div>
			{/each}
		</div>
	{/if}

	{#if lobby!.players![own.credentials.id]?.is_host}
		<div class="sticky bottom-0 z-50 flex w-full flex-col items-center justify-center">
			<div class="preset-tonal w-fit rounded-md p-2 backdrop-blur-lg">
				<button class="btn preset-filled-primary-500" onclick={reset_game}>
					<RotateCw size={20} />
					Restart Game
				</button>
			</div>
		</div>
	{/if}
</div>
