<script>
	import { onMount, onDestroy } from 'svelte';
	import { musicPlayerStore } from '$lib/stores/musicPlayer.js';

	// Component props
	export let musicPlayer = null;

	// Component state
	let mounted = false;
	let showOverlay = false;
	let showQueueOverlay = false;
	let showQueueInOverlay = false;
	let isMinimized = true;
	let currentTrack = null;
	let isPlaying = false;
	let currentTime = 0;
	let duration = 0;
	let volume = 1;
	let queue = [];
	let currentIndex = 0;

	// Floating player position
	let isDragging = false;
	let position = { x: 20, y: window.innerHeight - 200 };
	let dragOffset = { x: 0, y: 0 };

	// Subscribe to the store to get real-time updates
	$: if ($musicPlayerStore) {
		currentTrack = $musicPlayerStore.currentTrack || null;
		isPlaying = $musicPlayerStore.isPlaying || false;
		currentTime = $musicPlayerStore.currentTime || 0;
		duration = $musicPlayerStore.duration || 0;
		volume = $musicPlayerStore.volume || 1;
		queue = $musicPlayerStore.queue || [];
		currentIndex = $musicPlayerStore.currentIndex || 0;

		console.log('🎵 FloatingPlayer: Store updated:', {
			hasCurrentTrack: !!currentTrack,
			currentTrackTitle: currentTrack?.title,
			isPlaying,
			mounted,
			shouldShow: mounted && currentTrack,
			storeMethods: {
				hasTogglePlayPause: typeof $musicPlayerStore.togglePlayPause,
				hasNextTrack: typeof $musicPlayerStore.nextTrack,
				hasPreviousTrack: typeof $musicPlayerStore.previousTrack,
				allStoreKeys: Object.keys($musicPlayerStore || {})
			}
		});
	}

	onMount(() => {
		mounted = true;
		console.log('🎵 FloatingPlayer: Component mounted');
		// Set initial position based on screen size
		position = {
			x: window.innerWidth - 100,
			y: window.innerHeight - 200
		};
		console.log('🎵 FloatingPlayer: Initial position set:', position);
	});

	onDestroy(() => {
		// No cleanup needed
	});

	// Player control functions
	function togglePlayPause() {
		console.log('🎵 FloatingPlayer: togglePlayPause called');
		console.log('🎵 FloatingPlayer: Store state:', $musicPlayerStore);
		if (musicPlayer?.togglePlayPause) {
			console.log('🎵 FloatingPlayer: Calling musicPlayer.togglePlayPause directly');
			musicPlayer.togglePlayPause();
		} else if ($musicPlayerStore?.togglePlayPause) {
			console.log('🎵 FloatingPlayer: Calling store togglePlayPause');
			$musicPlayerStore.togglePlayPause();
		} else {
			console.warn('🎵 FloatingPlayer: No togglePlayPause method available');
			console.warn('🎵 FloatingPlayer: musicPlayer:', musicPlayer);
			console.warn('🎵 FloatingPlayer: store:', $musicPlayerStore);
		}
	}

	function nextTrack() {
		console.log('🎵 FloatingPlayer: nextTrack called');
		if (musicPlayer?.nextTrack) {
			console.log('🎵 FloatingPlayer: Calling musicPlayer.nextTrack directly');
			musicPlayer.nextTrack();
		} else if ($musicPlayerStore?.nextTrack) {
			console.log('🎵 FloatingPlayer: Calling store nextTrack');
			$musicPlayerStore.nextTrack();
		} else {
			console.warn('🎵 FloatingPlayer: No nextTrack method available');
		}
	}

	function previousTrack() {
		console.log('🎵 FloatingPlayer: previousTrack called');
		if (musicPlayer?.previousTrack) {
			console.log('🎵 FloatingPlayer: Calling musicPlayer.previousTrack directly');
			musicPlayer.previousTrack();
		} else if ($musicPlayerStore?.previousTrack) {
			console.log('🎵 FloatingPlayer: Calling store previousTrack');
			$musicPlayerStore.previousTrack();
		} else {
			console.warn('🎵 FloatingPlayer: No previousTrack method available');
		}
	}

	function toggleOverlay() {
		showOverlay = !showOverlay;
		if (!showOverlay) {
			showQueueInOverlay = false;
		}
	}

	function closeOverlay() {
		showOverlay = false;
		showQueueInOverlay = false;
	}

	function toggleQueueOverlay() {
		showQueueOverlay = !showQueueOverlay;
	}

	function closeQueueOverlay() {
		showQueueOverlay = false;
	}

	function toggleQueueInOverlay() {
		showQueueInOverlay = !showQueueInOverlay;
	}

	function playFromQueue(index) {
		if (musicPlayer?.setQueue && queue.length > 0) {
			console.log('🎵 FloatingPlayer: Playing from queue index:', index);
			musicPlayer.setQueue(queue, index);
		}
	}

	function removeFromQueue(index) {
		if (queue.length > 0 && index >= 0 && index < queue.length) {
			console.log('🎵 FloatingPlayer: Removing from queue index:', index);
			const newQueue = queue.filter((_, i) => i !== index);
			if (musicPlayer?.setQueue) {
				const newCurrentIndex = index < currentIndex ? currentIndex - 1 : currentIndex;
				musicPlayer.setQueue(newQueue, Math.max(0, newCurrentIndex));
			}
		}
	}

	function clearQueue() {
		if (musicPlayer?.setQueue) {
			console.log('🎵 FloatingPlayer: Clearing queue');
			musicPlayer.setQueue([], 0);
		}
	}

	// Dragging functionality
	function handleMouseDown(e) {
		isDragging = true;
		dragOffset.x = e.clientX - position.x;
		dragOffset.y = e.clientY - position.y;
		document.addEventListener('mousemove', handleMouseMove);
		document.addEventListener('mouseup', handleMouseUp);
	}

	function handleMouseMove(e) {
		if (isDragging) {
			position.x = Math.max(0, Math.min(window.innerWidth - 80, e.clientX - dragOffset.x));
			position.y = Math.max(0, Math.min(window.innerHeight - 160, e.clientY - dragOffset.y));
		}
	}

	function handleMouseUp() {
		isDragging = false;
		document.removeEventListener('mousemove', handleMouseMove);
		document.removeEventListener('mouseup', handleMouseUp);
	}

	// Format time helper
	function formatTime(seconds) {
		if (!seconds || isNaN(seconds)) return '0:00';
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}

	// Progress calculation
	$: progress = duration > 0 ? (currentTime / duration) * 100 : 0;

	// Debug reactive statement
	$: {
		console.log('🎵 FloatingPlayer: State check:', {
			mounted,
			hasCurrentTrack: !!currentTrack,
			currentTrackTitle: currentTrack?.title,
			isPlaying,
			showDisc: mounted && currentTrack
		});
	}
