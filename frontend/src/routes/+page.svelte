<script>
	// Basic test - this should always show if JavaScript is working
	console.log('🚀 BASIC TEST: JavaScript is executing in +page.svelte');
	console.log('🚀 BASIC TEST: Window available:', typeof window !== 'undefined');
	console.log('🚀 BASIC TEST: Document available:', typeof document !== 'undefined');

	import { onMount } from 'svelte';
	import { musicPlayerActions } from '$lib/stores/musicPlayer.js';

	// Debug: Check if import worked
	console.log('🔧 Debug: musicPlayerActions import result:', musicPlayerActions);
	console.log('🔧 Debug: Import type:', typeof musicPlayerActions);
	console.log('🔧 Debug: Available methods:', musicPlayerActions ? Object.keys(musicPlayerActions) : 'None');

	// Export to window for debugging
	if (typeof window !== 'undefined') {
		window['musicPlayerActions'] = musicPlayerActions;
		window['debugInfo'] = {
			musicPlayerActions,
			importWorked: !!musicPlayerActions,
			availableMethods: musicPlayerActions ? Object.keys(musicPlayerActions) : []
		};
		console.log('🔧 Debug: Exported to window.musicPlayerActions and window.debugInfo');
	}

	// State variables
	let mounted = false;
	let systemStats = null;
	let recentRecommendations = [];
	let isLoading = true;
	let error = null;



	// API functions
	async function fetchSystemStats() {
		try {
			const response = await fetch('/api/v1/library/stats');
			if (!response.ok) throw new Error('Failed to fetch stats');
			return await response.json();
		} catch (err) {
			console.error('Error fetching system stats:', err);
			throw err;
		}
	}

	async function fetchRecentRecommendations() {
		try {
			const response = await fetch('/api/v1/recommendations/user1?limit=6');
			if (!response.ok) throw new Error('Failed to fetch recommendations');
			const data = await response.json();
			return data.recommendations || [];
		} catch (err) {
			console.error('Error fetching recommendations:', err);
			throw err;
		}
	}

	async function generateQuickPlaylist() {
		try {
			const response = await fetch('/api/v1/playlists/generate', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					name: 'Quick Mix',
					description: 'Auto-generated playlist for your current mood',
					duration_minutes: 30
				})
			});

			if (!response.ok) throw new Error('Failed to generate playlist');
			const playlist = await response.json();

			// Show success message or navigate to playlist
			alert(`Generated playlist: ${playlist.name} with ${playlist.tracks?.length || 0} tracks!`);
		} catch (err) {
			console.error('Error generating playlist:', err);
			alert('Failed to generate playlist. Please try again.');
		}
	}

	// Initialize data
	onMount(async () => {
		console.log('🚀 Dashboard: Component mounting...');
		mounted = true;

		// Debug music player store availability
		console.log('🔍 Dashboard: Checking musicPlayerActions:', {
			available: !!musicPlayerActions,
			playTrack: !!musicPlayerActions?.playTrack,
			addToQueue: !!musicPlayerActions?.addToQueue,
			setQueue: !!musicPlayerActions?.setQueue
		});

		try {
			console.log('📊 Dashboard: Fetching data...');
			const [stats, recommendations] = await Promise.all([
				fetchSystemStats(),
				fetchRecentRecommendations()
			]);

			systemStats = stats;
			recentRecommendations = recommendations;
			console.log('✅ Dashboard: Data loaded successfully:', {
				stats: !!stats,
				recommendationsCount: recommendations.length
			});
		} catch (err) {
			console.error('❌ Dashboard: Error loading data:', err);
			error = err.message;
		} finally {
			isLoading = false;
			console.log('🏁 Dashboard: Component mount complete');
		}

		// Debug: Export current state to window after mount
		if (typeof window !== 'undefined') {
			window['dashboardState'] = {
				mounted,
				systemStats,
				recentRecommendations,
				isLoading,
				error,
				playTrack,
				addToQueue,
				playAllRecommendations
			};
			console.log('🔧 Debug: Dashboard state exported to window.dashboardState');
		}
	});

	// Music player functions
	function playTrack(track) {
		console.log('🎵 Dashboard: playTrack button clicked!');
		console.log('🎵 Dashboard: Track details:', {
			title: track.title,
			artist: track.artist,
			stream_url: track.stream_url,
			track_id: track.track_id
		});
		console.log('🎵 Dashboard: musicPlayerActions available:', !!musicPlayerActions);
		console.log('🎵 Dashboard: musicPlayerActions type:', typeof musicPlayerActions);
		console.log('🎵 Dashboard: musicPlayerActions content:', musicPlayerActions);

		if (!musicPlayerActions) {
			console.error('❌ Dashboard: musicPlayerActions is undefined/null');
			alert('Music player not available - check console for details');
			return;
		}

		if (!musicPlayerActions.playTrack) {
			console.error('❌ Dashboard: musicPlayerActions.playTrack method missing');
			alert('Music player playTrack method not available');
			return;
		}

		console.log('🎵 Dashboard: Calling musicPlayerActions.playTrack...');
		try {
			musicPlayerActions.playTrack(track);
			console.log('✅ Dashboard: musicPlayerActions.playTrack called successfully');
		} catch (error) {
			console.error('❌ Dashboard: Error calling musicPlayerActions.playTrack:', error);
			alert('Error playing track: ' + error.message);
		}
	}

	function addToQueue(track) {
		console.log('📝 Dashboard: addToQueue button clicked!');
		console.log('📝 Dashboard: Track details:', {
			title: track.title,
			artist: track.artist,
			stream_url: track.stream_url
		});
		console.log('📝 Dashboard: musicPlayerActions available:', !!musicPlayerActions);

		if (!musicPlayerActions || !musicPlayerActions.addToQueue) {
			console.error('❌ Dashboard: musicPlayerActions.addToQueue not available');
			alert('Add to queue function not available');
			return;
		}

		try {
			musicPlayerActions.addToQueue(track);
			console.log('✅ Dashboard: musicPlayerActions.addToQueue called successfully');
		} catch (error) {
			console.error('❌ Dashboard: Error calling musicPlayerActions.addToQueue:', error);
			alert('Error adding to queue: ' + error.message);
		}
	}

	function playAllRecommendations() {
		console.log('🎼 Dashboard: playAllRecommendations button clicked!');
		console.log('🎼 Dashboard: Recommendations count:', recentRecommendations.length);

		if (recentRecommendations.length > 0) {
			console.log('🎼 Dashboard: Setting queue with recommendations:', recentRecommendations.map(r => ({ title: r.title, artist: r.artist })));

			try {
				musicPlayerActions.setQueue(recentRecommendations, 0);
				console.log('✅ Dashboard: musicPlayerActions.setQueue called successfully');
			} catch (error) {
				console.error('❌ Dashboard: Error calling musicPlayerActions.setQueue:', error);
			}
		} else {
			console.warn('⚠️ Dashboard: No recommendations available to play');
		}
	}

	// Utility functions
	function formatNumber(num) {
		return new Intl.NumberFormat().format(num);
	}

	function formatDuration(seconds) {
		const minutes = Math.floor(seconds / 60);
		const remainingSeconds = seconds % 60;
		return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
	}
