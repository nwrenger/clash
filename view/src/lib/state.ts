import { persistedState } from 'svelte-persisted-state';
import type api from './api';

export interface Session {
	lobby_id: api.Uuid;
	credentials: api.Credentials;
}

export const session = persistedState<Session | null>('session', null, {
	storage: 'session'
});
