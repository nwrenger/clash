<script lang="ts">
	import { randomName } from '$lib/utils';
	import { Shuffle } from 'lucide-svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		name: string;
		cta: string;
		buttonText: Snippet<[{ busy?: boolean }]>;
		submit: (e: SubmitEvent) => void;
		busy?: boolean;
	}

	let { name = $bindable(), cta, buttonText, submit, busy }: Props = $props();
</script>

<div class="card preset-tonal mx-auto w-full max-w-xl overflow-hidden rounded-2xl">
	<div class="p-5 sm:p-7">
		<form class="space-y-3" onsubmit={submit} aria-describedby="helper">
			<label class="label">
				<span class="label-text flex items-center justify-between pe-[41px]">
					<span>Your nickname</span>
					<span>
						press <kbd class="kbd text-xs">↵</kbd> to {cta}
					</span>
				</span>

				<div class="flex gap-2">
					<input
						class="input w-full"
						type="text"
						placeholder="ChaoticWombat..."
						bind:value={name}
						autocomplete="nickname"
						disabled={busy}
						maxlength="24"
						required
						aria-invalid={!name.trim() && busy ? 'true' : 'false'}
					/>
					<button
						type="button"
						class="btn-icon preset-tonal"
						title="Surprise me!"
						aria-label="Generate a random nickname"
						onclick={() => (name = randomName())}
						disabled={busy}
					>
						<Shuffle size={18} />
					</button>
				</div>
			</label>

			<button
				type="submit"
				class="btn preset-filled-primary-500 flex w-full items-center justify-center gap-2"
				disabled={!name.trim() || busy}
				aria-busy={busy}
				aria-live="polite"
			>
				{@render buttonText({ busy })}
			</button>

			<p id="helper" class="text-on-surface-variant text-sm">Nicknames are visible to players</p>
		</form>
	</div>
</div>
