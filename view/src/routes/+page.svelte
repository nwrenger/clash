<script lang="ts">
	import { goto } from '$app/navigation';
	import api from '$lib/api';
	import { session } from '$lib/state';
	import { handle_promise } from '$lib/toaster';
	import { randomName } from '$lib/utils';
	import { LoaderCircle, Plus, Sparkles, Github, ExternalLink, Shuffle } from 'lucide-svelte';
	import LoginCard from './LoginCard.svelte';

	let name: string = $state('');
	let creating = $state(false);

	async function createLobby(e: Event) {
		if (!name.trim() || creating) return;

		e.preventDefault();
		creating = true;

		try {
			const credentials = {
				name: name.trim(),
				id: crypto.randomUUID(),
				secret: crypto.randomUUID()
			};
			const data = await handle_promise(api.create_lobby(credentials));
			$session = { lobby_id: data.id, credentials };
			await goto(`/lobby?id=${data.id}`);
		} finally {
			creating = false;
		}
	}
</script>

<svelte:head>
	<title>Cards (Ludicrous Ones) Against Humanity</title>
	<meta
		name="description"
		content="Cards (Ludicrous Ones) Against Humanity (CLASH) is a lightning-fast, online and open-source Cards Against Humanity alternative built with Rust and Svelte."
	/>
	<!-- Open Graph -->
	<meta property="og:title" content="Cards (Ludicrous Ones) Against Humanity" />
	<meta
		property="og:description"
		content="Cards (Ludicrous Ones) Against Humanity (CLASH) is a lightning-fast, online and open-source Cards Against Humanity alternative built with Rust and Svelte."
	/>
</svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col items-center space-y-4 px-4 py-8 sm:py-14">
	<!-- hero -->
	<div class="badge preset-tonal flex-wrap rounded-full">
		<span class="inline-flex items-center gap-1"><Sparkles size={14} /> now live</span>
		<span class="mx-2 opacity-40">•</span>
		<span>free to play</span>
	</div>

	<h1 class="h2 text-center">
		Cards <span class="text-primary-500">(Ludicrous Ones)</span> Against Humanity
	</h1>
	<p class="text-center sm:text-lg">
		A lightning-fast, open-source, browser-based party game. Assemble your friends, unleash chaos,
		and crown the most unhinged comedian.
	</p>

	<!-- action card -->
	<div class="w-full pt-4">
		<LoginCard bind:name cta="create" submit={createLobby} busy={creating}>
			{#snippet buttonText({ busy })}
				{#if busy}
					<LoaderCircle size={20} class="animate-spin" />
					Creating lobby…
				{:else}
					<Plus size={20} />
					Create lobby
				{/if}
			{/snippet}
		</LoginCard>
	</div>

	<!-- slim social/tech strip (moved out of CTA card) -->
	<div class="flex w-full max-w-xl flex-wrap items-center justify-between gap-3 pt-2">
		<a
			href="https://github.com/nwrenger/clash"
			target="_blank"
			class="btn preset-tonal h-9 w-full sm:w-auto"
		>
			<Github size={18} />
			Star on GitHub
			<ExternalLink size={16} class="opacity-60" />
		</a>

		<ul class="flex w-full flex-wrap items-center justify-center gap-2 sm:w-auto sm:justify-evenly">
			<li class="badge preset-filled">Open Source</li>
			<li class="badge preset-filled-secondary-500">Rust Backend</li>
			<li class="badge preset-filled-tertiary-500">Svelte Frontend</li>
		</ul>
	</div>

	<!-- feature bites -->
	<div class="grid w-full max-w-5xl grid-cols-1 gap-4 pt-6 sm:grid-cols-2 md:grid-cols-4">
		<div class="card preset-tonal space-y-1 p-5 transition hover:-translate-y-[2px]">
			<h3 class="font-semibold">No accounts</h3>
			<p>Pick a nickname and play. No logins, no hassle!</p>
		</div>
		<div
			class="card preset-tonal space-y-1 p-5 transition will-change-transform hover:-translate-y-[2px]"
		>
			<h3 class="font-semibold">Blazing rounds</h3>
			<p>Minimal latency and snappy UI for chaotic fun.</p>
		</div>
		<div class="card preset-tonal space-y-1 p-5 transition hover:-translate-y-[2px]">
			<h3 class="font-semibold">Private lobbies</h3>
			<p>Spin up a room and drop the link. Done!</p>
		</div>
		<div class="card preset-tonal space-y-1 p-5 transition hover:-translate-y-[2px]">
			<h3 class="font-semibold">Extensible</h3>
			<p>House rules? Custom packs? Tweak away.</p>
		</div>
	</div>
</div>
