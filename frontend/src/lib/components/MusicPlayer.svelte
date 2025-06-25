<script>
	import { onMount, onDestroy } from 'svelte';
	import { writable } from 'svelte/store';

	// Player state
	let audio;
	let isPlaying = false;
	let currentTrack = null;
	let queue = [];
	let currentIndex = 0;
	let volume = 0.8;
	let currentTime = 0;
	let duration = 0;
	let isLoading = false;
	let showQueue = false;
	let repeatMode = 'none'; // 'none', 'one', 'all'
	let shuffleEnabled = false;

	// Visual state
	let mounted = false;
	let progressDragging = false;
	let volumeDragging = false;

	// Initialize component
	onMount(() => {
		console.log('🎵 MusicPlayer: Component mounted');
		mounted = true;
		loadPlayerState();

		// Set up audio element event listeners
		if (audio) {
			console.log('🔊 MusicPlayer: Setting up audio event listeners');
			audio.addEventListener('loadstart', handleLoadStart);
			audio.addEventListener('loadeddata', handleLoadedData);
			audio.addEventListener('canplay', handleCanPlay);
			audio.addEventListener('timeupdate', handleTimeUpdate);
			audio.addEventListener('ended', handleTrackEnded);
			audio.addEventListener('error', handleError);
		} else {
			console.warn('⚠️ MusicPlayer: Audio element not available on mount');
		}
	});

	onDestroy(() => {
		if (audio) {
			audio.removeEventListener('loadstart', handleLoadStart);
			audio.removeEventListener('loadeddata', handleLoadedData);
			audio.removeEventListener('canplay', handleCanPlay);
			audio.removeEventListener('timeupdate', handleTimeUpdate);
			audio.removeEventListener('ended', handleTrackEnded);
			audio.removeEventListener('error', handleError);
		}
	});

	// Audio event handlers
	function handleLoadStart() {
		isLoading = true;
	}

	function handleLoadedData() {
		duration = audio.duration || 0;
	}

	function handleCanPlay() {
		isLoading = false;
	}

	function handleTimeUpdate() {
		if (!progressDragging) {
			currentTime = audio.currentTime || 0;
		}
	}

	function handleTrackEnded() {
		nextTrack();
	}

	function handleError(e) {
		console.error('🚨 MusicPlayer: Audio error occurred:', e);
		console.error('🚨 MusicPlayer: Current audio src:', audio?.src);
		console.error('🚨 MusicPlayer: Current track:', currentTrack);
		isLoading = false;
		isPlaying = false;
	}

	// Player controls
	async function togglePlayPause() {
		console.log('⏯️ MusicPlayer: togglePlayPause called, isPlaying:', isPlaying, 'currentTrack:', currentTrack);
		if (!currentTrack) {
			console.warn('⚠️ MusicPlayer: No current track to toggle');
			return;
		}

		if (isPlaying) {
			await pauseTrack();
		} else {
			await playTrack();
		}
	}

	async function playTrack(track = null) {
		console.log('▶️ MusicPlayer: playTrack called with:', track);

		if (track) {
			console.log('🔗 MusicPlayer: Setting audio source to:', track.stream_url);
			currentTrack = track;
			audio.src = track.stream_url;
		}

		if (!audio.src) {
			console.warn('⚠️ MusicPlayer: No audio source available');
			return;
		}

		try {
			console.log('🎧 MusicPlayer: Starting playback...');
			isLoading = true;
			await audio.play();
			isPlaying = true;
			console.log('✅ MusicPlayer: Playback started successfully');

			// Update backend
			if (currentTrack) {
				console.log('📡 MusicPlayer: Notifying backend of playback');
				await fetch(`/api/v1/player/play/${currentTrack.id}`, { method: 'POST' });
			}
		} catch (error) {
			console.error('❌ MusicPlayer: Failed to play track:', error);
			console.error('❌ MusicPlayer: Audio src was:', audio.src);
			isPlaying = false;
		} finally {
			isLoading = false;
		}
	}

	async function pauseTrack() {
		if (audio) {
			audio.pause();
			isPlaying = false;

			// Update backend
			await fetch('/api/v1/player/pause', { method: 'POST' });
		}
	}

	async function nextTrack() {
		if (queue.length === 0) return;

		if (shuffleEnabled) {
			currentIndex = Math.floor(Math.random() * queue.length);
		} else {
			currentIndex = (currentIndex + 1) % queue.length;
		}

		const track = queue[currentIndex];
		if (track) {
			await playTrack(track);
			await fetch('/api/v1/player/next', { method: 'POST' });
		}
	}

	async function previousTrack() {
		if (queue.length === 0) return;

		if (currentTime > 3) {
			// If more than 3 seconds have played, restart current track
			audio.currentTime = 0;
		} else {
			// Go to previous track
			currentIndex = currentIndex === 0 ? queue.length - 1 : currentIndex - 1;
			const track = queue[currentIndex];
			if (track) {
				await playTrack(track);
			}
		}

		await fetch('/api/v1/player/previous', { method: 'POST' });
	}

	// Queue management
	function addToQueue(track) {
		queue = [...queue, track];
		updateQueue();
	}

	function removeFromQueue(index) {
		queue = queue.filter((_, i) => i !== index);
		if (currentIndex >= index && currentIndex > 0) {
			currentIndex--;
		}
		updateQueue();
	}

	function playFromQueue(index) {
		currentIndex = index;
		const track = queue[index];
		if (track) {
			playTrack(track);
		}
	}

	function clearQueue() {
		queue = [];
		currentIndex = 0;
		updateQueue();
	}

	async function updateQueue() {
		try {
			await fetch('/api/v1/player/queue', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ queue, currentIndex })
			});
		} catch (error) {
			console.error('Failed to update queue:', error);
		}
	}

	// Progress control
	function handleProgressClick(e) {
		if (!audio || !duration) return;

		const rect = e.currentTarget.getBoundingClientRect();
		const percentage = (e.clientX - rect.left) / rect.width;
		const newTime = percentage * duration;

		audio.currentTime = newTime;
		currentTime = newTime;
	}

	function handleProgressDrag(e) {
		progressDragging = true;
		const rect = e.currentTarget.getBoundingClientRect();
		const percentage = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
		currentTime = percentage * duration;
	}

	function handleProgressDragEnd() {
		if (audio && progressDragging) {
			audio.currentTime = currentTime;
			progressDragging = false;
		}
	}

	// Volume control
	function handleVolumeChange(e) {
		const rect = e.currentTarget.getBoundingClientRect();
		const percentage = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
		volume = percentage;
		if (audio) {
			audio.volume = volume;
		}
	}

	// Utility functions
	function formatTime(seconds) {
		if (!seconds || isNaN(seconds)) return '0:00';
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}

	function toggleRepeat() {
		const modes = ['none', 'one', 'all'];
		const currentModeIndex = modes.indexOf(repeatMode);
		repeatMode = modes[(currentModeIndex + 1) % modes.length];
	}

	function toggleShuffle() {
		shuffleEnabled = !shuffleEnabled;
	}

	function toggleQueue() {
		showQueue = !showQueue;
	}

	// Load initial player state
	async function loadPlayerState() {
		try {
			const response = await fetch('/api/v1/player/current');
			const data = await response.json();

			if (data.success && data.current_track) {
				currentTrack = data.current_track;
				isPlaying = data.is_playing;
				currentTime = data.position || 0;
			}

			const queueResponse = await fetch('/api/v1/player/queue');
			const queueData = await queueResponse.json();

			if (queueData.success) {
				queue = queueData.queue || [];
				currentIndex = queueData.current_index || 0;
			}
		} catch (error) {
			console.error('Failed to load player state:', error);
		}
	}

	// Export functions for parent components
	export function playTrackFromParent(track) {
		console.log('🎵 MusicPlayer: playTrackFromParent called with track:', track);
		playTrack(track);
	}

	export function addTrackToQueue(track) {
		console.log('📝 MusicPlayer: addTrackToQueue called with track:', track);
		addToQueue(track);
	}

	export function setQueue(tracks, startIndex = 0) {
		console.log('🎼 MusicPlayer: setQueue called with', tracks.length, 'tracks, startIndex:', startIndex);
		queue = [...tracks];
		currentIndex = startIndex;
		if (tracks.length > 0) {
			console.log('🎼 MusicPlayer: Starting playback of track:', tracks[startIndex]);
			playTrack(tracks[startIndex]);
		}
		updateQueue();
	}
