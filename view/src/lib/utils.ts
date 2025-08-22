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
	return arr.sort(([keyA, valA], [keyB, valB]) => {
		if (valA.name == valB.name) {
			return keyA.localeCompare(keyB);
		} else {
			return valA.name.localeCompare(valB.name);
		}
	});
}

/**
 * Deterministic colors from UUIDs
 * - Uses FNV-1a (32-bit) to hash the UUID
 * - Maps hash -> HSL for stable, vibrant backgrounds
 * - Picks #000 or #fff text for best contrast
 */
export function colorFromUUID(uuid: api.Uuid) {
	const hash = fnv1a(uuid);

	// Map hash -> HSL (keep S/L in friendly ranges)
	const h = hash % 360; // 0..359
	const s = 60 + ((hash >>> 1) % 20); // 60..79%
	const l = 45 + ((hash >>> 3) % 20); // 45..64%

	const { r, g, b } = hslToRgb(h, s / 100, l / 100);
	const bgHex = rgbToHex(r, g, b);

	// Choose readable text color
	// This is always given with white color
	const textHex = '#ffffff';

	return { background: bgHex, text: textHex };
}

export function textColorFromUUID(uuid: api.Uuid) {
	return colorFromUUID(uuid).text;
}

function fnv1a(str: api.Uuid) {
	// 32-bit FNV-1a hash
	let hash = 0x811c9dc5;
	for (let i = 0; i < str.length; i++) {
		hash ^= str.charCodeAt(i);
		hash = Math.imul(hash, 0x01000193);
	}
	return hash >>> 0; // unsigned
}

function hslToRgb(h: number, s: number, l: number) {
	const c = (1 - Math.abs(2 * l - 1)) * s;
	const hp = h / 60;
	const x = c * (1 - Math.abs((hp % 2) - 1));
	let r1 = 0,
		g1 = 0,
		b1 = 0;

	if (0 <= hp && hp < 1) [r1, g1, b1] = [c, x, 0];
	else if (1 <= hp && hp < 2) [r1, g1, b1] = [x, c, 0];
	else if (2 <= hp && hp < 3) [r1, g1, b1] = [0, c, x];
	else if (3 <= hp && hp < 4) [r1, g1, b1] = [0, x, c];
	else if (4 <= hp && hp < 5) [r1, g1, b1] = [x, 0, c];
	else if (5 <= hp && hp < 6) [r1, g1, b1] = [c, 0, x];

	const m = l - c / 2;
	return {
		r: Math.round((r1 + m) * 255),
		g: Math.round((g1 + m) * 255),
		b: Math.round((b1 + m) * 255)
	};
}

function rgbToHex(r: number, g: number, b: number) {
	const toHex = (n: number) => n.toString(16).padStart(2, '0');
	return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}
