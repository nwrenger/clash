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
		| { kind: 'WebSocket'; value: string }
		| { kind: 'FileSystem'; value: string }
		| { kind: 'Json'; value: string };

	export type Uuid = `${string}-${string}-${string}-${string}-${string}`;

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

	export async function create_lobby(name: string, id: string): Promise<LobbyId> {
		return fetch_api(`${API_BASE}/lobby`, {
			method: 'POST',
			body: JSON.stringify({ name, id })
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
		max_rounds: number;
		max_submitting_time_secs?: number;
		max_judging_time_secs?: number;
		wait_time_secs?: number;
		max_players: number;
		decks: DeckInfo[];
	}

	export interface DeckInfo {
		name: string;
		deckcode: string;
		enabled: boolean;
	}

	export interface PlayerInfo {
		name: string;
		is_host: boolean;
		is_czar: boolean;
		points: number;
	}

	export interface LobbyState {
		players: Record<Uuid, PlayerInfo>;
		settings: Settings;
		phase: GamePhase;
	}

	export type GamePhase = 'LobbyOpen' | 'Submitting' | 'Judging' | 'RoundFinished' | 'GameOver';

	export type ClientEvent =
		| { type: 'JoinLobby'; data: { name: string; id: Uuid } }
		| { type: 'UpdateSettings'; data: { settings: Settings } }
		| { type: 'AddDeck'; data: { deckcode: String } }
		| { type: 'Kick'; data: { kicked: Uuid } }
		| { type: 'StartRound' }
		| { type: 'RestartRound' }
		| { type: 'SubmitOwnCards'; data: { indexes: number[] } }
		| { type: 'CzarPick'; data: { index: number } };

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
				type: 'LobbyState';
				data: LobbyState;
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
