<script lang="ts">
	import { flyin, opacity } from '$lib/animations';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import { Fullscreen, X } from 'lucide-svelte';

	interface Props {
		image_url: string;
	}

	let { image_url }: Props = $props();
</script>

<Dialog>
	<Dialog.Trigger
		onclick={(e) => e.stopPropagation()}
		class="btn-icon bg-surface-950 text-surface-50 absolute top-1 left-1 z-100"
	>
		<Fullscreen />
	</Dialog.Trigger>
	<Portal>
		<Dialog.Backdrop class="bg-surface-50-950/50 fixed inset-0 z-50 backdrop-blur-sm {opacity}" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center p-4">
			<Dialog.Content
				class="relative max-h-[90vh] max-w-[90vw] overflow-auto rounded-md p-0 {flyin}"
			>
				<img
					src={image_url}
					alt={image_url}
					title="Source: {image_url}"
					class="block h-auto max-h-[90vh] w-auto max-w-[90vw] rounded-md object-contain"
				/>
				<Dialog.CloseTrigger class="btn-icon bg-surface-950 text-surface-50 absolute top-1 right-1">
					<X />
				</Dialog.CloseTrigger>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
