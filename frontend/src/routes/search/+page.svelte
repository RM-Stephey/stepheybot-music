<script>
	import { onMount } from 'svelte';
	import GlobalSearch from '$lib/components/GlobalSearch.svelte';

	let mounted = false;

	onMount(() => {
		mounted = true;
	});
</script>

<svelte:head>
	<title>Global Search - StepheyBot Music</title>
	<meta name="description" content="Search for music across your library and external sources. Discover new tracks and request downloads with StepheyBot Music's global search." />
</svelte:head>

<div class="search-page" class:loaded={mounted}>
	<div class="container">
		<GlobalSearch />
	</div>
</div>

<style>
	.search-page {
		opacity: 0;
		transition: opacity 0.5s ease-in-out;
		min-height: 100vh;
		padding: var(--spacing-lg) 0;
		background: linear-gradient(135deg, rgba(10, 10, 15, 0.95), rgba(20, 20, 30, 0.95));
	}

	.search-page.loaded {
		opacity: 1;
	}

	.container {
		max-width: 1400px;
		margin: 0 auto;
		padding: 0 var(--spacing-md);
	}

	/* Ensure proper spacing for floating music player */
	.search-page {
		padding-bottom: 100px;
	}

	/* Responsive adjustments */
	@media (max-width: 768px) {
		.search-page {
			padding: var(--spacing-md) 0 120px 0;
		}

		.container {
			padding: 0 var(--spacing-sm);
		}
	}

	/* Add some visual enhancements */
	.search-page::before {
		content: '';
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background:
			radial-gradient(circle at 20% 80%, rgba(0, 255, 255, 0.1) 0%, transparent 50%),
			radial-gradient(circle at 80% 20%, rgba(255, 107, 157, 0.1) 0%, transparent 50%);
		pointer-events: none;
		z-index: -1;
	}
</style>
