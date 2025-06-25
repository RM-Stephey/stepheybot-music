<script>
	import { onMount } from 'svelte';

	// Component props
	export let musicPlayer = null; // Reference to music player component

	// State variables
	let mounted = false;
	let searchQuery = '';
	let searchResults = [];
	let discoveryTracks = [];
	let trendingArtists = [];
	let isSearching = false;
	let isLoadingDiscovery = false;
	let searchError = null;
	let discoveryError = null;
	let activeTab = 'search'; // 'search', 'discover', 'trending'

	// Search and discovery data
	let searchTimeout = null;
	let lastSearchQuery = '';

	onMount(() => {
		mounted = true;
		loadDiscoveryData();
	});

	// Search functionality
	async function performSearch(query) {
		if (!query || query.trim().length < 2) {
			searchResults = [];
			return;
		}

		if (query === lastSearchQuery) return;
		lastSearchQuery = query;

		isSearching = true;
		searchError = null;

		try {
			const response = await fetch(`/api/v1/tracks/search/${encodeURIComponent(query)}`);
			const data = await response.json();

			if (data.success) {
				searchResults = data.tracks || [];
			} else {
				searchError = data.error || 'Search failed';
				searchResults = [];
			}
		} catch (error) {
			console.error('Search error:', error);
			searchError = 'Network error occurred';
			searchResults = [];
		} finally {
			isSearching = false;
		}
	}

	// Debounced search
	function handleSearchInput() {
		if (searchTimeout) {
			clearTimeout(searchTimeout);
		}

		searchTimeout = setTimeout(() => {
			performSearch(searchQuery);
		}, 300);
	}

	// Load discovery data
	async function loadDiscoveryData() {
		isLoadingDiscovery = true;
		discoveryError = null;

		try {
			const response = await fetch('/api/v1/discover');
			const data = await response.json();

			if (data.success) {
				discoveryTracks = data.discovery.tracks || [];
				trendingArtists = data.discovery.trending_artists || [];
			} else {
				discoveryError = data.error || 'Failed to load discovery data';
			}
		} catch (error) {
			console.error('Discovery error:', error);
			discoveryError = 'Network error occurred';
		} finally {
			isLoadingDiscovery = false;
		}
	}

	// Artist search and management
	async function searchArtist(artistName) {
		try {
			const response = await fetch(`/api/v1/lidarr/search/${encodeURIComponent(artistName)}`);
			const data = await response.json();

			if (data.success && data.results.length > 0) {
				return data.results[0]; // Return first match
			}
			return null;
		} catch (error) {
			console.error('Artist search error:', error);
			return null;
		}
	}

	async function addArtistToMonitoring(artistData) {
		try {
			const response = await fetch('/api/v1/lidarr/add', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					foreignArtistId: artistData.foreignArtistId,
					artistName: artistData.artistName,
					overview: artistData.overview,
					disambiguation: artistData.disambiguation,
					genres: artistData.genres,
					qualityProfileId: 1, // Default quality profile
					metadataProfileId: 1, // Default metadata profile
					rootFolderPath: '/music'
				})
			});

			const result = await response.json();

			if (result.success) {
				alert(`Successfully added "${artistData.artistName}" to monitoring!`);
			} else {
				alert(`Failed to add artist: ${result.error}`);
			}
		} catch (error) {
			console.error('Add artist error:', error);
			alert('Network error occurred while adding artist');
		}
	}

	// Player integration
	function playTrack(event, track) {
		event.preventDefault();
		event.stopPropagation();

		if (musicPlayer && musicPlayer.playTrackFromParent) {
			console.log('Playing track from discovery:', track.title);
			musicPlayer.playTrackFromParent(track);
		} else {
			console.warn('Music player not available for playTrack');
		}
	}

	function addToQueue(event, track) {
		event.preventDefault();
		event.stopPropagation();
		console.log('📝 Discovery: addToQueue called with:', track.title);

		if (musicPlayer && musicPlayer.addTrackToQueue) {
			console.log('Adding track to queue from discovery:', track.title);
			musicPlayer.addTrackToQueue(track);
		} else {
			console.warn('Music player not available for addToQueue');
		}
	}

	function playAllTracks(tracks) {
		if (musicPlayer && musicPlayer.setQueue && tracks.length > 0) {
			console.log('Playing all tracks from discovery, count:', tracks.length);
			musicPlayer.setQueue(tracks, 0);
		} else if (!musicPlayer) {
			console.warn('Music player not available for playAllTracks');
		} else if (tracks.length === 0) {
			console.warn('No tracks to play');
		}
	}

	// Utility functions
	function formatDuration(seconds) {
		if (!seconds) return '0:00';
		const mins = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}

	function truncateText(text, maxLength = 50) {
		if (!text) return '';
		return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
	}

	// Handle artist monitoring
	async function handleAddArtist(artistName) {
		const artistData = await searchArtist(artistName);
		if (artistData) {
			await addArtistToMonitoring(artistData);
		} else {
			alert(`Artist "${artistName}" not found in database`);
		}
	}

	// Tab switching
	function switchTab(tab) {
		activeTab = tab;
		if (tab === 'discover' && discoveryTracks.length === 0) {
			loadDiscoveryData();
		}
	}
