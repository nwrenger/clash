<script lang="ts">
	interface Props {
		classes?: string;
		time: number;
	}

	let { classes, time }: Props = $props();

	let timer = setInterval(() => count--, 1_000);
	$effect(() => {
		if (count == 0) {
			clearInterval(timer);
		}
	});
	// svelte-ignore state_referenced_locally
	// Change count only locally, allow overwrites by the server
	let count = $state(time);
</script>

<div class="{classes} card preset-filled w-fit px-4 py-2">
	{count}s
</div>
