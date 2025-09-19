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

	let tabs = $state('lobby');

	let shared: Shared = $state({
		saving: false,
		valid_config: false
	});
</script>

<div class="mx-auto flex h-full max-w-3xl flex-col items-center overflow-y-hidden px-4 pt-8">
	<Tabs
		value={tabs}
		onValueChange={(e) => (tabs = e.value)}
		fluid
		composite
		classes="h-full"
		contentClasses="h-[calc(100%-53px)]"
		listClasses="preset-tonal backdrop-blur-lg pt-2 px-2 !mb-0 rounded-md whitespace-nowrap"
	>
		{#snippet list()}
			<Tabs.Control value="lobby" labelBase="btn hover:filter-none!">
				{#snippet lead()}<House size="18" />{/snippet}
				<span>Lobby</span>
			</Tabs.Control>
			<Tabs.Control value="settings" labelBase="btn hover:filter-none!">
				{#snippet lead()}<Settings2 size="18" />{/snippet}
				<span>Settings</span>
			</Tabs.Control>
		{/snippet}
		{#snippet content()}
			<Tabs.Panel classes="h-full" value="lobby">
				<LobbyPanel {connection} {lobby} {own} {disconnect} {shared} />
			</Tabs.Panel>
			<Tabs.Panel classes="h-full" value="settings">
				<SettingsPanel {connection} {lobby} {own} bind:shared />
			</Tabs.Panel>
		{/snippet}
	</Tabs>
</div>
