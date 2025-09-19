<script lang="ts">
	import { Tooltip } from '@skeletonlabs/skeleton-svelte';
	import { CircleQuestionMark } from 'lucide-svelte';

	interface Props {
		description: string;
	}

	let { description }: Props = $props();

	let open = $state(false);
</script>

<Tooltip
	{open}
	onOpenChange={(e) => (open = e.open)}
	positioning={{ placement: 'top' }}
	base="flex items-center"
	triggerBase="px-1"
	contentBase="card preset-filled text-center p-4 max-w-[calc(100vw-16px)]"
	openDelay={0}
	closeDelay={50}
	closeOnClick={false}
	closeOnPointerDown={false}
	onclick={() => (open = !open)}
>
	{#snippet trigger()}
		<CircleQuestionMark size={14} strokeWidth="2px" />
	{/snippet}
	{#snippet content()}{@html description}{/snippet}
</Tooltip>