</script>

<svelte:head>
	<title>Dashboard - StepheyBot Music</title>
</svelte:head>

<div class="dashboard" class:loaded={mounted}>
	<div class="container">
		<!-- Hero Section -->
		<section class="hero">
			<div class="hero-content">
				<h1 class="hero-title">
					Welcome to the <span class="text-neon-cyan">Future</span> of Music
				</h1>
				<p class="hero-subtitle">
					AI-powered recommendations tailored just for you, <span class="text-neon-pink">Stephey</span>
				</p>
				<div class="hero-actions">
					<button class="btn btn-primary" on:click={generateQuickPlaylist}>
						<span>🎵</span>
						Generate Quick Mix
					</button>
					<a href="/recommendations" class="btn btn-secondary">
						<span>🎧</span>
						Explore Recommendations
					</a>
				</div>
			</div>
			<div class="hero-visual">
				<div class="pulse-ring pulse-ring-1"></div>
				<div class="pulse-ring pulse-ring-2"></div>
				<div class="pulse-ring pulse-ring-3"></div>
				<div class="hero-icon">🤖</div>
			</div>
		</section>

		<!-- Quick Stats -->
		<section class="stats-section">
			<h2 class="section-title">System Overview</h2>
			{#if isLoading}
				<div class="stats-grid">
					{#each Array(4) as _}
						<div class="stat-card skeleton">
							<div class="stat-icon skeleton" style="width: 40px; height: 40px;"></div>
							<div class="stat-content">
								<div class="skeleton" style="width: 60px; height: 20px; margin-bottom: 8px;"></div>
								<div class="skeleton" style="width: 100px; height: 16px;"></div>
							</div>
						</div>
					{/each}
				</div>
			{:else if error}
				<div class="error-message">
					<span class="error-icon">⚠️</span>
					<p>Failed to load system stats: {error}</p>
				</div>
			{:else if systemStats}
				<div class="stats-grid">
					<div class="stat-card neon-glow">
						<div class="stat-icon text-neon-cyan">🎵</div>
						<div class="stat-content">
							<div class="stat-value">{formatNumber(systemStats.total_tracks)}</div>
							<div class="stat-label">Total Tracks</div>
						</div>
					</div>
					<div class="stat-card neon-glow-pink">
						<div class="stat-icon text-neon-pink">💿</div>
						<div class="stat-content">
							<div class="stat-value">{formatNumber(systemStats.total_albums)}</div>
							<div class="stat-label">Albums</div>
						</div>
					</div>
					<div class="stat-card neon-glow-purple">
						<div class="stat-icon text-neon-purple">🎤</div>
						<div class="stat-content">
							<div class="stat-value">{formatNumber(systemStats.total_artists)}</div>
							<div class="stat-label">Artists</div>
						</div>
					</div>
					<div class="stat-card neon-glow">
						<div class="stat-icon text-neon-cyan">👥</div>
						<div class="stat-content">
							<div class="stat-value">{formatNumber(systemStats.total_users)}</div>
							<div class="stat-label">Active Users</div>
						</div>
					</div>
				</div>
			{/if}
		</section>

		<!-- Recent Recommendations -->
		<section class="recommendations-section">
			<div class="section-header">
				<h2 class="section-title">Your Latest Recommendations</h2>
				<div class="section-actions">
					{#if recentRecommendations.length > 0}
						<button
							class="play-all-btn"
							on:click={() => {
								console.log('🎼 Dashboard: Play All button clicked');
								playAllRecommendations();
							}}
							title="Play All">
							<span class="btn-icon">▶️</span>
							Play All
						</button>
					{/if}
					<a href="/recommendations" class="section-link">
						View All
						<span class="section-link-arrow">→</span>
					</a>
				</div>
			</div>

			{#if isLoading}
				<div class="recommendations-grid">
					{#each Array(6) as _}
						<div class="recommendation-card skeleton" style="height: 120px;"></div>
					{/each}
				</div>
			{:else if recentRecommendations.length > 0}
				<div class="recommendations-grid">
					{#each recentRecommendations as track}
						<div class="recommendation-card card-interactive">
							<div class="track-header">
								<div class="track-score">
									<span class="score-value">{Math.round(track.score * 100)}%</span>
									<span class="score-label">Match</span>
								</div>
								<div class="track-type">
									<span class="type-badge type-{track.recommendation_type}">
										{track.recommendation_type}
									</span>
								</div>
							</div>
							<div class="track-info">
								<h3 class="track-title">{track.title}</h3>
								<p class="track-artist">{track.artist}</p>
								<p class="track-album">{track.album}</p>
							</div>
							<div class="track-meta">
								<span class="track-duration">{formatDuration(track.duration)}</span>
								<span class="track-year">{track.year}</span>
							</div>
							<div class="track-reason">
								<p class="reason-text">{track.reason}</p>
							</div>
							<div class="track-actions">
								<button
									class="action-btn play-btn"
									on:click={() => {
										console.log('🎵 Dashboard: Play button clicked for:', track.title);
										playTrack(track);
									}}
									title="Play">
									<span class="btn-icon">▶️</span>
								</button>
								<button
									class="action-btn queue-btn"
									on:click={() => {
										console.log('📝 Dashboard: Queue button clicked for:', track.title);
										addToQueue(track);
									}}
									title="Add to Queue">
									<span class="btn-icon">➕</span>
								</button>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="empty-state">
					<div class="empty-icon">🎵</div>
					<h3>No recommendations yet</h3>
					<p>Generate your first recommendations to see them here</p>
					<a href="/recommendations" class="btn btn-primary">Get Started</a>
				</div>
			{/if}
		</section>

		<!-- Quick Actions -->
		<section class="actions-section">
			<h2 class="section-title">Quick Actions</h2>
			<div class="actions-grid">
				<a href="/library" class="action-card">
					<div class="action-icon">📚</div>
					<h3 class="action-title">Browse Library</h3>
					<p class="action-description">Explore your music collection</p>
				</a>
				<a href="/playlists" class="action-card">
					<div class="action-icon">📝</div>
					<h3 class="action-title">Create Playlist</h3>
					<p class="action-description">Build custom playlists</p>
				</a>
				<a href="/stats" class="action-card">
					<div class="action-icon">📊</div>
					<h3 class="action-title">View Analytics</h3>
					<p class="action-description">See your listening patterns</p>
				</a>
				<a href="/health" class="action-card">
					<div class="action-icon">⚡</div>
					<h3 class="action-title">System Status</h3>
					<p class="action-description">Check system health</p>
				</a>
			</div>
		</section>
	</div>
</div>

<style>
	.dashboard {
		opacity: 0;
		transition: opacity 0.5s ease-in-out;
	}

	.dashboard.loaded {
		opacity: 1;
	}

	/* Hero Section */
	.hero {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-xl) 0;
		margin-bottom: var(--spacing-xl);
		gap: var(--spacing-xl);
	}

	.hero-content {
		flex: 1;
		max-width: 600px;
	}

	.hero-title {
		font-size: 3.5rem;
		font-weight: 900;
		margin-bottom: var(--spacing-md);
		line-height: 1.1;
	}

	.hero-subtitle {
		font-size: 1.25rem;
		color: var(--text-secondary);
		margin-bottom: var(--spacing-lg);
		line-height: 1.4;
	}

	.hero-actions {
		display: flex;
		gap: var(--spacing-md);
		flex-wrap: wrap;
	}

	.hero-visual {
		position: relative;
		width: 200px;
		height: 200px;
		flex-shrink: 0;
	}

	.pulse-ring {
		position: absolute;
		border: 2px solid var(--neon-cyan);
		border-radius: 50%;
		animation: pulse-ring 3s ease-out infinite;
	}

	.pulse-ring-1 {
		width: 100%;
		height: 100%;
		animation-delay: 0s;
	}

	.pulse-ring-2 {
		width: 80%;
		height: 80%;
		top: 10%;
		left: 10%;
		border-color: var(--neon-pink);
		animation-delay: 1s;
	}

	.pulse-ring-3 {
		width: 60%;
		height: 60%;
		top: 20%;
		left: 20%;
		border-color: var(--neon-purple);
		animation-delay: 2s;
	}

	.hero-icon {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		font-size: 4rem;
		filter: drop-shadow(0 0 20px var(--neon-cyan));
	}

	@keyframes pulse-ring {
		0% {
			transform: scale(0.8);
			opacity: 1;
		}
		100% {
			transform: scale(1.2);
			opacity: 0;
		}
	}

	/* Section Styles */
	.section-title {
		font-size: 2rem;
		margin-bottom: var(--spacing-lg);
		text-align: center;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-lg);
	}

	.section-actions {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
	}

	.section-link {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		color: var(--neon-cyan);
		text-decoration: none;
		font-weight: 600;
		transition: all var(--transition-normal);
	}

	.section-link:hover {
		text-shadow: 0 0 10px var(--neon-cyan);
		transform: translateX(4px);
	}

	.section-link-arrow {
		transition: transform var(--transition-normal);
	}

	.section-link:hover .section-link-arrow {
		transform: translateX(4px);
	}

	/* Stats Section */
	.stats-section {
		margin-bottom: var(--spacing-xl);
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: var(--spacing-md);
	}

	.stat-card {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		padding: var(--spacing-lg);
		background: var(--bg-card);
		border-radius: var(--border-radius-lg);
		transition: all var(--transition-normal);
	}

	.stat-icon {
		font-size: 2.5rem;
		filter: drop-shadow(0 0 10px currentColor);
	}

	.stat-value {
		font-size: 2rem;
		font-weight: 800;
		font-family: var(--font-primary);
		color: var(--text-primary);
		margin-bottom: 4px;
	}

	.stat-label {
		font-size: 0.875rem;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		font-weight: 600;
	}

	/* Recommendations Section */
	.recommendations-section {
		margin-bottom: var(--spacing-xl);
	}

	.recommendations-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: var(--spacing-md);
	}

	.recommendation-card {
		padding: var(--spacing-md);
		background: var(--bg-card);
		border-radius: var(--border-radius-lg);
		border: 1px solid rgba(0, 255, 255, 0.2);
		transition: all var(--transition-normal);
	}

	.recommendation-card:hover {
		border-color: var(--neon-cyan);
		box-shadow: var(--shadow-neon);
		transform: translateY(-4px);
	}

	.track-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--spacing-sm);
	}

	.track-score {
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.score-value {
		font-size: 1.25rem;
		font-weight: 800;
		color: var(--neon-cyan);
		font-family: var(--font-primary);
	}

	.score-label {
		font-size: 0.75rem;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.type-badge {
		padding: 4px 8px;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.type-collaborative {
		background: rgba(0, 255, 255, 0.2);
		color: var(--neon-cyan);
		border: 1px solid var(--neon-cyan);
	}

	.type-content {
		background: rgba(255, 0, 255, 0.2);
		color: var(--neon-pink);
		border: 1px solid var(--neon-pink);
	}

	.type-popular {
		background: rgba(128, 0, 255, 0.2);
		color: var(--neon-purple);
		border: 1px solid var(--neon-purple);
	}

	.track-title {
		font-size: 1.1rem;
		font-weight: 700;
		color: var(--text-primary);
		margin-bottom: 4px;
	}

	.track-artist {
		color: var(--text-secondary);
		font-weight: 600;
		margin-bottom: 2px;
	}

	.track-album {
		color: var(--text-muted);
		font-size: 0.875rem;
	}

	.track-meta {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin: var(--spacing-sm) 0;
		font-size: 0.875rem;
		color: var(--text-muted);
	}

	.reason-text {
		font-size: 0.875rem;
		color: var(--text-secondary);
		font-style: italic;
		margin: 0;
	}

	/* Music Player Controls */
	.play-all-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-sm) var(--spacing-md);
		background: linear-gradient(45deg, var(--neon-cyan), var(--neon-pink));
		color: var(--bg-primary);
		border: none;
		border-radius: var(--border-radius);
		font-weight: 600;
		font-size: 0.875rem;
		cursor: pointer;
		transition: all var(--transition-normal);
		text-shadow: none;
	}

	.play-all-btn:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 20px rgba(0, 255, 255, 0.3);
	}

	.track-actions {
		display: flex;
		gap: var(--spacing-xs);
		margin-top: var(--spacing-sm);
		padding-top: var(--spacing-sm);
		border-top: 1px solid rgba(0, 255, 255, 0.1);
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 1px solid rgba(0, 255, 255, 0.3);
		background: rgba(0, 255, 255, 0.1);
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: all var(--transition-fast);
		color: var(--neon-cyan);
	}

	.action-btn:hover {
		background: rgba(0, 255, 255, 0.2);
		border-color: var(--neon-cyan);
		transform: scale(1.1);
		box-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
	}

	.play-btn:hover {
		color: var(--neon-pink);
		border-color: var(--neon-pink);
		background: rgba(255, 0, 255, 0.2);
		box-shadow: 0 0 10px rgba(255, 0, 255, 0.5);
	}

	.queue-btn:hover {
		color: var(--neon-purple);
		border-color: var(--neon-purple);
		background: rgba(128, 0, 255, 0.2);
		box-shadow: 0 0 10px rgba(128, 0, 255, 0.5);
	}

	.btn-icon {
		font-size: 0.875rem;
		filter: drop-shadow(0 0 5px currentColor);
	}

	/* Quick Actions */
	.actions-section {
		margin-bottom: var(--spacing-xl);
	}

	.actions-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: var(--spacing-md);
	}

	.action-card {
		display: block;
		padding: var(--spacing-lg);
		background: var(--bg-card);
		border: 1px solid rgba(0, 255, 255, 0.2);
		border-radius: var(--border-radius-lg);
		text-decoration: none;
		color: inherit;
		transition: all var(--transition-normal);
		text-align: center;
	}

	.action-card:hover {
		border-color: var(--neon-cyan);
		box-shadow: var(--shadow-neon);
		transform: translateY(-4px);
	}

	.action-icon {
		font-size: 3rem;
		margin-bottom: var(--spacing-md);
		filter: drop-shadow(0 0 10px var(--neon-cyan));
	}

	.action-title {
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--text-primary);
		margin-bottom: var(--spacing-sm);
	}

	.action-description {
		color: var(--text-secondary);
		margin: 0;
	}

	/* Error and Empty States */
	.error-message {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-md);
		background: rgba(255, 0, 0, 0.1);
		border: 1px solid rgba(255, 0, 0, 0.3);
		border-radius: var(--border-radius);
		color: #ff6b6b;
	}

	.error-icon {
		font-size: 1.5rem;
	}

	.empty-state {
		text-align: center;
		padding: var(--spacing-xl);
	}

	.empty-icon {
		font-size: 4rem;
		margin-bottom: var(--spacing-md);
		filter: drop-shadow(0 0 10px var(--neon-cyan));
	}

	.empty-state h3 {
		color: var(--text-primary);
		margin-bottom: var(--spacing-sm);
	}

	.empty-state p {
		color: var(--text-secondary);
		margin-bottom: var(--spacing-lg);
	}

	/* Responsive Design */
	@media (max-width: 768px) {
		.container {
			padding: 0 var(--spacing-sm);
		}

		.hero {
			flex-direction: column;
			text-align: center;
			gap: var(--spacing-lg);
			padding: var(--spacing-md);
		}

		.hero-title {
			font-size: 2rem;
			line-height: 1.2;
			word-wrap: break-word;
			hyphens: auto;
			overflow-wrap: break-word;
		}

		.hero-visual {
			width: 150px;
			height: 150px;
		}

		.hero-icon {
			font-size: 3rem;
		}

		.section-header {
			flex-direction: column;
			gap: var(--spacing-sm);
			text-align: center;
		}

		.section-header h2 {
			font-size: 1.5rem;
			word-wrap: break-word;
		}

		.stats-grid,
		.recommendations-grid,
		.actions-grid {
			grid-template-columns: 1fr;
			gap: var(--spacing-md);
		}

		.recommendation-card,
		.stat-card,
		.action-card {
			margin: 0 var(--spacing-xs);
		}
	}

	@media (max-width: 480px) {
		.container {
			padding: 0 var(--spacing-xs);
		}

		.hero {
			padding: var(--spacing-sm);
		}

		.hero-title {
			font-size: 1.5rem;
			line-height: 1.3;
			margin-bottom: var(--spacing-md);
			max-width: 100%;
			overflow-wrap: break-word;
			word-break: break-word;
		}

		.hero-subtitle {
			font-size: 0.9rem;
			line-height: 1.4;
			overflow-wrap: break-word;
		}

		.hero-actions {
			flex-direction: column;
			width: 100%;
			gap: var(--spacing-sm);
		}

		.hero-actions .btn {
			width: 100%;
			padding: var(--spacing-sm) var(--spacing-md);
			font-size: 0.9rem;
			word-wrap: break-word;
		}

		.dashboard {
			padding: var(--spacing-xs) 0;
		}
	}

	@media (max-width: 360px) {
		.container {
			padding: 0 var(--spacing-xxs, 8px);
		}

		.hero-title {
			font-size: 1.3rem;
			line-height: 1.4;
			padding: 0 var(--spacing-xs);
		}

		.hero-subtitle {
			font-size: 0.8rem;
			padding: 0 var(--spacing-xs);
		}

		.hero-visual {
			width: 120px;
			height: 120px;
		}

		.hero-icon {
			font-size: 2.5rem;
		}

		.recommendation-card,
		.stat-card,
		.action-card {
			margin: 0;
			padding: var(--spacing-sm);
		}

		.card-title {
			font-size: 0.9rem;
			word-wrap: break-word;
		}
	}
</style>
