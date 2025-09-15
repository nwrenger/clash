<script lang="ts">
	import { RefreshCw, X } from 'lucide-svelte';

	interface Props {
		connect: () => void;
	}

	let { connect }: Props = $props();

	let loading = $state(true);

	// show buttons after a short delay
	setTimeout(() => {
		loading = false;
	}, 2000);
</script>

<div class="mx-auto flex max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	{#if loading}
		<p class="text-center">Loading...</p>
	{:else}
		<div class="preset-tonal grid w-full gap-1.5 rounded-md p-2 backdrop-blur-lg sm:grid-cols-2">
			<button class="btn preset-filled-primary-500 h-fit w-full" onclick={connect}>
				<RefreshCw size={20} />
				Reconnect
			</button>
			<a href="/" class="btn preset-filled-error-500">
				<X size={20} />
				Close Connection
			</a>
		</div>
	{/if}
</div>
