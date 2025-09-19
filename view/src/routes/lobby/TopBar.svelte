<script lang="ts">
	import Countdown from '$lib/components/Countdown.svelte';
	import { colorFromUUID, sortedEntries } from '$lib/utils';
	import { Crown, Gavel, UserRound } from 'lucide-svelte';
	import type { Connection, Lobby, Own, Round } from './+page.svelte';
	import EndGame from './EndGame.svelte';

	interface Props {
		connection: Connection;
		lobby: Lobby;
		own: Own;
		round: Round;
	}

	let { connection, lobby, own, round }: Props = $props();

	let is_host = $derived(lobby!.players![own.credentials.id]?.is_host || false);

	$effect(() => {
		if (round.result?.player_id) {
			let el = document.getElementById(`points-${round.result.player_id}`);
			el?.scrollIntoView({ inline: 'center', behavior: 'smooth' });
		}
	});
</script>

<div class="grid w-full grid-cols-[1fr_auto]">
	<div class="flex max-w-full gap-2 overflow-x-auto px-2 py-1">
		{#each sortedEntries(lobby?.players) as [id, player]}
			<span
				id="points-{id}"
				class="card preset-filled {round.result?.player_id === id
					? 'outline-primary-500 outline-2 outline-offset-2'
					: ''} pointer-events-none px-4 py-2"
			>
				<span class="flex h-full w-full flex-col items-center justify-center">
					<span
						class="flex items-center space-x-1.5 {id === own.credentials.id
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
						<span class="font-semibold text-nowrap">{player.name}</span>
						{#if player.is_czar}
							<div class="sprite is-falling">
								<Gavel class="text-surface-50-950" size={20} />
							</div>
						{/if}
					</span>
					<span>
						{player.points}
					</span>
				</span>
			</span>
		{/each}
	</div>

	<div class="flex w-full items-center justify-center space-x-2">
		{#key round.time}
			{#if round.time.self}
				<Countdown time={round.time.self} />
			{/if}
		{/key}
		<div class="card preset-filled-secondary-500 w-fit px-4 py-2">
			{round.count}
		</div>
		{#if is_host}
			<EndGame {connection} />
		{/if}
	</div>
</div>

<style>
	.sprite {
		--fall-tilt: 45deg;
		--cycle: 2s;
		--fall-x: -5px;
		--fall-y: 0px;
		--slam-squash: 0.88;
		--slam-stretch: 1.06;
		display: inline-block;
		transform-origin: 50% 100%;
		will-change: transform;
	}

	.is-falling {
		animation: fall-impact-rise var(--cycle) infinite;
	}

	@keyframes fall-impact-rise {
		0% {
			transform: translate(0, 0) rotate(0deg) scale(1, 1);
			animation-timing-function: cubic-bezier(0.15, 1, 0.3, 1);
			filter: none;
		}
		12% {
			filter: saturate(1.08) contrast(1.03);
			transform: translate(var(--fall-x), var(--fall-y))
				scale(var(--slam-stretch), var(--slam-squash)) rotate(var(--fall-tilt));
			animation-timing-function: cubic-bezier(0.2, 1, 0.3, 1);
		}
		14% {
			filter: saturate(1.08) contrast(1.03);
			transform: translate(var(--fall-x), var(--fall-y)) scale(1, 1) rotate(var(--fall-tilt));
			animation-timing-function: cubic-bezier(0.1, 0.4, 0, 1);
		}
		100% {
			transform: translate(0, 0) rotate(0deg) scale(1, 1);
			filter: none;
		}
	}

	/* a11y */
	@media (prefers-reduced-motion: reduce) {
		.is-falling {
			animation: none;
		}
	}
</style>
