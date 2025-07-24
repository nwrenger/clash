<script lang="ts">
	import { goto } from '$app/navigation';
	import api from '$lib/api';
	import { own } from '$lib/state';
	import { handle_promise } from '$lib/toaster';
	import { Plus } from 'lucide-svelte';

	let name: string = $state('');

	async function createLobby() {
		const id: api.Uuid = crypto.randomUUID();
		const data = await handle_promise(api.create_lobby(name, id));
		$own = { lobby_id: data.id, id, name };
		goto(`/lobby?id=${data.id}`);
	}
</script>

<svelte:head>
	<title>Cards (Ludicrous ones) Against Humanity</title>
	<meta
		name="description"
		content="CLASH is the ultimate irreverent party game where your wildest cards collide with humanity’s darkest humor."
	/>
</svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	<h3 class="h3 text-center">Cards (Ludicrous ones) Against Humanity</h3>
	<p class="text-center">
		Dive into <strong>CLASH</strong>, where outrageous cards collide in a battle of chaos and
		laughter. Open‑source on
		<a href="https://github.com/nwrenger/clash" class="anchor" target="_blank">GitHub</a>. Built
		with <em>Rust</em> and <em>Svelte</em> for blazing speed.
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
			/>
		</label>
	</div>

	<button
		class="btn preset-filled-primary-500 flex w-full items-center justify-center sm:w-auto"
		disabled={!name}
		onclick={createLobby}
	>
		<Plus class="mr-2" />
		Create Lobby
	</button>
</div>
