<script>
    import { onMount, onDestroy } from "svelte";

    // Component state
    let mounted = false;
    let downloadStats = null;
    let activeDownloads = [];
    let isLoading = true;
    let error = null;
    let updateInterval = null;

    onMount(() => {
        mounted = true;
        loadDownloadData();
        // Start auto-refresh every 3 seconds
        updateInterval = setInterval(loadDownloadData, 3000);
    });

    onDestroy(() => {
        if (updateInterval) {
            clearInterval(updateInterval);
        }
    });

    // Load download statistics and active downloads
    async function loadDownloadData() {
        try {
            // Load stats and active downloads in parallel
            const [statsResponse, activeResponse] = await Promise.all([
                fetch("/api/v1/download/stats"),
                fetch("/api/v1/download/active"),
            ]);

            if (statsResponse.ok) {
                const statsData = await statsResponse.json();
                if (statsData.success) {
                    downloadStats = statsData.stats;
                }
            }

            if (activeResponse.ok) {
                const activeData = await activeResponse.json();
                if (activeData.success) {
                    activeDownloads = activeData.downloads || [];
                }
            }

            error = null;
        } catch (err) {
            console.error("Failed to load download data:", err);
            error = "Failed to load download information";
        } finally {
            isLoading = false;
        }
    }

    // Pause a download
    async function pauseDownload(download) {
        try {
            const response = await fetch(
                `/api/v1/download/pause/${download.torrent_hash}`,
                {
                    method: "POST",
                },
            );
            const data = await response.json();

            if (data.success) {
                console.log("Download paused successfully");
                await loadDownloadData(); // Refresh data
            } else {
                console.error("Failed to pause download:", data.error);
            }
        } catch (error) {
            console.error("Error pausing download:", error);
        }
    }

    // Resume a download
    async function resumeDownload(download) {
        try {
            const response = await fetch(
                `/api/v1/download/resume/${download.torrent_hash}`,
                {
                    method: "POST",
                },
            );
            const data = await response.json();

            if (data.success) {
                console.log("Download resumed successfully");
                await loadDownloadData(); // Refresh data
            } else {
                console.error("Failed to resume download:", data.error);
            }
        } catch (error) {
            console.error("Error resuming download:", error);
        }
    }

    // Cancel a download
    async function cancelDownload(download) {
        if (
            !confirm(
                `Are you sure you want to cancel the download of "${download.name}"?`,
            )
        ) {
            return;
        }

        try {
            const response = await fetch(
                `/api/v1/download/cancel/${download.torrent_hash}`,
                {
                    method: "POST",
                },
            );
            const data = await response.json();

            if (data.success) {
                console.log("Download cancelled successfully");
                await loadDownloadData(); // Refresh data
            } else {
                console.error("Failed to cancel download:", data.error);
            }
        } catch (error) {
            console.error("Error cancelling download:", error);
        }
    }

    // Format bytes to human readable format
    function formatBytes(bytes) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }

    // Format speed
    function formatSpeed(bytesPerSecond) {
        return formatBytes(bytesPerSecond) + "/s";
    }

    // Format time (seconds to human readable)
    function formatTime(seconds) {
        if (seconds < 0 || !isFinite(seconds)) return "Unknown";
        if (seconds < 60) return `${Math.round(seconds)}s`;
        if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
        return `${Math.round(seconds / 3600)}h`;
    }

    // Get status color
    function getStatusColor(status) {
        switch (status.toLowerCase()) {
            case "downloading":
                return "#ffa500";
            case "seeding":
                return "#32cd32";
            case "paused":
                return "#ffd700";
            case "completed":
                return "#32cd32";
            case "error":
                return "#dc143c";
            default:
                return "#00bfff";
        }
    }
</script>

<svelte:head>
    <title>Downloads - StepheyBot Music</title>
</svelte:head>

