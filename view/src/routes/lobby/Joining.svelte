<script lang="ts">
	import { randomName } from '$lib/utils';
	import { LogIn, Shuffle } from 'lucide-svelte';
	import LoginCard from '../LoginCard.svelte';

	interface Props {
		own_name: string;
		join_lobby: (e: Event) => void;
	}

	let { own_name = $bindable(), join_lobby }: Props = $props();
</script>

<div class="mx-auto flex max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	<LoginCard bind:name={own_name} cta="join" submit={join_lobby}>
		{#snippet buttonText()}
			<LogIn size={20} />
			Join lobby
		{/snippet}
	</LoginCard>
</div>
