import { persisted, type Persisted } from 'svelte-persisted-store';
import type api from './api';

export interface Session {
	lobby_id: api.Uuid;
	credentials: api.Credentials;
}

export const session: Persisted<null | Session> = persisted('session', null, {
	storage: 'session'
});
