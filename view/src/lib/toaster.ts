import type api from './api';
import { createToaster } from '@skeletonlabs/skeleton-svelte';
export const toaster = createToaster({ placement: 'bottom-end' });

/** Gets `T` of `api.Result<T>` if no error occurred otherwise displays the error via a toast and throws the error */
export async function handle_promise<T>(
	promise: Promise<T>,
	on_error?: () => void
): Promise<T> | never {
	try {
		return await promise;
	} catch (e: unknown) {
		let error = e as api.Error;

		error_toast(error_msg(error));

		if (on_error) on_error();

		throw error.kind;
	}
}

/** Displays the error via a toast and throws it */
export function show_error(error: api.Error): never {
	error_toast(error_msg(error));
	throw error.kind;
}

export interface Options {
	title: string;
	description: string;
}

/** Displays a error toast with improved configuration */
export function error_toast(opt: Options) {
	toaster.error(opt);
}

/** Server Error translations */
function error_msg(error: api.Error): Options {
	switch (error.kind) {
		case 'LobbyLogin':
			return {
				title: 'Lobby couldn’t be joined',
				description: `You’re trying to join a lobby with malformed data. Try reloading the Website!`
			};
		case 'LobbyClosed':
			return {
				title: 'Lobby Is Closed',
				description: `The lobby you're trying to join is closed. A game is currently going on.`
			};
		case 'LobbyFull':
			return {
				title: 'Lobby Already Full',
				description: `The lobby you're trying to join is already full.`
			};
		case 'LobbyStart':
			return {
				title: `Lobby couldn’t be started`,
				description: `We couldn’t start the lobby because your configuration, or having only one player in the lobby, would leave the game in an unplayable state.`
			};
		case 'LobbyNotFound':
			return {
				title: 'Lobby Not Found',
				description: `The lobby you're trying to join couldn't be found.`
			};
		case 'CardSubmission':
			return {
				title: 'Card Submission',
				description: `The card couln't be submitted. This might be happening due to a Game Phase missmatch.`
			};
		case 'CzarChoice':
			return {
				title: 'Czar Choice',
				description: `Your choice as a Czar couln't be submitted. This might be happening due to a Game Phase missmatch.`
			};
		case 'Unauthorized':
			return {
				title: 'Authorization Error',
				description: `You're not authorized to due that action.`
			};
		case 'Deck':
			return { title: 'Deck Error', description: error.value };
		case 'Reqwest':
			return { title: 'Third Party Request Error', description: error.value };
		case 'FileSystem':
			return { title: 'File System Error', description: error.value };
		case 'Json':
			return { title: 'Json Serialzing/Desializing Error', description: error.value };
		default:
			return {
				title: 'Fatal Frontend Error',
				description: `An unknown Error has occurred. This could be happening due the backend being down. Try reloading the Website!`
			};
	}
}
