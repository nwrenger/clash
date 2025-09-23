import { API_BASE } from './utils';

namespace api {
	export type Error =
		| { kind: 'LobbyLogin' }
		| { kind: 'LobbyClosed' }
		| { kind: 'LobbyFull' }
		| { kind: 'LobbyStart' }
		| { kind: 'LobbyNotFound' }
		| { kind: 'CardSubmission' }
		| { kind: 'CzarChoice' }
		| { kind: 'Unauthorized' }
		| { kind: 'Deck'; value: string }
		| { kind: 'Reqwest'; value: string }
		| { kind: 'FileSystem'; value: string }
		| { kind: 'Json'; value: string };

	export type Uuid = `${string}-${string}-${string}-${string}-${string}`;

	export interface Credentials {
		name: string;
		id: Uuid;
		secret: Uuid;
	}

	export interface LobbyId {
		id: Uuid;
	}

	async function fetch_api(endpoint: string, options: RequestInit): Promise<any> {
		const response = await fetch(endpoint, {
			headers: {
				'Content-Type': 'application/json',
				...options.headers
			},
			...options
		});
		if (response.headers.get('Content-Length') === '0') {
			return;
		} else {
			return response.json();
		}
	}

	// === REST call ===

	export async function create_lobby(host: Credentials): Promise<LobbyId> {
		return fetch_api(`${API_BASE}/lobby`, {
			method: 'POST',
			body: JSON.stringify(host)
		});
	}

	// === Types for WS protocol ===

	export interface WhiteCard {
		text: string;
	}

	export interface BlackCard {
		text: string;
		fields: number;
	}

	export interface Settings {
		max_rounds: number | null;
		max_points: number | null;
		max_submitting_time_secs: Scaling | null;
		max_judging_time_secs: number | null;
		wait_time_secs: number | null;
		max_players: number;
		decks: DeckInfo[];
	}

	export type Scaling = { type: 'Player'; seconds: number } | { type: 'Constant'; seconds: number };

	export interface DeckInfo {
		meta: DeckMeta;
		enabled: boolean;
	}

	export interface DeckMeta {
		name: string;
		deckcode: string;
		language: string;
		nsfw: boolean;
		blacks_count: number;
		whites_count: number;
		fetched_at: number;
	}

	export interface PlayerInfo {
		name: string;
		is_host: boolean;
		is_czar: boolean;
		points: number;
	}

	export interface ClientLobby {
		players: Record<Uuid, PlayerInfo>;
		settings: Settings;
		phase: GamePhase;
		round: number;
		hand?: WhiteCard[];
		revealed_cards?: api.WhiteCard[][];
		submitted_players?: Uuid[];
		selected_cards?: number[];
		czar_pick?: number;
		winner?: Uuid;
		black_card?: BlackCard;
	}

	export type GamePhase = 'LobbyOpen' | 'Submitting' | 'Judging' | 'RoundFinished' | 'GameOver';

	export type ClientEvent =
		| { type: 'JoinLobby'; data: { credentials: Credentials } }
		| { type: 'UpdateSettings'; data: { settings: Settings } }
		| { type: 'AddDeck'; data: { deckcode: String } }
		| { type: 'FetchDecks' }
		| { type: 'Kick'; data: { kicked: Uuid } }
		| { type: 'EndGame' }
		| { type: 'StartRound' }
		| { type: 'RestartRound' }
		| { type: 'SubmitOwnCards'; data: { indexes: number[] } }
		| { type: 'CzarPick'; data: { index: number } }
		| { type: 'LeaveLobby' };

	export type ServerEvent =
		| { type: 'PlayerJoin'; data: { player_id: Uuid; player_info: PlayerInfo } }
		| { type: 'PlayerRemove'; data: { player_id: Uuid } }
		| { type: 'AssignHost'; data: { player_id: Uuid } }
		| { type: 'StartRound'; data: { czar_id: Uuid; black_card: BlackCard } }
		| { type: 'CardsSubmitted'; data: { player_id: Uuid } }
		| { type: 'UpdateDecks'; data: { decks: DeckInfo[] } }
		| { type: 'UpdateSettings'; data: { settings: Settings } }
		| { type: 'RevealCards'; data: { selected_cards: WhiteCard[][] } }
		| { type: 'RoundSkip' }
		| { type: 'RoundResult'; data: { player_id: Uuid; winning_card_index: number } }
		| { type: 'GameOver' }
		| { type: 'LobbyReset' };

	export type PrivateServerEvent =
		| {
				type: 'ClientLobby';
				data: ClientLobby;
		  }
		| { type: 'UpdateHand'; data: { cards: WhiteCard[] } }
		| { type: 'Timeout' }
		| { type: 'Kick' }
		| { type: 'Error'; data: api.Error };

	export type IncommingEvent = ServerEvent | PrivateServerEvent;

	export function send_ws(ws: WebSocket, message: ClientEvent) {
		ws.send(JSON.stringify(message));
	}

	export function connect_ws(lobbyId: Uuid): WebSocket {
		return new WebSocket(`${API_BASE}/ws/${lobbyId}`);
	}
}

export default api;
