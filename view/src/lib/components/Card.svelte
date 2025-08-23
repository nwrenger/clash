<script lang="ts">
	import api from '$lib/api';
	import { Fullscreen } from 'lucide-svelte';
	import ImageModal from './ImageModal.svelte';

	interface Props {
		card: api.WhiteCard | api.BlackCard;
		text_classes: string;
		card_classes: string;
		[key: string]: any;
	}

	let { text_classes, card_classes, card, ...restProps }: Props = $props();

	let is_image = $derived(card.text.startsWith('[img]') && card.text.endsWith('[/img]'));
	let image_url = $derived(card.text.replace('[img]', '').replace('[/img]', ''));
</script>

<div
	{...restProps}
	class="{card_classes} relative h-48 w-32 flex-shrink-0 transform cursor-pointer
	rounded-xl shadow-sm transition-transform duration-300 ease-out select-none perspective-midrange hover:z-50"
>
	{#if is_image}
		<ImageModal {image_url} />
	{/if}
	<!-- Card frame and holo pattern overlay -->
	<div class="absolute inset-0 rounded-xl bg-gradient-to-br from-white/10 to-black/10"></div>
	<!-- Card content -->
	<div class="relative z-10 flex h-full flex-col items-center justify-center p-2">
		{#if is_image}
			<img src={image_url} alt={image_url} class="aspect-auto h-fit max-h-full w-fit max-w-full" />
		{:else}
			{@const text = card.text.replaceAll('\n', '<br/>')}
			<span
				class="{text_classes} {card.text.length > 95
					? 'text-xs'
					: 'text-sm'} text-center text-sm font-bold break-normal"
			>
				{@html text}
			</span>
		{/if}
	</div>
</div>
