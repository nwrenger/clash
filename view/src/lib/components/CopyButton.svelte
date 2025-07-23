<script lang="ts">
	import clickToCopy from '$lib/clickToCopy';
	import type { Snippet } from 'svelte';

	interface Props {
		text: string | (() => string);
		child: Snippet<[{ copied: boolean }]>;
		[key: string]: any;
	}

	let { text, child, ...restProps }: Props = $props();

	let copyTimer: number | null = null;
	let copied = $state(false);

	$effect(() => {
		if (text) {
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
</script>

<button
	use:clickToCopy={{ text }}
	oncopyDone={handleCopyDone}
	oncopyError={handleCopyError}
	{...restProps}
>
	{@render child({ copied })}
</button>
