<script lang="ts">
	import { randomName } from '$lib/utils';
	import { DoorOpen, LogIn, Shuffle } from 'lucide-svelte';

	interface Props {
		own_name: string;
		join_lobby: (e: Event) => void;
	}

	let { own_name = $bindable(), join_lobby }: Props = $props();
</script>

<div class="mx-auto flex max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	<div class="card preset-tonal mx-auto w-full max-w-xl overflow-hidden rounded-2xl">
		<div class="p-5 sm:p-7">
			<form class="space-y-3" onsubmit={join_lobby} aria-describedby="helper">
				<label class="label">
					<span class="label-text flex items-center justify-between pe-[41px]">
						<span>Your nickname</span>
						<span>
							press <kbd class="kbd text-xs">↵</kbd> to join
						</span>
					</span>

					<div class="flex gap-2">
						<input
							class="input w-full"
							type="text"
							placeholder="ChaoticWombat..."
							bind:value={own_name}
							autocomplete="nickname"
							onkeydown={(e) => e.key === 'Enter' && join_lobby(e)}
							maxlength="24"
							aria-invalid={!own_name.trim() ? 'true' : 'false'}
						/>
						<button
							type="button"
							class="btn-icon preset-tonal"
							title="Surprise me!"
							onclick={() => (own_name = randomName())}
						>
							<Shuffle size={18} />
						</button>
					</div>
				</label>

				<button
					type="submit"
					class="btn preset-filled-primary-500 mt-2 flex w-full items-center justify-center gap-2"
					disabled={!own_name.trim()}
				>
					<LogIn size={20} />
					Join lobby
				</button>

				<p class="text-on-surface-variant text-sm">
					Nicknames are visible to players. Keep it spicy 🌶️
				</p>
			</form>
		</div>
	</div>
</div>