<div class="downloads-page" class:loaded={mounted}>
    <div class="page-header">
        <h1>🎵 Downloads</h1>
        <p>Monitor and manage your music downloads</p>
    </div>

    {#if isLoading}
        <div class="loading">
            <div class="spinner"></div>
            <p>Loading download information...</p>
        </div>
    {:else if error}
        <div class="error">
            <h3>⚠️ Error</h3>
            <p>{error}</p>
            <button class="retry-btn" on:click={loadDownloadData}>
                🔄 Retry
            </button>
        </div>
    {:else}
        <!-- Download Statistics -->
        {#if downloadStats}
            <div class="stats-section">
                <h2>📊 Download Statistics</h2>
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-value">{downloadStats.total_downloads}</div>
                        <div class="stat-label">Total Downloads</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">{downloadStats.active_downloads}</div>
                        <div class="stat-label">Active</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">{downloadStats.completed_downloads}</div>
                        <div class="stat-label">Completed</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">{downloadStats.queued_downloads}</div>
                        <div class="stat-label">Queued</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">{downloadStats.failed_downloads}</div>
                        <div class="stat-label">Failed</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">
                            {formatBytes(downloadStats.total_downloaded_bytes)}
                        </div>
                        <div class="stat-label">Downloaded</div>
                    </div>
                </div>
            </div>
        {/if}

        <!-- Active Downloads -->
        <div class="active-section">
            <h2>⬇️ Active Downloads</h2>

            {#if activeDownloads.length === 0}
                <div class="no-downloads">
                    <div class="no-downloads-icon">📭</div>
                    <h3>No Active Downloads</h3>
                    <p>Your downloads will appear here when you start downloading music.</p>
                    <a href="/search" class="search-link">🔍 Search for Music</a>
                </div>
            {:else}
                <div class="downloads-list">
                    {#each activeDownloads as download}
                        <div class="download-item">
                            <div class="download-header">
                                <div class="download-name">
                                    <h4>{download.name}</h4>
                                    <div class="download-meta">
                                        <span class="download-size">{formatBytes(download.size)}</span>
                                        <span class="download-status" style="color: {getStatusColor(download.status)}">
                                            {download.status.toUpperCase()}
                                        </span>
                                    </div>
                                </div>
                                <div class="download-controls">
                                    {#if download.status === "downloading"}
                                        <button
                                            class="control-btn pause-btn"
                                            on:click={() => pauseDownload(download)}
                                            title="Pause Download"
                                        >
                                            ⏸️
                                        </button>
                                    {:else if download.status === "paused"}
                                        <button
                                            class="control-btn resume-btn"
                                            on:click={() => resumeDownload(download)}
                                            title="Resume Download"
                                        >
                                            ▶️
                                        </button>
                                    {/if}
                                    <button
                                        class="control-btn cancel-btn"
                                        on:click={() => cancelDownload(download)}
                                        title="Cancel Download"
                                    >
                                        ❌
                                    </button>
                                </div>
                            </div>

                            <div class="progress-section">
                                <div class="progress-bar">
                                    <div
                                        class="progress-fill"
                                        style="width: {download.progress * 100}%; background-color: {getStatusColor(download.status)}"
                                    ></div>
                                </div>
                                <div class="progress-text">
                                    {Math.round(download.progress * 100)}%
                                </div>
                            </div>

                            <div class="download-details">
                                <div class="detail-row">
                                    <span>Downloaded:</span>
                                    <span>{formatBytes(download.downloaded)} of {formatBytes(download.size)}</span>
                                </div>
                                <div class="detail-row">
                                    <span>Speed:</span>
                                    <span>↓ {formatSpeed(download.download_speed)} ↑ {formatSpeed(download.upload_speed)}</span>
                                </div>
                                <div class="detail-row">
                                    <span>Peers:</span>
                                    <span>{download.seeds} seeds, {download.peers} peers</span>
                                </div>
                                {#if download.eta > 0}
                                    <div class="detail-row">
                                        <span>ETA:</span>
                                        <span>{formatTime(download.eta)}</span>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    {/if}
</div>

<style>
    .downloads-page {
        opacity: 0;
        transition: opacity 0.5s ease-in-out;
        max-width: 1200px;
        margin: 0 auto;
        padding: 20px;
        min-height: 100vh;
    }

    .downloads-page.loaded {
        opacity: 1;
    }

    .page-header {
        text-align: center;
        margin-bottom: 40px;
    }

    .page-header h1 {
        font-size: 2.5rem;
        font-weight: bold;
        background: linear-gradient(45deg, #ff00ff, #00ffff);
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
        margin-bottom: 10px;
    }

    .page-header p {
        color: rgba(255, 255, 255, 0.7);
        font-size: 1.1rem;
    }

    .loading {
        text-align: center;
        padding: 60px 20px;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 4px solid rgba(255, 255, 255, 0.1);
        border-left: 4px solid #00ffff;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin: 0 auto 20px;
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }

    .error {
        text-align: center;
        padding: 40px 20px;
        background: rgba(220, 20, 60, 0.1);
        border: 1px solid #dc143c;
        border-radius: 12px;
        margin: 20px 0;
    }

    .retry-btn {
        background: rgba(0, 191, 255, 0.1);
        border: 1px solid #00bfff;
        color: #00bfff;
        padding: 10px 20px;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.3s ease;
        margin-top: 15px;
    }

    .retry-btn:hover {
        background: rgba(0, 191, 255, 0.2);
        transform: translateY(-1px);
    }

    .stats-section {
        margin-bottom: 40px;
    }

    .stats-section h2 {
        color: #ff00ff;
        margin-bottom: 20px;
        font-size: 1.8rem;
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 20px;
        margin-bottom: 20px;
    }

    .stat-card {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 20px;
        text-align: center;
        transition: all 0.3s ease;
    }

    .stat-card:hover {
        transform: translateY(-2px);
        border-color: #00ffff;
        box-shadow: 0 5px 20px rgba(0, 255, 255, 0.1);
    }

    .stat-value {
        font-size: 2rem;
        font-weight: bold;
        color: #00ffff;
        margin-bottom: 5px;
    }

    .stat-label {
        color: rgba(255, 255, 255, 0.7);
        font-size: 0.9rem;
    }

    .active-section h2 {
        color: #00ffff;
        margin-bottom: 20px;
        font-size: 1.8rem;
    }

    .no-downloads {
        text-align: center;
        padding: 60px 20px;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
    }

    .no-downloads-icon {
        font-size: 4rem;
        margin-bottom: 20px;
    }

    .no-downloads h3 {
        color: rgba(255, 255, 255, 0.8);
        margin-bottom: 10px;
    }

    .no-downloads p {
        color: rgba(255, 255, 255, 0.6);
        margin-bottom: 20px;
    }

    .search-link {
        display: inline-block;
        background: rgba(0, 191, 255, 0.1);
        border: 1px solid #00bfff;
        color: #00bfff;
        padding: 12px 24px;
        border-radius: 8px;
        text-decoration: none;
        transition: all 0.3s ease;
    }

    .search-link:hover {
        background: rgba(0, 191, 255, 0.2);
        transform: translateY(-1px);
    }

    .downloads-list {
        display: flex;
        flex-direction: column;
        gap: 20px;
    }

    .download-item {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 20px;
        transition: all 0.3s ease;
    }

    .download-item:hover {
        border-color: rgba(0, 255, 255, 0.3);
        box-shadow: 0 5px 20px rgba(0, 255, 255, 0.1);
    }

    .download-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 15px;
    }

    .download-name h4 {
        color: #ffffff;
        margin: 0 0 8px 0;
        font-size: 1.1rem;
    }

    .download-meta {
        display: flex;
        gap: 15px;
        font-size: 0.9rem;
    }

    .download-size {
        color: rgba(255, 255, 255, 0.7);
    }

    .download-status {
        font-weight: bold;
    }

    .download-controls {
        display: flex;
        gap: 8px;
    }

    .control-btn {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        color: #ffffff;
        padding: 6px 10px;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.3s ease;
        font-size: 14px;
    }

    .control-btn:hover {
        background: rgba(255, 255, 255, 0.2);
        transform: translateY(-1px);
    }

    .pause-btn:hover {
        border-color: #ffd700;
        color: #ffd700;
    }

    .resume-btn:hover {
        border-color: #32cd32;
        color: #32cd32;
    }

    .cancel-btn:hover {
        border-color: #dc143c;
        color: #dc143c;
    }

    .progress-section {
        display: flex;
        align-items: center;
        gap: 15px;
        margin-bottom: 15px;
    }

    .progress-bar {
        flex: 1;
        height: 8px;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 4px;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        transition: width 0.3s ease;
        border-radius: 4px;
    }

    .progress-text {
        color: #ffffff;
        font-weight: bold;
        font-size: 0.9rem;
        min-width: 50px;
        text-align: right;
    }

    .download-details {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 10px;
        font-size: 0.9rem;
    }

    .detail-row {
        display: flex;
        justify-content: space-between;
        color: rgba(255, 255, 255, 0.8);
    }

    .detail-row span:first-child {
        color: rgba(255, 255, 255, 0.6);
    }

    @media (max-width: 768px) {
        .downloads-page {
            padding: 15px;
        }

        .page-header h1 {
            font-size: 2rem;
        }

        .stats-grid {
            grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
            gap: 15px;
        }

        .download-header {
            flex-direction: column;
            gap: 15px;
        }

        .download-details {
            grid-template-columns: 1fr;
        }
    }
</style>
