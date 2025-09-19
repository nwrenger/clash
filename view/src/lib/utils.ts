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
 * Deterministic, perceptually balanced colors from UUIDs (OKLCH, no deps)
 * - Hash (FNV-1a 32-bit) → hue
 * - Fixed L,C in OKLCH for even-looking colors across hues
 * - Gamut-fit: scales chroma down until sRGB-safe
 * - WCAG contrast pick for text (#000 / #fff)
 */
export function colorFromUUID(uuid: string) {
	const h = ((fnv1a(uuid) % 360) + 360) % 360; // 0..359
	const L = 0.8; // perceptual lightness (0..1)
	let C = 0.12; // perceptual chroma (start; will be reduced if out-of-gamut)

	// Optional: tiny hue-based chroma trim to avoid common clip zones (purples/blues)
	C *= 0.98 - 0.04 * Math.cos(((h - 305) * Math.PI) / 180);

	const { r, g, b } = oklchToSRGBGamutFit(L, C, h);
	const bgHex = rgbToHex(Math.round(r * 255), Math.round(g * 255), Math.round(b * 255));

	// Choose readable text color
	// This is always given with white color
	const textHex = '#ffffff';

	return { background: bgHex, text: textHex };
}

function fnv1a(str: string) {
	let hash = 0x811c9dc5 >>> 0;
	for (let i = 0; i < str.length; i++) {
		hash ^= str.charCodeAt(i);
		hash = Math.imul(hash, 0x01000193);
	}
	return hash >>> 0;
}

function oklchToSRGBGamutFit(L: number, C: number, hDeg: number) {
	// First try full chroma
	let rgb = oklchToSRGB(L, C, hDeg);
	if (inSRGB(rgb)) return rgb;

	// Binary search chroma scale in [0, 1]
	let lo = 0,
		hi = 1;
	for (let i = 0; i < 18; i++) {
		const mid = (lo + hi) / 2;
		rgb = oklchToSRGB(L, C * mid, hDeg);
		if (inSRGB(rgb)) lo = mid;
		else hi = mid;
	}
	return oklchToSRGB(L, C * lo, hDeg);
}

function oklchToSRGB(L: number, C: number, hDeg: number) {
	const h = (hDeg * Math.PI) / 180;
	const a = C * Math.cos(h);
	const b = C * Math.sin(h);
	return oklabToSRGB(L, a, b);
}

function oklabToSRGB(L: number, a: number, b: number) {
	// Oklab → LMS'
	const l_ = L + 0.3963377774 * a + 0.2158037573 * b;
	const m_ = L - 0.1055613458 * a - 0.0638541728 * b;
	const s_ = L - 0.0894841775 * a - 1.291485548 * b;

	// Cube to LMS
	const l = l_ * l_ * l_;
	const m = m_ * m_ * m_;
	const s = s_ * s_ * s_;

	// LMS → linear sRGB
	let r = +4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
	let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
	let b2 = -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s;

	// Linear → gamma-encoded sRGB
	r = compandSRGB(r);
	g = compandSRGB(g);
	b2 = compandSRGB(b2);

	return { r, g, b: b2 };
}

function compandSRGB(u: number) {
	// clamp linear to a safe range before gamma
	if (u <= 0.0031308) return 12.92 * Math.max(0, u);
	return 1.055 * Math.pow(Math.max(0, u), 1 / 2.4) - 0.055;
}

function inSRGB({ r, g, b }: { r: number; g: number; b: number }) {
	return r >= 0 && r <= 1 && g >= 0 && g <= 1 && b >= 0 && b <= 1;
}

function rgbToHex(r: number, g: number, b: number) {
	const h = (n: number) => Math.max(0, Math.min(255, n)).toString(16).padStart(2, '0');
	return `#${h(r)}${h(g)}${h(b)}`;
}

export function relativeTime(tsSec: number) {
	const d = new Date(tsSec * 1000);
	const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });
	const diffMs = d.getTime() - Date.now();

	const minute = 60_000;
	const hour = 60 * minute;
	const day = 24 * hour;
	const year = 365.25 * day;

	const minutes = Math.round(diffMs / minute);
	const hours = Math.round(diffMs / hour);
	const days = Math.round(diffMs / day);
	const years = Math.round(diffMs / year);

	if (Math.abs(minutes) < 60) return rtf.format(minutes, 'minute');
	if (Math.abs(hours) < 24) return rtf.format(hours, 'hour');
	if (Math.abs(days) < 365) return rtf.format(days, 'day');
	return rtf.format(years, 'year');
}
