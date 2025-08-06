import { persisted, type Persisted } from 'svelte-persisted-store';
import type api from './api';

export interface Credentials {
	lobby_id: api.Uuid;
	id: api.Uuid;
	name: string;
}

export const credentials: Persisted<null | Credentials> = persisted('credentials', null, {
	storage: 'session'
});
