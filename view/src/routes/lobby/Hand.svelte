<script lang="ts">
	import api from '$lib/api';
	import Card from '$lib/components/Card.svelte';

	interface Props {
		ws: WebSocket | undefined;
		selectedIndexes: number[];
		black_card: api.BlackCard;
		selectable?: boolean;
		cards: api.WhiteCard[];
		disabled?: boolean;
	}

	let {
		ws,
		selectedIndexes = $bindable(),
		black_card,
		selectable,
		cards,
		disabled
	}: Props = $props();

	function selectCard(index: number) {
		if (selectable) {
			if (selectedIndexes.length < black_card.fields) {
				selectedIndexes.push(index);
			}
			if (selectedIndexes.length == black_card.fields) {
				api.send_ws(ws!, { type: 'SubmitOwnCards', data: { indexes: selectedIndexes } });
			}
		}
	}
</script>

<!-- Container fixed at bottom with soft gradient overlay -->
<div
	class="fixed bottom-0 left-0 z-50 w-full translate-y-40 bg-gradient-to-t from-black/60 via-transparent to-transparent py-6 backdrop-blur-sm transition-transform duration-300 ease-out hover:translate-y-0 {disabled
		? 'opacity-60'
		: ''}"
>
	<div class="flex h-full w-full items-center justify-center">
		<div class="flex items-center space-x-4 overflow-x-scroll px-4 pt-9 pb-2 perspective-distant">
			{#each cards as card, index (index)}
				<Card
					{card}
					card_classes="{selectedIndexes.find((p) => p == index)
						? 'z-20 -translate-y-4 scale-120 hover:-translate-y-4! hover:scale-120!'
						: ''}
						bg-surface-50 ml-[-3rem] first:ml-0"
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