</script>

<div class="music-discovery" class:loaded={mounted}>
	<div class="container">
		<!-- Header -->
		<div class="discovery-header">
			<h1 class="discovery-title">
				<span class="title-icon">🔍</span>
				Discover Music
			</h1>
			<p class="discovery-subtitle">
				Find new tracks, explore artists, and expand your library
			</p>
		</div>

		<!-- Tab Navigation -->
		<nav class="tab-nav">
			<button
				class="tab-btn"
				class:active={activeTab === 'search'}
				on:click={() => switchTab('search')}
			>
				<span class="tab-icon">🔍</span>
				Search
			</button>
			<button
				class="tab-btn"
				class:active={activeTab === 'discover'}
				on:click={() => switchTab('discover')}
			>
				<span class="tab-icon">🎵</span>
				Discover
			</button>
			<button
				class="tab-btn"
				class:active={activeTab === 'trending'}
				on:click={() => switchTab('trending')}
			>
				<span class="tab-icon">🔥</span>
				Trending
			</button>
		</nav>

		<!-- Search Tab -->
		{#if activeTab === 'search'}
			<div class="search-section">
				<div class="search-container">
					<div class="search-input-wrapper">
						<input
							type="text"
							placeholder="Search for tracks, artists, or albums..."
							bind:value={searchQuery}
							on:input={handleSearchInput}
							class="search-input"
						/>
						<div class="search-icon">🔍</div>
						{#if isSearching}
							<div class="search-loading">
								<div class="loading-spinner"></div>
							</div>
						{/if}
					</div>
				</div>

				<!-- Search Results -->
				<div class="search-results">
					{#if searchError}
						<div class="error-message">
							<span class="error-icon">⚠️</span>
							<p>{searchError}</p>
						</div>
					{:else if searchResults.length > 0}
						<div class="results-header">
							<h3>Search Results ({searchResults.length})</h3>
							{#if searchResults.length > 0}
								<button class="play-all-btn" on:click={() => playAllTracks(searchResults)}>
									▶️ Play All
								</button>
							{/if}
						</div>
						<div class="tracks-grid">
							{#each searchResults as track}
								<div class="track-card">
									<div class="track-info">
										<h4 class="track-title">{track.title}</h4>
										<p class="track-artist">{track.artist}</p>
										<p class="track-album">{track.album}</p>
										<span class="track-duration">{formatDuration(track.duration)}</span>
									</div>
									<div class="track-actions">
										<button class="action-btn play-btn" on:click={(event) => playTrack(event, track)} title="Play">
											▶️
										</button>
										<button class="action-btn queue-btn" on:click={(event) => addToQueue(event, track)} title="Add to Queue">
											➕
										</button>
										<button class="action-btn artist-btn" on:click={() => handleAddArtist(track.artist)} title="Monitor Artist">
											👤
										</button>
									</div>
								</div>
							{/each}
						</div>
					{:else if searchQuery.length >= 2 && !isSearching}
						<div class="no-results">
							<span class="no-results-icon">🎵</span>
							<h3>No tracks found</h3>
							<p>Try a different search term</p>
						</div>
					{:else if searchQuery.length < 2}
						<div class="search-help">
							<span class="help-icon">💡</span>
							<h3>Start searching</h3>
							<p>Enter at least 2 characters to search for music</p>
						</div>
					{/if}
				</div>
			</div>
		{/if}

		<!-- Discovery Tab -->
		{#if activeTab === 'discover'}
			<div class="discovery-section">
				{#if isLoadingDiscovery}
					<div class="loading-container">
						<div class="loading-spinner large"></div>
						<p>Loading discovery recommendations...</p>
					</div>
				{:else if discoveryError}
					<div class="error-message">
						<span class="error-icon">⚠️</span>
						<p>{discoveryError}</p>
						<button class="retry-btn" on:click={loadDiscoveryData}>
							🔄 Retry
						</button>
					</div>
				{:else}
					<div class="discovery-content">
						<div class="section-header">
							<h3>Recommended for You</h3>
							<p class="section-subtitle">Based on your listening history</p>
						</div>

						{#if discoveryTracks.length > 0}
							<div class="tracks-grid">
								{#each discoveryTracks as track}
									<div class="track-card discovery-card">
										<div class="track-info">
											<h4 class="track-title">{track.title}</h4>
											<p class="track-artist">{track.artist}</p>
											<p class="track-album">{track.album}</p>
											<span class="track-duration">{formatDuration(track.duration)}</span>
										</div>
										<div class="track-actions">
											<button class="action-btn play-btn" on:click={(event) => playTrack(event, track)} title="Play">
												▶️
											</button>
											<button class="action-btn queue-btn" on:click={(event) => addToQueue(event, track)} title="Add to Queue">
												➕
											</button>
											<button class="action-btn artist-btn" on:click={() => handleAddArtist(track.artist)} title="Monitor Artist">
												👤
											</button>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<div class="empty-discovery">
								<span class="empty-icon">🎧</span>
								<h3>No recommendations available</h3>
								<p>Start listening to music to get personalized recommendations</p>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		{/if}

		<!-- Trending Tab -->
		{#if activeTab === 'trending'}
			<div class="trending-section">
				<div class="section-header">
					<h3>Trending Artists</h3>
					<p class="section-subtitle">Popular artists in your library</p>
				</div>

				{#if trendingArtists.length > 0}
					<div class="artists-grid">
						{#each trendingArtists as artist}
							<div class="artist-card">
								<div class="artist-info">
									<h4 class="artist-name">{artist.artist_name || artist.artistName}</h4>
									<p class="artist-stats">
										{#if artist.statistics}
											{artist.statistics.album_count || 0} albums • {artist.statistics.track_count || 0} tracks
										{:else}
											Monitored artist
										{/if}
									</p>
									{#if artist.genres && artist.genres.length > 0}
										<div class="artist-genres">
											{#each artist.genres.slice(0, 3) as genre}
												<span class="genre-tag">{genre}</span>
											{/each}
										</div>
									{/if}
								</div>
								<div class="artist-actions">
									<button class="action-btn monitor-btn"
											on:click={() => handleAddArtist(artist.artist_name || artist.artistName)}
											title="Monitor Artist">
										👤 Monitor
									</button>
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<div class="empty-trending">
						<span class="empty-icon">⭐</span>
						<h3>No trending artists</h3>
						<p>Add some artists to monitoring to see trending content</p>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.music-discovery {
		opacity: 0;
		transition: opacity 0.5s ease-in-out;
		min-height: 100vh;
		padding-bottom: 200px; /* Space for music player */
	}

	.music-discovery.loaded {
		opacity: 1;
	}

	/* Header */
	.discovery-header {
		text-align: center;
		margin-bottom: var(--spacing-xl);
	}

	.discovery-title {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--spacing-sm);
		font-size: 3rem;
		font-weight: 800;
		margin-bottom: var(--spacing-md);
		background: linear-gradient(45deg, var(--neon-cyan), var(--neon-pink));
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
	}

	.title-icon {
		font-size: 2.5rem;
		filter: drop-shadow(0 0 20px var(--neon-cyan));
	}

	.discovery-subtitle {
		font-size: 1.25rem;
		color: var(--text-secondary);
		margin: 0;
	}

	/* Tab Navigation */
	.tab-nav {
		display: flex;
		justify-content: center;
		gap: var(--spacing-sm);
		margin-bottom: var(--spacing-xl);
		background: var(--bg-card);
		padding: var(--spacing-sm);
		border-radius: var(--border-radius-lg);
		border: 1px solid rgba(0, 255, 255, 0.3);
	}

	.tab-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		background: transparent;
		border: 1px solid transparent;
		color: var(--text-secondary);
		padding: var(--spacing-sm) var(--spacing-md);
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: all var(--transition-normal);
		font-family: var(--font-primary);
		font-weight: 500;
	}

	.tab-btn:hover {
		color: var(--neon-cyan);
		background: rgba(0, 255, 255, 0.1);
		border-color: var(--neon-cyan);
	}

	.tab-btn.active {
		color: var(--neon-cyan);
		background: rgba(0, 255, 255, 0.2);
		border-color: var(--neon-cyan);
		box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
	}

	.tab-icon {
		font-size: 1.1rem;
	}

	/* Search Section */
	.search-container {
		margin-bottom: var(--spacing-lg);
	}

	.search-input-wrapper {
		position: relative;
		max-width: 600px;
		margin: 0 auto;
	}

	.search-input {
		width: 100%;
		padding: var(--spacing-md) var(--spacing-lg);
		padding-left: 50px;
		font-size: 1.1rem;
		background: var(--bg-card);
		border: 2px solid rgba(0, 255, 255, 0.3);
		border-radius: var(--border-radius-lg);
		color: var(--text-primary);
		transition: all var(--transition-normal);
	}

	.search-input:focus {
		outline: none;
		border-color: var(--neon-cyan);
		box-shadow: 0 0 20px rgba(0, 255, 255, 0.3);
		background: rgba(26, 26, 46, 0.95);
	}

	.search-input::placeholder {
		color: var(--text-muted);
	}

	.search-icon {
		position: absolute;
		left: var(--spacing-md);
		top: 50%;
		transform: translateY(-50%);
		font-size: 1.2rem;
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-loading {
		position: absolute;
		right: var(--spacing-md);
		top: 50%;
		transform: translateY(-50%);
	}

	/* Results */
	.results-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--spacing-md);
	}

	.results-header h3 {
		color: var(--text-primary);
		margin: 0;
	}

	.play-all-btn {
		background: linear-gradient(45deg, var(--neon-cyan), var(--neon-pink));
		border: none;
		color: var(--bg-primary);
		padding: var(--spacing-sm) var(--spacing-md);
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: all var(--transition-normal);
		font-weight: 600;
	}

	.play-all-btn:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 20px rgba(0, 255, 255, 0.4);
	}

	/* Track Grid */
	.tracks-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: var(--spacing-md);
	}

	.track-card {
		background: var(--bg-card);
		border: 1px solid rgba(0, 255, 255, 0.2);
		border-radius: var(--border-radius-lg);
		padding: var(--spacing-md);
		transition: all var(--transition-normal);
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--spacing-md);
	}

	.track-card:hover {
		border-color: var(--neon-cyan);
		box-shadow: var(--shadow-neon);
		transform: translateY(-2px);
	}

	.discovery-card {
		border-color: rgba(255, 0, 255, 0.2);
	}

	.discovery-card:hover {
		border-color: var(--neon-pink);
		box-shadow: var(--shadow-pink);
	}

	.track-info {
		flex: 1;
		min-width: 0;
	}

	.track-title {
		font-weight: 600;
		color: var(--text-primary);
		margin: 0 0 4px 0;
		font-size: 1rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.track-artist {
		color: var(--text-secondary);
		margin: 0 0 2px 0;
		font-size: 0.875rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.track-album {
		color: var(--text-muted);
		margin: 0 0 8px 0;
		font-size: 0.8rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.track-duration {
		color: var(--text-muted);
		font-size: 0.75rem;
		font-family: var(--font-primary);
	}

	.track-actions {
		display: flex;
		gap: var(--spacing-xs);
		flex-shrink: 0;
	}

	.action-btn {
		background: transparent;
		border: 1px solid var(--neon-cyan);
		color: var(--neon-cyan);
		width: 35px;
		height: 35px;
		border-radius: 50%;
		cursor: pointer;
		transition: all var(--transition-fast);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 0.875rem;
	}

	.action-btn:hover {
		background: rgba(0, 255, 255, 0.1);
		transform: scale(1.1);
	}

	.play-btn:hover {
		background: var(--neon-cyan);
		color: var(--bg-primary);
	}

	.queue-btn {
		border-color: var(--neon-purple);
		color: var(--neon-purple);
	}

	.queue-btn:hover {
		background: var(--neon-purple);
		color: var(--bg-primary);
	}

	.artist-btn {
		border-color: var(--neon-pink);
		color: var(--neon-pink);
	}

	.artist-btn:hover {
		background: var(--neon-pink);
		color: var(--bg-primary);
	}

	/* Artist Grid */
	.artists-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: var(--spacing-md);
	}

	.artist-card {
		background: var(--bg-card);
		border: 1px solid rgba(255, 0, 255, 0.2);
		border-radius: var(--border-radius-lg);
		padding: var(--spacing-lg);
		transition: all var(--transition-normal);
	}

	.artist-card:hover {
		border-color: var(--neon-pink);
		box-shadow: var(--shadow-pink);
		transform: translateY(-2px);
	}

	.artist-name {
		font-weight: 700;
		color: var(--text-primary);
		margin: 0 0 var(--spacing-xs) 0;
		font-size: 1.1rem;
	}

	.artist-stats {
		color: var(--text-secondary);
		margin: 0 0 var(--spacing-sm) 0;
		font-size: 0.875rem;
	}

	.artist-genres {
		display: flex;
		gap: var(--spacing-xs);
		flex-wrap: wrap;
		margin-bottom: var(--spacing-md);
	}

	.genre-tag {
		background: rgba(128, 0, 255, 0.2);
		color: var(--neon-purple);
		padding: 2px 8px;
		border-radius: 12px;
		font-size: 0.75rem;
		border: 1px solid var(--neon-purple);
	}

	.monitor-btn {
		background: transparent;
		border: 1px solid var(--neon-pink);
		color: var(--neon-pink);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 0.875rem;
	}

	.monitor-btn:hover {
		background: var(--neon-pink);
		color: var(--bg-primary);
	}

	/* Loading States */
	.loading-container {
		text-align: center;
		padding: var(--spacing-xl);
		color: var(--text-secondary);
	}

	.loading-spinner {
		width: 20px;
		height: 20px;
		border: 2px solid transparent;
		border-top: 2px solid var(--neon-cyan);
		border-radius: 50%;
		animation: spin 1s linear infinite;
		margin: 0 auto var(--spacing-sm);
	}

	.loading-spinner.large {
		width: 40px;
		height: 40px;
		border-width: 3px;
	}

	/* Empty States */
	.no-results,
	.search-help,
	.empty-discovery,
	.empty-trending {
		text-align: center;
		padding: var(--spacing-xl);
		color: var(--text-muted);
	}

	.no-results-icon,
	.help-icon,
	.empty-icon {
		font-size: 3rem;
		margin-bottom: var(--spacing-md);
		filter: drop-shadow(0 0 10px currentColor);
	}

	.no-results h3,
	.search-help h3,
	.empty-discovery h3,
	.empty-trending h3 {
		color: var(--text-primary);
		margin-bottom: var(--spacing-sm);
	}

	/* Error States */
	.error-message {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-lg);
		background: rgba(255, 0, 0, 0.1);
		border: 1px solid rgba(255, 0, 0, 0.3);
		border-radius: var(--border-radius);
		color: #ff6b6b;
		text-align: center;
	}

	.error-icon {
		font-size: 1.5rem;
	}

	.retry-btn {
		background: transparent;
		border: 1px solid #ff6b6b;
		color: #ff6b6b;
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.retry-btn:hover {
		background: rgba(255, 107, 107, 0.1);
	}

	/* Section Headers */
	.section-header {
		text-align: center;
		margin-bottom: var(--spacing-lg);
	}

	.section-header h3 {
		color: var(--text-primary);
		font-size: 2rem;
		margin: 0 0 var(--spacing-xs) 0;
	}

	.section-subtitle {
		color: var(--text-secondary);
		margin: 0;
	}

	/* Animations */
	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	/* Responsive Design */
	@media (max-width: 768px) {
		.discovery-title {
			font-size: 2rem;
		}

		.title-icon {
			font-size: 1.8rem;
		}

		.tab-nav {
			flex-direction: column;
			gap: var(--spacing-xs);
		}

		.tab-btn {
			justify-content: center;
		}

		.tracks-grid {
			grid-template-columns: 1fr;
		}

		.artists-grid {
			grid-template-columns: 1fr;
		}

		.track-card {
			flex-direction: column;
			align-items: flex-start;
			gap: var(--spacing-sm);
		}

		.track-actions {
			width: 100%;
			justify-content: center;
		}

		.results-header {
			flex-direction: column;
			gap: var(--spacing-sm);
			align-items: center;
		}
	}
</style>
