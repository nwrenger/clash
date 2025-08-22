<script lang="ts">
	import Countdown from '$lib/components/Countdown.svelte';
	import { colorFromUUID, sortedEntries } from '$lib/utils';
	import { Crown, Gavel, UserRound } from 'lucide-svelte';
	import type { Lobby, Own, Round } from './+page.svelte';

	interface Props {
		lobby: Lobby;
		own: Own;
		round: Round;
	}

	let { lobby, own, round }: Props = $props();
</script>

<div class="grid w-full grid-cols-[1fr_auto]">
	<div class="flex max-w-full gap-2 overflow-x-auto px-2 py-1">
		{#each sortedEntries(lobby?.players) as [id, player]}
			<span
				class="card preset-filled {round.result?.player_id === id
					? 'outline-primary-500 outline-2 outline-offset-2'
					: ''} pointer-events-none px-4 py-2"
			>
				<span class="flex h-full w-full flex-col items-center justify-center">
					<span class="flex items-center space-x-1.5 {id === own.id ? 'text-primary-500' : ''}">
						<div
							class="w-7 rounded-sm p-1"
							style="background-color: {colorFromUUID(id).background};"
						>
							{#if player.is_host}
								<Crown color={colorFromUUID(id).text} size={20} strokeWidth={2.5} />
							{:else}
								<UserRound color={colorFromUUID(id).text} size={20} strokeWidth={2.5} />
							{/if}
						</div>
						<span class="font-semibold text-nowrap">{player.name}</span>
						{#if player.is_czar}
							<Gavel class="text-surface-50-950 animate-pulse" size={20} strokeWidth={2.25} />
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
	</div>
</div>
