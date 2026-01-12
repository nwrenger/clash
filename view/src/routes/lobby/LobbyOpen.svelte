<script lang="ts" module>
	export interface Shared {
		saving: boolean;
		valid_config: boolean;
	}
</script>

<script lang="ts">
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import { House, Settings2 } from 'lucide-svelte';
	import LobbyPanel from './LobbyPanel.svelte';
	import SettingsPanel from './SettingsPanel.svelte';
	import type { Connection, Lobby, Own } from './+page.svelte';

	interface Props {
		connection: Connection;
		lobby: Lobby;
		own: Own;
		disconnect: () => void;
	}

	let { connection, lobby, own, disconnect }: Props = $props();

	let shared: Shared = $state({
		saving: false,
		valid_config: false
	});
</script>

<div class="mx-auto flex h-full max-w-3xl flex-col items-center overflow-y-hidden px-4 pt-8">
	<Tabs class="h-full" defaultValue="lobby">
		<Tabs.List class="preset-tonal mb-0! rounded-md px-2 pt-2 whitespace-nowrap backdrop-blur-lg">
			<Tabs.Trigger
				class="text-surface-950-50 flex-1 bg-transparent brightness-100 hover:opacity-75"
				value="lobby"
			>
				<House size="18" />
				<span>Lobby</span>
			</Tabs.Trigger>
			<Tabs.Trigger
				class="text-surface-950-50 flex-1 bg-transparent brightness-100 hover:opacity-75"
				value="settings"
			>
				<Settings2 size="18" />
				<span>Settings</span>
			</Tabs.Trigger>
			<Tabs.Indicator />
		</Tabs.List>
		<div class="h-[calc(100%-53px)]">
			<Tabs.Content class="h-full" value="lobby">
				<LobbyPanel {connection} {lobby} {own} {disconnect} {shared} />
			</Tabs.Content>
			<Tabs.Content class="h-full" value="settings">
				<SettingsPanel {connection} {lobby} {own} bind:shared />
			</Tabs.Content>
		</div>
	</Tabs>
</div>
