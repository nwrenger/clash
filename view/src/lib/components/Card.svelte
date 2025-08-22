<script lang="ts">
	import api from '$lib/api';

	interface Props {
		card: api.WhiteCard | api.BlackCard;
		text_classes: string;
		card_classes: string;
		[key: string]: any;
	}

	let { text_classes, card_classes, card, ...restProps }: Props = $props();
</script>

<button
	{...restProps}
	class="{card_classes} relative h-48 w-32 flex-shrink-0 transform
	rounded-xl shadow-sm transition-transform duration-300 ease-out select-none perspective-midrange hover:z-50"
>
	<!-- Card frame and holo pattern overlay -->
	<div class="absolute inset-0 rounded-xl bg-gradient-to-br from-white/10 to-black/10"></div>
	<!-- Card content -->
	<div class="relative z-10 flex h-full flex-col items-center justify-center p-2">
		{#if card.text.startsWith('[img]') && card.text.endsWith('[/img]')}
			{@const url = card.text.replace('[img]', '').replace('[/img]', '')}
			<img src={url} alt={url} class="aspect-auto h-fit max-h-full w-fit max-w-full" />
		{:else}
			{@const text = card.text.replaceAll('\n', '<br/>')}
			<span
				class="{text_classes} {card.text.length > 95
					? 'text-xs'
					: 'text-sm'} text-sm font-bold break-normal"
			>
				{@html text}
			</span>
		{/if}
	</div>
</button>
