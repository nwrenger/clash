<script lang="ts">
	import api from '$lib/api';
	import Card from '$lib/components/Card.svelte';
	import type { Connection, Own, Round } from './+page.svelte';

	interface Props {
		connection: Connection;
		round: Round;
		own: Own;
		selectable: boolean;
		disabled: boolean;
	}

	let { connection, round, own = $bindable(), selectable, disabled }: Props = $props();
	let localSelectable = $state(selectable);
	$effect(() => {
		localSelectable = selectable;
	});

	function hasIndex(index: number) {
		return own.selected_cards.find((p) => p === index) != undefined;
	}

	function selectCard(index: number) {
		if (localSelectable && round.black_card) {
			if (own.selected_cards.length < round.black_card.fields && !hasIndex(index)) {
				own.selected_cards.push(index);
			}
			if (own.selected_cards.length == round.black_card.fields) {
				api.send_ws(connection.ws!, {
					type: 'SubmitOwnCards',
					data: { indexes: own.selected_cards }
				});
				localSelectable = false;
			}
		}
	}
</script>

<!-- Container fixed at bottom with soft gradient overlay -->
<div
	class="fixed bottom-0 left-0 z-50 w-full bg-gradient-to-t from-black/60 via-transparent to-transparent pb-6"
>
	<div class="flex h-full w-full items-center justify-center">
		<div class="flex items-center space-x-4 overflow-x-auto px-4 pt-9 pb-2 perspective-distant">
			{#each own.cards as card, index}
				<Card
					{card}
					card_classes="{hasIndex(index)
						? 'z-20 -translate-y-4 scale-120'
						: 'hover:-translate-y-2 hover:scale-110'}
						bg-surface-50 ml-[-18px] first:ml-0 hover:-rotate-2
						{disabled ? 'brightness-60 cursor-default' : ''}"
					text_classes="text-surface-950"
					onclick={() => selectCard(index)}
				/>
			{/each}

			<div>
				<span class="me-[0.1px] h-48"></span>
			</div>
		</div>
	</div>
</div>