</script>

<!-- Floating Player Widget -->
{#if mounted && currentTrack}
	<div
		class="floating-player"
		style="left: {position.x}px; top: {position.y}px;"
		class:dragging={isDragging}
	>
		<!-- Vinyl Disc -->
		<div
			class="vinyl-disc"
			class:spinning={isPlaying}
			on:click={toggleOverlay}
			on:mousedown={handleMouseDown}
			role="button"
			tabindex="0"
		>
			<!-- Disc Surface -->
			<div class="disc-surface">
				<div class="disc-center">
					<div class="disc-hole"></div>
				</div>
				<!-- Track info on disc -->
				<div class="disc-label">
					<div class="track-title">{currentTrack.title}</div>
					<div class="track-artist">{currentTrack.artist}</div>
				</div>
			</div>

			<!-- Progress Ring -->
			<svg class="progress-ring" width="80" height="80">
				<circle
					cx="40"
					cy="40"
					r="38"
					stroke="#333"
					stroke-width="2"
					fill="none"
					opacity="0.3"
				/>
				<circle
					cx="40"
					cy="40"
					r="38"
					stroke="#ff6b9d"
					stroke-width="2"
					fill="none"
					stroke-dasharray="{2 * Math.PI * 38}"
					stroke-dashoffset="{2 * Math.PI * 38 * (1 - progress / 100)}"
					stroke-linecap="round"
					class="progress-circle"
				/>
			</svg>
		</div>

		<!-- Mini Controls -->
		<div class="mini-controls">
			<button class="control-btn" on:click|stopPropagation={previousTrack} title="Previous">
				⏮️
			</button>
			<button class="control-btn play-pause" on:click|stopPropagation={togglePlayPause} title={isPlaying ? 'Pause' : 'Play'}>
				{isPlaying ? '⏸️' : '▶️'}
			</button>
			<button class="control-btn" on:click|stopPropagation={nextTrack} title="Next">
				⏭️
			</button>
			<button class="control-btn queue-toggle" on:click|stopPropagation={toggleQueueOverlay} title="Queue">
				📋
			</button>
		</div>
	</div>
{/if}

<!-- Full Player Overlay -->
{#if showOverlay && currentTrack}
	<div class="player-overlay" on:click={closeOverlay}>
		<div class="overlay-content" on:click|stopPropagation>
			<!-- Close Button -->
			<button class="close-btn" on:click={closeOverlay}>✕</button>

			<!-- Large Disc -->
			<div class="large-disc" class:spinning={isPlaying}>
				<div class="large-disc-surface">
					<div class="large-disc-center">
						<div class="large-disc-hole"></div>
					</div>
					<div class="large-disc-label">
						<div class="large-track-title">{currentTrack.title}</div>
						<div class="large-track-artist">{currentTrack.artist}</div>
						<div class="large-track-album">{currentTrack.album || 'Unknown Album'}</div>
					</div>
				</div>
			</div>

			<!-- Full Controls -->
			<div class="full-controls">
				<div class="control-buttons">
					<button class="large-control-btn" on:click={previousTrack}>⏮️</button>
					<button class="large-control-btn play-pause" on:click={togglePlayPause}>
						{isPlaying ? '⏸️' : '▶️'}
					</button>
					<button class="large-control-btn" on:click={nextTrack}>⏭️</button>
				</div>

				<!-- Progress Bar -->
				<div class="progress-section">
					<span class="time">{formatTime(currentTime)}</span>
					<div class="progress-bar">
						<div class="progress-fill" style="width: {progress}%"></div>
					</div>
					<span class="time">{formatTime(duration)}</span>
				</div>

				<!-- Queue Info -->
				{#if queue.length > 0}
					<div class="queue-info">
						<span>Track {currentIndex + 1} of {queue.length}</span>
						<button class="queue-toggle-btn" on:click={toggleQueueInOverlay}>
							{showQueueInOverlay ? '🔼 Hide Queue' : '🔽 Show Queue'}
						</button>
					</div>
				{/if}

				<!-- Queue Section in Overlay -->
				{#if showQueueInOverlay && queue.length > 0}
					<div class="queue-section">
						<div class="queue-header">
							<h4>Current Queue</h4>
							<button class="clear-queue" on:click={clearQueue} title="Clear Queue">
								🗑️ Clear All
							</button>
						</div>
						<div class="queue-list">
							{#each queue as track, index}
								<div
									class="queue-item"
									class:current={index === currentIndex}
									on:click={() => playFromQueue(index)}
								>
									<span class="queue-number">{index + 1}</span>
									<div class="queue-track-info">
										<div class="queue-track-title">{track.title}</div>
										<div class="queue-track-artist">{track.artist}</div>
									</div>
									<div class="queue-actions">
										{#if index === currentIndex}
											<span class="now-playing">▶️</span>
										{/if}
										<button
											class="remove-btn"
											on:click|stopPropagation={() => removeFromQueue(index)}
											title="Remove from queue"
										>
											✕
										</button>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<!-- Dedicated Queue Overlay -->
{#if showQueueOverlay}
	<div class="queue-overlay" on:click={closeQueueOverlay}>
		<div class="queue-overlay-content" on:click|stopPropagation>
			<!-- Close Button -->
			<button class="close-btn" on:click={closeQueueOverlay}>✕</button>

			<!-- Queue Header -->
			<div class="queue-overlay-header">
				<h3>🎵 Music Queue</h3>
				{#if queue.length > 0}
					<p>{queue.length} track{queue.length !== 1 ? 's' : ''} in queue</p>
					<button class="clear-all-btn" on:click={clearQueue}>
						🗑️ Clear All
					</button>
				{:else}
					<p>No tracks in queue</p>
				{/if}
			</div>

			<!-- Queue List -->
			{#if queue.length > 0}
				<div class="queue-overlay-list">
					{#each queue as track, index}
						<div
							class="queue-overlay-item"
							class:current={index === currentIndex}
							on:click={() => playFromQueue(index)}
						>
							<div class="queue-item-number">
								{#if index === currentIndex}
									<span class="playing-indicator">▶️</span>
								{:else}
									<span class="track-number">{index + 1}</span>
								{/if}
							</div>

							<div class="queue-item-info">
								<div class="queue-item-title">{track.title}</div>
								<div class="queue-item-artist">{track.artist}</div>
								<div class="queue-item-album">{track.album || 'Unknown Album'}</div>
							</div>

							<div class="queue-item-duration">
								{formatTime(track.duration)}
							</div>

							<div class="queue-item-actions">
								<button
									class="queue-remove-btn"
									on:click|stopPropagation={() => removeFromQueue(index)}
									title="Remove from queue"
								>
									🗑️
								</button>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="empty-queue">
					<span class="empty-icon">🎵</span>
					<h4>Queue is Empty</h4>
					<p>Add some tracks to start your music journey!</p>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.floating-player {
		position: fixed;
		z-index: 1000;
		cursor: grab;
		user-select: none;
		transition: transform 0.2s ease;
	}

	.floating-player:hover {
		transform: scale(1.05);
	}

	.floating-player.dragging {
		cursor: grabbing;
		transform: scale(1.1);
	}

	.vinyl-disc {
		position: relative;
		width: 80px;
		height: 80px;
		border-radius: 50%;
		background: linear-gradient(45deg, #1a1a1a, #333);
		box-shadow: 0 8px 25px rgba(0, 0, 0, 0.3);
		cursor: pointer;
		transition: all 0.3s ease;
	}

	.vinyl-disc:hover {
		box-shadow: 0 12px 35px rgba(255, 107, 157, 0.3);
	}

	.vinyl-disc.spinning {
		animation: spin 3s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.disc-surface {
		position: relative;
		width: 100%;
		height: 100%;
		border-radius: 50%;
		background:
			radial-gradient(circle at 30% 30%, rgba(255, 255, 255, 0.1), transparent 50%),
			conic-gradient(from 0deg, #2a2a2a, #1a1a1a, #2a2a2a, #1a1a1a);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
	}

	.disc-center {
		position: absolute;
		width: 20px;
		height: 20px;
		background: #ff6b9d;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10;
	}

	.disc-hole {
		width: 8px;
		height: 8px;
		background: #000;
		border-radius: 50%;
	}

	.disc-label {
		position: absolute;
		text-align: center;
		color: #fff;
		font-size: 8px;
		line-height: 1.1;
		max-width: 50px;
		margin-top: 5px;
	}

	.track-title {
		font-weight: bold;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.track-artist {
		opacity: 0.8;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.progress-ring {
		position: absolute;
		top: 0;
		left: 0;
		transform: rotate(-90deg);
	}

	.progress-circle {
		transition: stroke-dashoffset 0.3s ease;
	}

	.mini-controls {
		display: flex;
		justify-content: center;
		gap: 8px;
		margin-top: 8px;
		padding: 8px;
		background: rgba(0, 0, 0, 0.8);
		border-radius: 20px;
		backdrop-filter: blur(10px);
	}

	.control-btn {
		background: none;
		border: none;
		color: #fff;
		font-size: 14px;
		cursor: pointer;
		padding: 4px 8px;
		border-radius: 12px;
		transition: all 0.2s ease;
	}

	.control-btn:hover {
		background: rgba(255, 107, 157, 0.2);
		transform: scale(1.1);
	}

	.control-btn.play-pause {
		background: #ff6b9d;
		font-size: 16px;
	}

	.control-btn.play-pause:hover {
		background: #ff5a8a;
	}

	.control-btn.queue-toggle {
		background: rgba(255, 165, 0, 0.2);
		color: #ffa500;
	}

	.control-btn.queue-toggle:hover {
		background: rgba(255, 165, 0, 0.4);
	}

	/* Overlay Styles */
	.player-overlay {
		position: fixed;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		background: rgba(0, 0, 0, 0.9);
		backdrop-filter: blur(10px);
		z-index: 2000;
		display: flex;
		align-items: center;
		justify-content: center;
		animation: fadeIn 0.3s ease;
	}

	@keyframes fadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	.overlay-content {
		position: relative;
		text-align: center;
		max-width: 500px;
		padding: 40px;
		background: linear-gradient(135deg, #1a1a1a, #2a2a2a);
		border-radius: 20px;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
		animation: slideIn 0.3s ease;
	}

	@keyframes slideIn {
		from { transform: translateY(50px); opacity: 0; }
		to { transform: translateY(0); opacity: 1; }
	}

	.close-btn {
		position: absolute;
		top: 15px;
		right: 15px;
		background: none;
		border: none;
		color: #fff;
		font-size: 24px;
		cursor: pointer;
		padding: 5px;
		border-radius: 50%;
		transition: all 0.2s ease;
	}

	.close-btn:hover {
		background: rgba(255, 107, 157, 0.2);
	}

	.large-disc {
		width: 250px;
		height: 250px;
		margin: 0 auto 30px;
		border-radius: 50%;
		background: linear-gradient(45deg, #1a1a1a, #333);
		box-shadow: 0 15px 40px rgba(0, 0, 0, 0.4);
		position: relative;
	}

	.large-disc.spinning {
		animation: spin 3s linear infinite;
	}

	.large-disc-surface {
		width: 100%;
		height: 100%;
		border-radius: 50%;
		background:
			radial-gradient(circle at 30% 30%, rgba(255, 255, 255, 0.1), transparent 50%),
			conic-gradient(from 0deg, #2a2a2a, #1a1a1a, #2a2a2a, #1a1a1a);
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
		overflow: hidden;
	}

	.large-disc-center {
		position: absolute;
		width: 60px;
		height: 60px;
		background: #ff6b9d;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10;
	}

	.large-disc-hole {
		width: 24px;
		height: 24px;
		background: #000;
		border-radius: 50%;
	}

	.large-disc-label {
		position: absolute;
		text-align: center;
		color: #fff;
		max-width: 180px;
		margin-top: 20px;
	}

	.large-track-title {
		font-size: 18px;
		font-weight: bold;
		margin-bottom: 5px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.large-track-artist {
		font-size: 14px;
		opacity: 0.9;
		margin-bottom: 3px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.large-track-album {
		font-size: 12px;
		opacity: 0.7;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.full-controls {
		color: #fff;
	}

	.control-buttons {
		display: flex;
		justify-content: center;
		gap: 20px;
		margin-bottom: 20px;
	}

	.large-control-btn {
		background: rgba(255, 107, 157, 0.1);
		border: 2px solid #ff6b9d;
		color: #fff;
		font-size: 24px;
		cursor: pointer;
		padding: 15px 20px;
		border-radius: 50%;
		transition: all 0.2s ease;
		width: 60px;
		height: 60px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.large-control-btn:hover {
		background: #ff6b9d;
		transform: scale(1.1);
	}

	.large-control-btn.play-pause {
		background: #ff6b9d;
		font-size: 28px;
		width: 70px;
		height: 70px;
	}

	.progress-section {
		display: flex;
		align-items: center;
		gap: 15px;
		margin-bottom: 15px;
	}

	.time {
		font-size: 14px;
		color: #ccc;
		min-width: 40px;
	}

	.progress-bar {
		flex: 1;
		height: 6px;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 3px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: linear-gradient(90deg, #ff6b9d, #ff5a8a);
		transition: width 0.3s ease;
	}

	.queue-info {
		font-size: 12px;
		color: #aaa;
		margin-top: 10px;
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 10px;
	}

	.queue-toggle-btn {
		background: rgba(255, 107, 157, 0.1);
		border: 1px solid #ff6b9d;
		color: #ff6b9d;
		padding: 5px 10px;
		border-radius: 6px;
		cursor: pointer;
		font-size: 10px;
		transition: all 0.2s ease;
	}

	.queue-toggle-btn:hover {
		background: rgba(255, 107, 157, 0.2);
	}

	/* Queue Section in Overlay */
	.queue-section {
		margin-top: 20px;
		background: rgba(0, 0, 0, 0.3);
		border-radius: 12px;
		padding: 15px;
		max-height: 300px;
		overflow-y: auto;
	}

	.queue-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 10px;
		padding-bottom: 10px;
		border-bottom: 1px solid rgba(255, 107, 157, 0.3);
	}

	.queue-header h4 {
		color: #ff6b9d;
		margin: 0;
		font-size: 1rem;
	}

	.clear-queue {
		background: rgba(255, 0, 0, 0.1);
		border: 1px solid #ff4444;
		color: #ff4444;
		padding: 4px 8px;
		border-radius: 6px;
		cursor: pointer;
		font-size: 10px;
		transition: all 0.2s ease;
	}

	.clear-queue:hover {
		background: rgba(255, 0, 0, 0.2);
	}

	.queue-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.queue-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px;
		background: rgba(255, 255, 255, 0.05);
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s ease;
		border: 1px solid transparent;
	}

	.queue-item:hover {
		background: rgba(255, 255, 255, 0.1);
		border-color: rgba(0, 255, 255, 0.3);
	}

	.queue-item.current {
		background: rgba(255, 107, 157, 0.2);
		border-color: #ff6b9d;
	}

	.queue-number {
		font-size: 12px;
		color: #999;
		min-width: 20px;
		text-align: center;
	}

	.queue-track-info {
		flex: 1;
		min-width: 0;
	}

	.queue-track-title {
		font-size: 12px;
		color: #fff;
		font-weight: bold;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.queue-track-artist {
		font-size: 10px;
		color: #ccc;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.queue-actions {
		display: flex;
		align-items: center;
		gap: 5px;
	}

	.now-playing {
		color: #00ff88;
		font-size: 10px;
	}

	.remove-btn {
		background: none;
		border: none;
		color: #ff4444;
		cursor: pointer;
		padding: 2px;
		border-radius: 3px;
		font-size: 10px;
		opacity: 0;
		transition: all 0.2s ease;
	}

	.queue-item:hover .remove-btn {
		opacity: 1;
	}

	.remove-btn:hover {
		background: rgba(255, 68, 68, 0.2);
	}

	/* Dedicated Queue Overlay */
	.queue-overlay {
		position: fixed;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		background: rgba(0, 0, 0, 0.8);
		backdrop-filter: blur(5px);
		z-index: 1500;
		display: flex;
		align-items: center;
		justify-content: center;
		animation: fadeIn 0.3s ease;
	}

	.queue-overlay-content {
		position: relative;
		width: 90%;
		max-width: 500px;
		max-height: 80vh;
		background: linear-gradient(135deg, #1a1a1a, #2a2a2a);
		border-radius: 15px;
		padding: 20px;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
		animation: slideIn 0.3s ease;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.queue-overlay-header {
		text-align: center;
		margin-bottom: 20px;
		padding-bottom: 15px;
		border-bottom: 2px solid rgba(255, 107, 157, 0.3);
	}

	.queue-overlay-header h3 {
		color: #ff6b9d;
		margin: 0 0 5px 0;
		font-size: 1.4rem;
	}

	.queue-overlay-header p {
		color: #ccc;
		margin: 0 0 10px 0;
		font-size: 0.9rem;
	}

	.clear-all-btn {
		background: rgba(255, 0, 0, 0.1);
		border: 1px solid #ff4444;
		color: #ff4444;
		padding: 8px 15px;
		border-radius: 8px;
		cursor: pointer;
		font-size: 0.9rem;
		transition: all 0.2s ease;
	}

	.clear-all-btn:hover {
		background: rgba(255, 0, 0, 0.2);
	}

	.queue-overlay-list {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding-right: 5px;
	}

	.queue-overlay-item {
		display: flex;
		align-items: center;
		gap: 15px;
		padding: 12px;
		background: rgba(255, 255, 255, 0.05);
		border-radius: 10px;
		cursor: pointer;
		transition: all 0.2s ease;
		border: 1px solid transparent;
	}

	.queue-overlay-item:hover {
		background: rgba(255, 255, 255, 0.1);
		border-color: rgba(0, 255, 255, 0.3);
		transform: translateX(5px);
	}

	.queue-overlay-item.current {
		background: rgba(255, 107, 157, 0.2);
		border-color: #ff6b9d;
	}

	.queue-item-number {
		min-width: 30px;
		text-align: center;
	}

	.playing-indicator {
		color: #00ff88;
		font-size: 14px;
		animation: pulse 1.5s infinite;
	}

	.track-number {
		color: #999;
		font-size: 14px;
		font-weight: bold;
	}

	.queue-item-info {
		flex: 1;
		min-width: 0;
	}

	.queue-item-title {
		font-size: 14px;
		color: #fff;
		font-weight: bold;
		margin-bottom: 2px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.queue-item-artist {
		font-size: 12px;
		color: #00ffff;
		margin-bottom: 2px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.queue-item-album {
		font-size: 11px;
		color: #999;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.queue-item-duration {
		font-size: 12px;
		color: #ccc;
		min-width: 40px;
		text-align: right;
	}

	.queue-item-actions {
		display: flex;
		align-items: center;
	}

	.queue-remove-btn {
		background: rgba(255, 68, 68, 0.1);
		border: 1px solid #ff4444;
		color: #ff4444;
		padding: 6px 8px;
		border-radius: 6px;
		cursor: pointer;
		font-size: 12px;
		opacity: 0;
		transition: all 0.2s ease;
	}

	.queue-overlay-item:hover .queue-remove-btn {
		opacity: 1;
	}

	.queue-remove-btn:hover {
		background: rgba(255, 68, 68, 0.2);
	}

	.empty-queue {
		text-align: center;
		padding: 40px 20px;
		color: #999;
	}

	.empty-icon {
		font-size: 3rem;
		margin-bottom: 15px;
		display: block;
		opacity: 0.5;
	}

	.empty-queue h4 {
		color: #ccc;
		margin: 0 0 10px 0;
		font-size: 1.2rem;
	}

	.empty-queue p {
		margin: 0;
		font-size: 0.9rem;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.5; }
	}

	/* Mobile Responsive */
	@media (max-width: 768px) {
		.vinyl-disc {
			width: 60px;
			height: 60px;
		}

		.mini-controls {
			gap: 6px;
			padding: 6px;
		}

		.control-btn {
			font-size: 12px;
			padding: 3px 6px;
		}

		.overlay-content {
			max-width: 90vw;
			padding: 20px;
		}

		.large-disc {
			width: 200px;
			height: 200px;
		}

		.large-track-title {
			font-size: 16px;
		}

		.large-control-btn {
			width: 50px;
			height: 50px;
			font-size: 20px;
		}

		.large-control-btn.play-pause {
			width: 60px;
			height: 60px;
			font-size: 24px;
		}

		.queue-overlay-content {
			width: 95%;
			max-height: 70vh;
			padding: 15px;
		}

		.queue-overlay-item {
			gap: 10px;
			padding: 10px;
		}

		.queue-item-number {
			min-width: 25px;
		}

		.queue-section {
			max-height: 200px;
		}
	}
</style>
