<script>
	import { onMount } from 'svelte';
	import MusicDiscovery from '$lib/components/MusicDiscovery.svelte';
	import { musicPlayerActions } from '$lib/stores/musicPlayer.js';

	let mounted = false;
	let musicPlayer = null;

	onMount(() => {
		mounted = true;
	});

	// Create a wrapper that provides the same interface as the music player component
	// but uses musicPlayerActions under the hood
	$: musicPlayer = {
		playTrackFromParent: (track) => {
			if (musicPlayerActions && musicPlayerActions.playTrack) {
				musicPlayerActions.playTrack(track);
			} else {
				console.error('Music player not available for playTrack');
			}
		},
		addTrackToQueue: (track) => {
			if (musicPlayerActions && musicPlayerActions.addToQueue) {
				musicPlayerActions.addToQueue(track);
			} else {
				console.error('Music player not available for addToQueue');
			}
		},
		setQueue: (tracks, startIndex = 0) => {
			if (musicPlayerActions && musicPlayerActions.setQueue) {
				musicPlayerActions.setQueue(tracks, startIndex);
			} else {
				console.error('Music player not available for setQueue');
			}
		}
	};
</script>

<svelte:head>
	<title>Discover Music - StepheyBot Music</title>
	<meta name="description" content="Discover new music, search tracks, and explore trending artists with StepheyBot Music's AI-powered recommendations." />
</svelte:head>

<div class="discover-page" class:loaded={mounted}>
	<MusicDiscovery {musicPlayer} />
</div>

<style>
	.discover-page {
		opacity: 0;
		transition: opacity 0.5s ease-in-out;
		min-height: 100vh;
		padding: var(--spacing-lg) 0;
	}

	.discover-page.loaded {
		opacity: 1;
	}

	/* Ensure proper spacing for fixed music player */
	.discover-page {
		padding-bottom: 150px;
	}

	/* Responsive adjustments */
	@media (max-width: 768px) {
		.discover-page {
			padding: var(--spacing-md) 0 180px 0;
		}
	}
</style>
