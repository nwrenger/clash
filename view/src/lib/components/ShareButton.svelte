<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		url: string;
		child: Snippet<[{ copied: boolean }]>;
		[key: string]: any;
	}

	let { url, child, ...restProps }: Props = $props();

	let copyTimer: number | null = null;
	let copied = $state(false);

	$effect(() => {
		if (url) {
			clearCopyTimer();
			copied = false;
		}
	});

	function handleCopyDone() {
		clearCopyTimer();
		copied = true;
		copyTimer = setTimeout(() => {
			copied = false;
		}, 1000);
	}

	function clearCopyTimer() {
		if (copyTimer !== null) {
			clearTimeout(copyTimer);
			copyTimer = null;
		}
	}

	function handleCopyError() {
		console.error('Error copying.');
	}

	async function handleAction() {
		try {
			await navigator.clipboard.writeText(url);
			handleCopyDone();
		} catch {
			handleCopyError();
		}
	}
</script>

<button onclick={handleAction} {...restProps}>
	{@render child({ copied })}
</button>
