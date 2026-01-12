<script lang="ts">
	import '../app.css';
	import { toaster } from '$lib/toaster';
	import { Toast } from '@skeletonlabs/skeleton-svelte';

	let { children } = $props();
</script>

{@render children()}

<Toast.Group {toaster}>
	{#snippet children(toast)}
		<Toast {toast}>
			<Toast.Message>
				<Toast.Title class="text-md font-extrabold">{toast.title}</Toast.Title>
				<Toast.Description>{toast.description}</Toast.Description>
			</Toast.Message>
			<Toast.CloseTrigger />
		</Toast>
	{/snippet}
</Toast.Group>
