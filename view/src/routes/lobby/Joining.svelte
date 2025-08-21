<script lang="ts">
	interface Props {
		own_name: string;
		join_lobby: () => void;
	}

	let { own_name = $bindable(), join_lobby }: Props = $props();
</script>

<div class="mx-auto flex max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	<h4 class="h4 text-center">Join Lobby</h4>

	<div class="w-full sm:max-w-xs">
		<label class="label">
			<span class="label-text">Your Nickname</span>
			<input
				class="input w-full"
				type="text"
				placeholder="Enter your nickname…"
				bind:value={own_name}
				onkeydown={(e) => e.key === 'Enter' && join_lobby()}
			/>
		</label>
	</div>

	<button
		class="btn preset-filled-primary-500 flex w-full items-center justify-center sm:w-auto"
		disabled={!own_name}
		onclick={join_lobby}
	>
		Join Lobby
	</button>
</div>
