import { persisted, type Persisted } from 'svelte-persisted-store';
import type api from './api';

export interface Own {
	lobby_id: api.Uuid;
	id: api.Uuid;
	name: string;
}

export const own: Persisted<null | Own> = persisted('own', null);
