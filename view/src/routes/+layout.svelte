<script lang="ts">
	import '../app.css';
	import { toaster } from '$lib/toaster';
	import { Toaster } from '@skeletonlabs/skeleton-svelte';

	let { children } = $props();
</script>

<Toaster {toaster}></Toaster>

{@render children()}
