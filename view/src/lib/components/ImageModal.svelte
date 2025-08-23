<script lang="ts">
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { Fullscreen, X } from 'lucide-svelte';

	interface Props {
		image_url: string;
	}

	let { image_url }: Props = $props();
	let openState = $state(false);

	function modalClose() {
		openState = false;
	}
</script>

<Modal
	open={openState}
	onclick={(e) => e.stopPropagation()}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase="btn-icon preset-filled p-1 absolute top-1 left-1 z-[100]"
	contentBase="p-0 relative rounded-md max-w-[90vw] max-h-[90vh] overflow-auto"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet trigger()}
		<Fullscreen size={20} strokeWidth={2.5} />
	{/snippet}
	{#snippet content()}
		<img
			src={image_url}
			alt={image_url}
			title="Source: {image_url}"
			class="block h-auto max-h-[90vh] w-auto max-w-[90vw] rounded-md object-contain"
		/>
		<button
			type="button"
			class="btn-icon preset-filled absolute top-1 right-1 p-1"
			onclick={modalClose}
		>
			<X />
		</button>
	{/snippet}
</Modal>
