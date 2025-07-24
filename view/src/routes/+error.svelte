<script lang="ts">
	import { page } from '$app/state';
</script>

<div class="mx-auto flex w-full max-w-3xl flex-col items-center space-y-6 px-4 py-8">
	<h3 class="h3 text-center">
		{page.status} - {page.error?.message || 'Unknown Error'}
	</h3>
</div>
