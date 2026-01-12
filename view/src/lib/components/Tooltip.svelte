<script lang="ts">
	import { opacity } from '$lib/animations';
	import { Portal, Tooltip } from '@skeletonlabs/skeleton-svelte';
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
	openDelay={50}
	closeDelay={50}
	closeOnClick={false}
	closeOnPointerDown={false}
>
	<Tooltip.Trigger onclick={() => (open = !open)} class="px-1">
		{@const Icon = icon}
		<Icon size={14} strokeWidth="2px" />
	</Tooltip.Trigger>
	<Portal>
		<Tooltip.Positioner>
			<Tooltip.Content
				class="card {preset} w-[calc(100vw-50px)] max-w-80 p-4 text-center text-xs {opacity}"
			>
				{description}
			</Tooltip.Content>
		</Tooltip.Positioner>
	</Portal>
</Tooltip>
