import { dev } from '$app/environment';
import type api from './api';

export const API_BASE = dev ? 'https://localhost:8080' : 'https://api.clash.nwrenger.dev';

/** Function deep-cloning objects with arrays */
export function deepClone(obj: any): any {
	if (obj === null || typeof obj !== 'object') return obj;

	if (Array.isArray(obj)) {
		return obj.map((item) => deepClone(item));
	}

	const clone = {} as any;
	for (const key in obj) {
		clone[key] = deepClone(obj[key]);
	}
	return clone;
}

/** Helper for checking if objects are really equal */
export function areObjectsEqual(obj1: any, obj2: any): boolean {
	if (Array.isArray(obj1) && Array.isArray(obj2)) {
		if (obj1.length !== obj2.length) return false;
		for (let i = 0; i < obj1.length; i++) {
			if (!areObjectsEqual(obj1[i], obj2[i])) return false;
		}
		return true;
	} else if (
		typeof obj1 !== 'object' ||
		typeof obj2 !== 'object' ||
		obj1 === null ||
		obj2 === null
	) {
		return obj1 === obj2;
	}

	const keys1 = Object.keys(obj1);
	const keys2 = Object.keys(obj2);

	if (keys1.length !== keys2.length) {
		return false;
	}

	for (const key of keys1) {
		if (!areObjectsEqual(obj1[key], obj2[key])) {
			return false;
		}
	}

	return true;
}

/** Making records into sorted arrays by their key */
export function sortedEntries<K extends string, V extends api.PlayerInfo>(
	rec: Record<K, V> | undefined
): [K, V][] {
	let arr = Object.entries(rec as any) as [K, V][];
	return arr.sort(([_keyA, valA], [_keyB, valB]) => valA.name.localeCompare(valA.name));
}
