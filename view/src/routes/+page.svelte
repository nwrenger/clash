<script lang="ts">
	import { goto } from '$app/navigation';
	import api from '$lib/api';
	import { own } from '$lib/state';
	import { handle_promise } from '$lib/toaster';
	import { LoaderCircle, Plus } from 'lucide-svelte';

	let name: string = $state('');
	let creating = $state(false);

	async function createLobby() {
		creating = true;
		try {
			const id: api.Uuid = crypto.randomUUID();
			const data = await handle_promise(api.create_lobby(name, id));
			$own = { lobby_id: data.id, id, name };
			await goto(`/lobby?id=${data.id}`);
		} finally {
			creating = false;
		}
	}
</script>

<svelte:head>
	<title>Cards (Ludicrous ones) Against Humanity</title>
	<meta
		name="description"
		content="Cards (Ludicrous Ones) Against Humanity (CLASH) is a lightning‑fast, online and open‑source Cards Against Humanity alternative built with Rust and Svelte."
	/>
	<!-- Open Graph -->
	<meta property="og:title" content="Cards (Ludicrous ones) Against Humanity" />
	<meta
		property="og:description"
		content="Cards (Ludicrous Ones) Against Humanity (CLASH) is a lightning‑fast, online and open‑source Cards Against Humanity alternative built with Rust and Svelte."
	/>
</svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	<h3 class="h3 text-center">Cards (Ludicrous ones) Against Humanity</h3>
	<p class="text-center">
		Welcome to <strong>Cards (Ludicrous Ones) Against Humanity (CLASH)</strong>. An uproarious,
		open‑source spin on Cards Against Humanity where outrageous cards collide in a riot of chaos and
		laughs. Experience the mayhem live in your browser, powered by <em>Rust</em> and <em>Svelte</em>
		for lightning‑fast performance. Dive into the code on
		<a href="https://github.com/nwrenger/clash" class="anchor" target="_blank">GitHub</a>.
	</p>
	<p class="text-center">
		<strong>Beta Notice:</strong> Expect rapid updates and occasional breaking changes!
	</p>

	<div class="w-full sm:max-w-xs">
		<label class="label">
			<span class="label-text">Your Nickname</span>
			<input
				class="input w-full"
				type="text"
				placeholder="Enter your nickname…"
				bind:value={name}
				disabled={creating}
			/>
		</label>
	</div>

	<button
		class="btn preset-filled-primary-500 flex w-full items-center justify-center sm:w-auto"
		disabled={!name || creating}
		onclick={createLobby}
	>
		{#if creating}
			<LoaderCircle class="animate-spin" />
			Creating...
		{:else}
			<Plus />
			Create Lobby
		{/if}
	</button>
</div>
