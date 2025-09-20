<script lang="ts">
	import { Tooltip } from '@skeletonlabs/skeleton-svelte';
	import { CircleQuestionMark, Icon } from 'lucide-svelte';

	interface Props {
		description: string;
		preset?: string;
		icon?: typeof Icon;
	}

	let { description, preset = 'preset-filled', icon = CircleQuestionMark }: Props = $props();

	let open = $state(false);
</script>

<Tooltip
	{open}
	onOpenChange={(e) => (open = e.open)}
	positioning={{ placement: 'top' }}
	base="flex items-center"
	triggerBase="px-1"
	contentBase="card {preset} text-center p-4 w-[calc(100vw-50px)] max-w-80"
	openDelay={50}
	closeDelay={50}
	closeOnClick={false}
	closeOnPointerDown={false}
	onclick={() => (open = !open)}
>
	{#snippet trigger()}
		{@const Icon = icon}
		<Icon size={14} strokeWidth="2px" />
	{/snippet}
	{#snippet content()}{@html description}{/snippet}
</Tooltip>
