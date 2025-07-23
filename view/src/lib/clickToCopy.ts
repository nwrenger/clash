import type { ActionReturn } from 'svelte/action';

interface Parameter {
	text: string | (() => string);
}

interface Attributes {
	oncopyDone: () => void;
	oncopyError: () => void;
}

/**
 * Copies the specified text to the clipboard when the element is clicked and dispatches
 * 'copy-done' or 'copy-error' custom events based on the outcome.
 */
export function clickToCopy(
	node: HTMLElement,
	{ text }: Parameter
): ActionReturn<Parameter, Attributes> {
	/**
	 * Handles the click event by attempting to copy the specified text to the clipboard.
	 * Dispatches 'copy-done' on success or 'copy-error' on failure.
	 */
	async function handleClick() {
		const textToCopy = typeof text === 'function' ? text() : text;
		try {
			await navigator.clipboard.writeText(textToCopy);
			node.dispatchEvent(new CustomEvent('copyDone'));
		} catch {
			node.dispatchEvent(new CustomEvent('copyError'));
		}
	}

	// Add the click event listener to the node
	node.addEventListener('click', handleClick);

	return {
		// Update called on updated props
		update(newProps: { text: string | (() => string) }) {
			text = newProps.text;
		},

		// Clean Up
		destroy() {
			node.removeEventListener('click', handleClick);
		}
	};
}

export default clickToCopy;
