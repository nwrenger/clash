<script lang="ts">
	interface Props {
		update?: (value?: number) => void;
		[key: string]: any;
	}

	let { update, ...restProps }: Props = $props();

	function handleInput(e: Event) {
		let target = e.target as HTMLInputElement;
		// Only allow integers and remove leading zeros
		let digits: string = target.value.replace(/[^0-9]/g, '').replace(/^0+/, '');
		// Cap at Rust u32 max
		if (digits.length > 10) {
			digits = digits.slice(0, 10);
		}
		const U32_MAX_STR = '4294967295';
		if (digits.length === 10 && digits > U32_MAX_STR) {
			digits = U32_MAX_STR;
		}
		// Update value and display
		let parsedInt: number | undefined = parseInt(digits);
		if (update) update(parsedInt);
		target.value = digits;
	}
</script>

<input
	class="input"
	type="text"
	inputmode="numeric"
	autocomplete="off"
	oninput={handleInput}
	{...restProps}
/>
