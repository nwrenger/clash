<script lang="ts">
	import { goto } from '$app/navigation';
	import api from '$lib/api';
	import { credentials } from '$lib/state';
	import { handle_promise } from '$lib/toaster';
	import { randomName } from '$lib/utils';
	import { LoaderCircle, Plus, Sparkles, Github, ExternalLink, Shuffle } from 'lucide-svelte';

	let name: string = $state('');
	let creating = $state(false);

	async function createLobby(e: Event) {
		if (!name.trim() || creating) return;

		e.preventDefault();
		creating = true;

		try {
			const id: api.Uuid = crypto.randomUUID();
			const data = await handle_promise(api.create_lobby(name.trim(), id));
			$credentials = { lobby_id: data.id, id, name: name.trim() };
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

<div class="mx-auto flex w-full max-w-3xl flex-col items-center px-4 py-8 sm:py-14">
	<!-- hero -->
	<div class="badge preset-tonal flex-wrap rounded-full">
		<span class="inline-flex items-center gap-1"><Sparkles size={14} /> public beta</span>
		<span class="mx-2 opacity-40">•</span>
		<span>built with Rust + Svelte</span>
	</div>

	<h1 class="h2 pt-4 text-center">
		Cards <span class="text-primary-500">(Ludicrous Ones)</span> Against Humanity
	</h1>
	<p class="pt-4 text-center sm:text-lg">
		A lightning-fast, open-source, browser-based party game. Assemble your friends, unleash chaos,
		and crown the most unhinged comedian.
	</p>

	<!-- action card -->
	<div class="w-full pt-10">
		<div class="card preset-tonal mx-auto w-full max-w-xl overflow-hidden rounded-2xl">
			<div class="p-5 sm:p-7">
				<form class="space-y-3" onsubmit={createLobby} aria-describedby="helper">
					<label class="label">
						<span class="label-text flex items-center justify-between pe-[41px]">
							<span>Your nickname</span>
							<span>
								press <kbd class="kbd text-xs">↵</kbd> to create
							</span>
						</span>

						<div class="flex gap-2">
							<input
								class="input w-full"
								type="text"
								placeholder="ChaoticWombat..."
								bind:value={name}
								autocomplete="nickname"
								disabled={creating}
								onkeydown={(e) => e.key === 'Enter' && createLobby(e)}
								maxlength="24"
								aria-invalid={!name.trim() && creating ? 'true' : 'false'}
							/>
							<button
								type="button"
								class="btn-icon preset-tonal"
								title="Surprise me!"
								onclick={() => (name = randomName())}
								disabled={creating}
							>
								<Shuffle size={18} />
							</button>
						</div>
					</label>

					<button
						type="submit"
						class="btn preset-filled-primary-500 mt-2 flex w-full items-center justify-center gap-2"
						disabled={!name.trim() || creating}
						aria-busy={creating}
					>
						{#if creating}
							<LoaderCircle size={20} class="animate-spin" />
							Creating lobby…
						{:else}
							<Plus size={20} />
							Create lobby
						{/if}
					</button>

					<p class="text-on-surface-variant text-sm">
						Nicknames are visible to players. Keep it spicy 🌶️
					</p>
				</form>
			</div>

			<hr class="hr" />

			<!-- strip footer -->
			<div class="flex flex-wrap items-center justify-between gap-3 p-5 sm:p-7">
				<a
					href="https://github.com/nwrenger/clash"
					target="_blank"
					rel="noopener noreferrer"
					class="btn preset-tonal h-9 w-full"
				>
					<Github size={18} />
					Star on GitHub
					<ExternalLink size={16} class="opacity-60" />
				</a>

				<ul class="flex w-full flex-wrap items-center justify-center gap-2">
					<li class="badge preset-filled">Open Source</li>
					<li class="badge preset-filled-secondary-500">Rust Backend</li>
					<li class="badge preset-filled-tertiary-500">Svelte Frontend</li>
				</ul>
			</div>
		</div>
	</div>

	<!-- feature bites -->
	<div class="grid w-full max-w-5xl grid-cols-1 gap-4 pt-12 sm:grid-cols-3">
		<div
			class="card preset-tonal space-y-1 p-5 transition will-change-transform hover:-translate-y-[2px]"
		>
			<h3 class="font-semibold">Blazing rounds</h3>
			<p>Zero-chill latency and snappy UI for chaotic fun.</p>
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

	<p class="pt-8 text-center text-sm">
		<strong>Beta notice:</strong> expect rapid updates and the occasional breaking change.
	</p>
</div>