</script>

<!-- Audio element -->
<audio bind:this={audio} preload="metadata" />

<!-- Music Player UI -->
<div class="music-player" class:loaded={mounted} class:playing={isPlaying}>
	<!-- Current Track Display -->
	<div class="current-track">
		{#if currentTrack}
			<div class="track-artwork">
				<div class="track-icon">🎵</div>
				{#if isLoading}
					<div class="loading-spinner"></div>
				{/if}
			</div>
			<div class="track-info">
				<div class="track-title">{currentTrack.title}</div>
				<div class="track-artist">{currentTrack.artist}</div>
			</div>
		{:else}
			<div class="no-track">
				<div class="track-icon">⏸️</div>
				<div class="track-info">
					<div class="track-title">No track selected</div>
					<div class="track-artist">Choose a song to start listening</div>
				</div>
			</div>
		{/if}
	</div>

	<!-- Player Controls -->
	<div class="player-controls">
		<div class="control-buttons">
			<button class="control-btn" class:active={shuffleEnabled} on:click={toggleShuffle} title="Shuffle">
				🔀
			</button>
			<button class="control-btn" on:click={previousTrack} disabled={!currentTrack} title="Previous">
				⏮️
			</button>
			<button class="play-pause-btn" class:loading={isLoading} on:click={togglePlayPause} disabled={!currentTrack} title={isPlaying ? 'Pause' : 'Play'}>
				{#if isLoading}
					<div class="loading-dot"></div>
				{:else}
					<span class="play-icon">{isPlaying ? '⏸️' : '▶️'}</span>
				{/if}
			</button>
			<button class="control-btn" on:click={nextTrack} disabled={!currentTrack} title="Next">
				⏭️
			</button>
			<button class="control-btn" class:active={repeatMode !== 'none'} on:click={toggleRepeat} title="Repeat">
				{#if repeatMode === 'one'}
					🔂
				{:else if repeatMode === 'all'}
					🔁
				{:else}
					🔁
				{/if}
			</button>
		</div>

		<!-- Progress Bar -->
		<div class="progress-container">
			<span class="time-display">{formatTime(currentTime)}</span>
			<div class="progress-bar"
				 on:click={handleProgressClick}
				 on:mousedown={handleProgressDrag}
				 on:mouseup={handleProgressDragEnd}
				 role="slider"
				 tabindex="0"
				 aria-label="Track progress">
				<div class="progress-track"></div>
				<div class="progress-fill" style="width: {duration ? (currentTime / duration) * 100 : 0}%"></div>
				<div class="progress-handle" style="left: {duration ? (currentTime / duration) * 100 : 0}%"></div>
			</div>
			<span class="time-display">{formatTime(duration)}</span>
		</div>
	</div>

	<!-- Volume and Queue Controls -->
	<div class="player-extras">
		<div class="volume-control">
			<span class="volume-icon">🔊</span>
			<div class="volume-bar" on:click={handleVolumeChange} role="slider" tabindex="0" aria-label="Volume">
				<div class="volume-track"></div>
				<div class="volume-fill" style="width: {volume * 100}%"></div>
				<div class="volume-handle" style="left: {volume * 100}%"></div>
			</div>
		</div>

		<button class="queue-btn" class:active={showQueue} on:click={toggleQueue} title="Queue">
			📋 {queue.length}
		</button>
	</div>

	<!-- Queue Panel -->
	{#if showQueue}
		<div class="queue-panel">
			<div class="queue-header">
				<h3>Queue ({queue.length} tracks)</h3>
				<button class="clear-queue-btn" on:click={clearQueue} disabled={queue.length === 0}>
					Clear All
				</button>
			</div>
			<div class="queue-list">
				{#each queue as track, index}
					<div class="queue-item" class:current={index === currentIndex} on:click={() => playFromQueue(index)}>
						<div class="queue-track-info">
							<div class="queue-track-title">{track.title}</div>
							<div class="queue-track-artist">{track.artist}</div>
						</div>
						<div class="queue-track-duration">{formatTime(track.duration)}</div>
						<button class="remove-track-btn" on:click|stopPropagation={() => removeFromQueue(index)}>
							❌
						</button>
					</div>
				{:else}
					<div class="queue-empty">
						<p>No tracks in queue</p>
						<p class="queue-empty-subtitle">Add some music to get started!</p>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>

<style>
	.music-player {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		background: rgba(10, 10, 15, 0.95);
		backdrop-filter: blur(20px);
		border-top: 2px solid var(--neon-cyan);
		box-shadow: 0 -4px 30px rgba(0, 255, 255, 0.3);
		padding: var(--spacing-md);
		z-index: 1000;
		opacity: 0;
		transform: translateY(100%);
		transition: all var(--transition-normal);
	}

	.music-player.loaded {
		opacity: 1;
		transform: translateY(0);
	}

	.music-player.playing {
		border-top-color: var(--neon-pink);
		box-shadow: 0 -4px 30px rgba(255, 0, 255, 0.3);
	}

	/* Current Track Display */
	.current-track {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		margin-bottom: var(--spacing-md);
	}

	.track-artwork {
		position: relative;
		width: 50px;
		height: 50px;
		background: var(--bg-card);
		border: 2px solid var(--neon-cyan);
		border-radius: var(--border-radius);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.5rem;
		box-shadow: 0 0 15px rgba(0, 255, 255, 0.3);
	}

	.loading-spinner {
		position: absolute;
		width: 20px;
		height: 20px;
		border: 2px solid transparent;
		border-top: 2px solid var(--neon-pink);
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	.track-info {
		flex: 1;
		min-width: 0;
	}

	.track-title {
		font-weight: 600;
		color: var(--text-primary);
		font-size: 1rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.track-artist {
		color: var(--text-secondary);
		font-size: 0.875rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.no-track .track-icon {
		opacity: 0.5;
	}

	.no-track .track-title {
		color: var(--text-muted);
	}

	/* Player Controls */
	.player-controls {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		margin-bottom: var(--spacing-md);
	}

	.control-buttons {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--spacing-md);
	}

	.control-btn {
		background: transparent;
		border: 1px solid var(--neon-cyan);
		color: var(--neon-cyan);
		border-radius: 50%;
		width: 40px;
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 1rem;
	}

	.control-btn:hover {
		background: rgba(0, 255, 255, 0.1);
		box-shadow: 0 0 15px rgba(0, 255, 255, 0.5);
		transform: scale(1.1);
	}

	.control-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
		transform: none;
	}

	.control-btn.active {
		background: var(--neon-cyan);
		color: var(--bg-primary);
	}

	.play-pause-btn {
		background: linear-gradient(45deg, var(--neon-cyan), var(--neon-pink));
		border: none;
		border-radius: 50%;
		width: 60px;
		height: 60px;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all var(--transition-fast);
		position: relative;
		overflow: hidden;
	}

	.play-pause-btn:hover {
		transform: scale(1.1);
		box-shadow: 0 0 25px rgba(0, 255, 255, 0.8);
	}

	.play-pause-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
		transform: none;
	}

	.play-icon {
		font-size: 1.5rem;
		color: var(--bg-primary);
	}

	.loading-dot {
		width: 8px;
		height: 8px;
		background: var(--bg-primary);
		border-radius: 50%;
		animation: pulse 1.5s ease-in-out infinite;
	}

	/* Progress Bar */
	.progress-container {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.time-display {
		font-size: 0.75rem;
		color: var(--text-muted);
		font-family: var(--font-primary);
		min-width: 35px;
	}

	.progress-bar {
		flex: 1;
		height: 6px;
		position: relative;
		cursor: pointer;
		border-radius: 3px;
		overflow: hidden;
	}

	.progress-track {
		position: absolute;
		width: 100%;
		height: 100%;
		background: rgba(0, 255, 255, 0.2);
		border-radius: 3px;
	}

	.progress-fill {
		position: absolute;
		height: 100%;
		background: linear-gradient(90deg, var(--neon-cyan), var(--neon-pink));
		border-radius: 3px;
		transition: width 0.1s ease;
	}

	.progress-handle {
		position: absolute;
		top: 50%;
		width: 12px;
		height: 12px;
		background: var(--neon-cyan);
		border-radius: 50%;
		transform: translate(-50%, -50%);
		opacity: 0;
		transition: opacity var(--transition-fast);
		box-shadow: 0 0 10px var(--neon-cyan);
	}

	.progress-bar:hover .progress-handle {
		opacity: 1;
	}

	/* Volume and Queue Controls */
	.player-extras {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.volume-control {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.volume-icon {
		font-size: 1rem;
		color: var(--text-secondary);
	}

	.volume-bar {
		width: 100px;
		height: 4px;
		position: relative;
		cursor: pointer;
		border-radius: 2px;
		overflow: hidden;
	}

	.volume-track {
		position: absolute;
		width: 100%;
		height: 100%;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 2px;
	}

	.volume-fill {
		position: absolute;
		height: 100%;
		background: var(--neon-cyan);
		border-radius: 2px;
		transition: width 0.1s ease;
	}

	.volume-handle {
		position: absolute;
		top: 50%;
		width: 8px;
		height: 8px;
		background: var(--neon-cyan);
		border-radius: 50%;
		transform: translate(-50%, -50%);
		opacity: 0;
		transition: opacity var(--transition-fast);
	}

	.volume-bar:hover .volume-handle {
		opacity: 1;
	}

	.queue-btn {
		background: transparent;
		border: 1px solid var(--neon-purple);
		color: var(--neon-purple);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 0.875rem;
		font-family: var(--font-primary);
	}

	.queue-btn:hover {
		background: rgba(128, 0, 255, 0.1);
		box-shadow: 0 0 10px rgba(128, 0, 255, 0.5);
	}

	.queue-btn.active {
		background: var(--neon-purple);
		color: var(--bg-primary);
	}

	/* Queue Panel */
	.queue-panel {
		position: absolute;
		bottom: 100%;
		right: 0;
		width: 350px;
		max-height: 400px;
		background: var(--bg-card);
		border: 1px solid var(--neon-cyan);
		border-radius: var(--border-radius-lg) var(--border-radius-lg) 0 0;
		box-shadow: 0 -4px 20px rgba(0, 255, 255, 0.3);
		animation: slideUp 0.3s ease-out;
	}

	.queue-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--spacing-md);
		border-bottom: 1px solid rgba(0, 255, 255, 0.3);
	}

	.queue-header h3 {
		margin: 0;
		color: var(--text-primary);
		font-size: 1rem;
	}

	.clear-queue-btn {
		background: transparent;
		border: 1px solid var(--neon-pink);
		color: var(--neon-pink);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 0.75rem;
	}

	.clear-queue-btn:hover {
		background: rgba(255, 0, 255, 0.1);
	}

	.clear-queue-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	.queue-list {
		max-height: 300px;
		overflow-y: auto;
		padding: var(--spacing-sm);
	}

	.queue-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: all var(--transition-fast);
		margin-bottom: 1px;
	}

	.queue-item:hover {
		background: rgba(0, 255, 255, 0.1);
	}

	.queue-item.current {
		background: rgba(0, 255, 255, 0.2);
		border: 1px solid var(--neon-cyan);
	}

	.queue-track-info {
		flex: 1;
		min-width: 0;
	}

	.queue-track-title {
		font-weight: 600;
		color: var(--text-primary);
		font-size: 0.875rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.queue-track-artist {
		color: var(--text-secondary);
		font-size: 0.75rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.queue-track-duration {
		color: var(--text-muted);
		font-size: 0.75rem;
		font-family: var(--font-primary);
	}

	.remove-track-btn {
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 2px;
		opacity: 0;
		transition: opacity var(--transition-fast);
		font-size: 0.75rem;
	}

	.queue-item:hover .remove-track-btn {
		opacity: 1;
	}

	.remove-track-btn:hover {
		color: var(--neon-pink);
	}

	.queue-empty {
		text-align: center;
		padding: var(--spacing-xl);
		color: var(--text-muted);
	}

	.queue-empty-subtitle {
		font-size: 0.875rem;
		margin-top: var(--spacing-xs);
	}

	/* Animations */
	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; transform: scale(1); }
		50% { opacity: 0.5; transform: scale(0.8); }
	}

	@keyframes slideUp {
		from {
			opacity: 0;
			transform: translateY(20px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	/* Responsive Design */
	@media (max-width: 768px) {
		.music-player {
			padding: var(--spacing-sm);
		}

		.current-track {
			margin-bottom: var(--spacing-sm);
		}

		.control-buttons {
			gap: var(--spacing-sm);
		}

		.control-btn {
			width: 35px;
			height: 35px;
			font-size: 0.875rem;
		}

		.play-pause-btn {
			width: 50px;
			height: 50px;
		}

		.volume-control {
			display: none;
		}

		.queue-panel {
			width: 100vw;
			right: 0;
			left: 0;
		}
	}

	/* Custom Scrollbar for Queue */
	.queue-list::-webkit-scrollbar {
		width: 6px;
	}

	.queue-list::-webkit-scrollbar-track {
		background: rgba(0, 255, 255, 0.1);
		border-radius: 3px;
	}

	.queue-list::-webkit-scrollbar-thumb {
		background: var(--neon-cyan);
		border-radius: 3px;
	}

	.queue-list::-webkit-scrollbar-thumb:hover {
		background: var(--neon-cyan-bright);
	}
</style>
