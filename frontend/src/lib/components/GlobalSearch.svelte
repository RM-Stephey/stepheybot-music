<script>
    import { onMount } from "svelte";
    import { musicPlayerActions } from "$lib/stores/musicPlayer.js";

    // Component props
    export let placeholder = "Search music globally...";
    export let showFilters = true;

    // Component state
    let mounted = false;
    let searchQuery = "";
    let searchResults = [];
    let isSearching = false;
    let searchError = null;
    let searchType = "global"; // 'global', 'local', 'external'
    let searchCategory = "all"; // 'all', 'artist', 'album', 'track'
    let sortBy = "relevance"; // 'relevance', 'popularity', 'year', 'duration'
    let downloadRequests = new Map(); // Track download request status

    // No more debouncing - search only on submit/enter

    onMount(() => {
        mounted = true;
    });

    // Handle Enter key press
    function handleKeyDown(event) {
        if (event.key === "Enter") {
            handleSearchSubmit();
        }
    }

    // Handle search submit (Enter key or button click)
    function handleSearchSubmit() {
        const query = searchQuery.trim();
        if (query.length >= 2) {
            performSearch(query);
        } else {
            searchResults = [];
            searchError = null;
        }
    }

    // Main search function
    async function performSearch(query) {
        if (!query || query.length < 2) return;

        isSearching = true;
        searchError = null;

        try {
            const baseEndpoint =
                searchType === "external"
                    ? `/api/v1/search/external/${encodeURIComponent(query)}`
                    : `/api/v1/search/global/${encodeURIComponent(query)}`;

            // Add search parameters
            const params = new URLSearchParams();
            params.append("category", searchCategory);
            params.append("type", searchType);

            const endpoint = `${baseEndpoint}?${params.toString()}`;

            const response = await fetch(endpoint);
            const data = await response.json();

            if (data.success) {
                searchResults = sortResults(data.results || []);
            } else {
                searchError = data.message || "Search failed";
                searchResults = [];
            }
        } catch (error) {
            console.error("Search error:", error);
            searchError = "Failed to perform search. Please try again.";
            searchResults = [];
        } finally {
            isSearching = false;
        }
    }

    // Sort search results
    function sortResults(results) {
        return results.sort((a, b) => {
            switch (sortBy) {
                case "popularity":
                    return (b.popularity || 0) - (a.popularity || 0);
                case "year":
                    return (b.year || 0) - (a.year || 0);
                case "duration":
                    return (a.duration || 0) - (b.duration || 0);
                case "relevance":
                default:
                    // Prioritize local results, then by popularity
                    if (a.source === "local" && b.source !== "local") return -1;
                    if (b.source === "local" && a.source !== "local") return 1;
                    return (b.popularity || 0) - (a.popularity || 0);
            }
        });
    }

    // Play track (local only)
    function playTrack(track) {
        if (track.available && track.stream_url) {
            console.log("Playing track from global search:", track.title);
            if (musicPlayerActions && musicPlayerActions.playTrack) {
                musicPlayerActions.playTrack(track);
            } else {
                console.warn("Music player actions not available");
            }
        }
    }

    // Add track to queue (local only)
    function addToQueue(track) {
        if (track.available && track.stream_url) {
            console.log(
                "Adding track to queue from global search:",
                track.title,
            );
            if (musicPlayerActions && musicPlayerActions.addToQueue) {
                musicPlayerActions.addToQueue(track);
            } else {
                console.warn("Music player actions not available");
            }
        }
    }

    // Request download for external tracks
    async function requestDownload(track) {
        const requestId = `${track.artist}-${track.title}`;

        if (downloadRequests.has(requestId)) {
            return; // Already requested
        }

        downloadRequests.set(requestId, "requesting");
        downloadRequests = downloadRequests; // Trigger reactivity

        try {
            const response = await fetch("/api/v1/download/request", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    title: track.title,
                    artist: track.artist,
                    album: track.album,
                    external_id: track.id,
                    source: track.source,
                }),
            });

            const data = await response.json();

            if (data.success) {
                downloadRequests.set(requestId, "monitoring");
                console.log("Download request successful:", data.message);
            } else {
                downloadRequests.set(requestId, "failed");
                console.error("Download request failed:", data.message);
            }
        } catch (error) {
            console.error("Download request error:", error);
            downloadRequests.set(requestId, "failed");
        }

        downloadRequests = downloadRequests; // Trigger reactivity
    }

    // Get download status for a track
    function getDownloadStatus(track) {
        const requestId = `${track.artist}-${track.title}`;
        return downloadRequests.get(requestId) || null;
    }

    // Clear search
    function clearSearch() {
        searchQuery = "";
        searchResults = [];
        searchError = null;
    }

    // Format duration
    function formatDuration(seconds) {
        if (!seconds) return "";
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, "0")}`;
    }

    // Handle search type change
    function handleSearchTypeChange() {
        if (searchQuery.length >= 2 && searchResults.length > 0) {
            handleSearchSubmit();
        }
    }

    // Handle sort change
    function handleSearchCategoryChange() {
        if (searchQuery.length >= 2 && searchResults.length > 0) {
            handleSearchSubmit();
        }
    }

    function handleSortChange() {
        if (searchResults.length > 0) {
            searchResults = sortResults(searchResults);
        }
    }
</script>

<div class="global-search" class:loaded={mounted}>
    <!-- Search Header -->
    <div class="search-header">
        <h2 class="search-title">
            <span class="title-icon">🔍</span>
            Global Music Search
        </h2>
        <p class="search-subtitle">
            Search your library and discover new music from external sources
        </p>
    </div>

    <!-- Search Controls -->
    <div class="search-controls">
        <!-- Search Input -->
        <div class="search-input-container">
            <div class="search-input-wrapper">
                <input
                    type="text"
                    class="search-input"
                    bind:value={searchQuery}
                    on:keydown={handleKeyDown}
                    {placeholder}
                    autocomplete="off"
                />
                <div class="search-icon">🔍</div>
                {#if searchQuery}
                    <button class="clear-search-btn" on:click={clearSearch}
                        >✕</button
                    >
                {/if}
                {#if isSearching}
                    <div class="search-loading">
                        <div class="loading-spinner"></div>
                    </div>
                {/if}
            </div>

            <!-- Search Submit Button -->
            <button
                class="search-submit-btn"
                on:click={handleSearchSubmit}
                disabled={searchQuery.trim().length < 2 || isSearching}
            >
                {#if isSearching}
                    Searching...
                {:else}
                    Search
                {/if}
            </button>
        </div>

        <!-- Search Filters -->
        {#if showFilters}
            <div class="search-filters">
                <div class="filter-group">
                    <label class="filter-label">Search Type:</label>
                    <select
                        bind:value={searchType}
                        on:change={handleSearchTypeChange}
                        class="filter-select"
                    >
                        <option value="global">All Sources</option>
                        <option value="local">Local Only</option>
                        <option value="external">External Only</option>
                    </select>
                </div>

                <div class="filter-group">
                    <label class="filter-label">Search for:</label>
                    <select
                        bind:value={searchCategory}
                        on:change={handleSearchCategoryChange}
                        class="filter-select"
                    >
                        <option value="all">Everything</option>
                        <option value="artist">Artists</option>
                        <option value="album">Albums</option>
                        <option value="track">Tracks</option>
                    </select>
                </div>

                <div class="filter-group">
                    <label class="filter-label">Sort by:</label>
                    <select
                        bind:value={sortBy}
                        on:change={handleSortChange}
                        class="filter-select"
                    >
                        <option value="relevance">Relevance</option>
                        <option value="popularity">Popularity</option>
                        <option value="year">Year</option>
                        <option value="duration">Duration</option>
                    </select>
                </div>
            </div>
        {/if}
    </div>

    <!-- Search Results -->
    <div class="search-results">
        {#if searchError}
            <div class="error-message">
                <span class="error-icon">⚠️</span>
                <p>{searchError}</p>
                <button
                    class="retry-btn"
                    on:click={() => performSearch(searchQuery)}
                >
                    Retry Search
                </button>
            </div>
        {:else if searchResults.length > 0}
            <div class="results-header">
                <h3>Search Results ({searchResults.length})</h3>
                <div class="results-legend">
                    <span class="legend-item">
                        <span class="source-badge local">Local</span>
                        Available in library
                    </span>
                    <span class="legend-item">
                        <span class="source-badge external">External</span>
                        Can be downloaded
                    </span>
                </div>
            </div>

            <div class="results-grid">
                {#each searchResults as track}
                    <div
                        class="track-card"
                        class:local={track.source === "local"}
                        class:external={track.source !== "local"}
                    >
                        <!-- Track Info -->
                        <div class="track-info">
                            <h4 class="track-title">{track.title}</h4>
                            <p class="track-artist">{track.artist}</p>
                            <p class="track-album">
                                {track.album || "Unknown Album"}
                            </p>
                            <div class="track-meta">
                                {#if track.year}
                                    <span class="meta-item"
                                        >📅 {track.year}</span
                                    >
                                {/if}
                                {#if track.duration}
                                    <span class="meta-item"
                                        >⏱️ {formatDuration(
                                            track.duration,
                                        )}</span
                                    >
                                {/if}
                                {#if track.genre}
                                    <span class="meta-item genre"
                                        >🎵 {track.genre}</span
                                    >
                                {/if}
                                {#if track.source === "lidarr"}
                                    {#if track.quality}
                                        <span class="meta-item quality"
                                            >🎧 {track.quality}</span
                                        >
                                    {/if}
                                    {#if track.size_mb}
                                        <span class="meta-item size"
                                            >💾 {track.size_mb}MB</span
                                        >
                                    {/if}
                                    {#if track.seeders !== undefined}
                                        <span class="meta-item seeders"
                                            >🌱 {track.seeders}↑ {track.leechers}↓</span
                                        >
                                    {/if}
                                {/if}
                            </div>
                        </div>

                        <!-- Source Badge -->
                        <div class="source-info">
                            <span
                                class="source-badge"
                                class:local={track.source === "local"}
                                class:external={track.source !== "local" &&
                                    track.source !== "lidarr"}
                                class:lidarr={track.source === "lidarr"}
                            >
                                {#if track.source === "local"}
                                    🏠 Local
                                {:else if track.source === "lidarr"}
                                    ⬇️ Download
                                {:else}
                                    🌐 External
                                {/if}
                            </span>
                            {#if track.popularity}
                                <span class="popularity">
                                    {"★".repeat(
                                        Math.ceil(track.popularity / 20),
                                    )}
                                </span>
                            {/if}
                            {#if track.source === "lidarr" && track.indexer}
                                <span class="indexer">📡 {track.indexer}</span>
                            {/if}
                        </div>

                        <!-- Track Actions -->
                        <div class="track-actions">
                            {#if track.available && track.stream_url}
                                <button
                                    class="action-btn play-btn"
                                    on:click={() => playTrack(track)}
                                    title="Play Track"
                                >
                                    ▶️ Play
                                </button>
                                <button
                                    class="action-btn queue-btn"
                                    on:click={() => addToQueue(track)}
                                    title="Add to Queue"
                                >
                                    ➕ Queue
                                </button>
                            {:else if track.source === "lidarr" && track.downloadable}
                                {#if getDownloadStatus(track) === "requesting"}
                                    <button
                                        class="action-btn requesting"
                                        disabled
                                    >
                                        ⏳ Requesting...
                                    </button>
                                {:else if getDownloadStatus(track) === "monitoring"}
                                    <button
                                        class="action-btn monitoring"
                                        disabled
                                    >
                                        👁️ Monitoring
                                    </button>
                                {:else if getDownloadStatus(track) === "failed"}
                                    <button
                                        class="action-btn download-btn"
                                        on:click={() => requestDownload(track)}
                                        title="Retry Download"
                                    >
                                        🔄 Retry Download
                                    </button>
                                {:else}
                                    <button
                                        class="action-btn download-btn lidarr-download"
                                        on:click={() => requestDownload(track)}
                                        title="Download {track.quality} - {track.size_mb}MB"
                                    >
                                        ⬇️ Get {track.quality}
                                    </button>
                                {/if}

                                {#if track.magnet_url}
                                    <a
                                        href={track.magnet_url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="action-btn external-link magnet-link"
                                        title="Open Magnet Link"
                                    >
                                        🧲 Magnet
                                    </a>
                                {/if}
                            {:else}
                                {#if getDownloadStatus(track) === "requesting"}
                                    <button
                                        class="action-btn requesting"
                                        disabled
                                    >
                                        ⏳ Requesting...
                                    </button>
                                {:else if getDownloadStatus(track) === "monitoring"}
                                    <button
                                        class="action-btn monitoring"
                                        disabled
                                    >
                                        👁️ Monitoring
                                    </button>
                                {:else if getDownloadStatus(track) === "failed"}
                                    <button
                                        class="action-btn download-btn"
                                        on:click={() => requestDownload(track)}
                                        title="Retry Download"
                                    >
                                        🔄 Retry Download
                                    </button>
                                {:else}
                                    <button
                                        class="action-btn download-btn"
                                        on:click={() => requestDownload(track)}
                                        title="Request Download"
                                    >
                                        ⬇️ Download
                                    </button>
                                {/if}

                                {#if track.external_url}
                                    <a
                                        href={track.external_url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="action-btn external-link"
                                        title="Open External Link"
                                    >
                                        🔗 Open
                                    </a>
                                {/if}
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {:else if searchQuery.length >= 2 && !isSearching}
            <div class="no-results">
                <span class="no-results-icon">🔍</span>
                <h3>No Results Found</h3>
                <p>
                    No tracks found for "{searchQuery}". Try different keywords
                    or check external sources.
                </p>
            </div>
        {:else if searchQuery.length < 2}
            <div class="search-help">
                <span class="help-icon">💡</span>
                <h3>Start Searching</h3>
                <p>
                    Enter at least 2 characters to search for music across your
                    library and external sources.
                </p>
            </div>
        {/if}
    </div>
</div>

<style>
    .global-search {
        opacity: 0;
        transition: opacity 0.5s ease-in-out;
        max-width: 1200px;
        margin: 0 auto;
        padding: 20px;
    }

    .global-search.loaded {
        opacity: 1;
    }

    /* Header */
    .search-header {
        text-align: center;
        margin-bottom: 30px;
    }

    .search-title {
        font-size: 2.5rem;
        font-weight: bold;
        background: linear-gradient(135deg, #ff6b9d, #00ffff);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        margin-bottom: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 15px;
    }

    .title-icon {
        font-size: 2rem;
    }

    .search-subtitle {
        color: #ccc;
        font-size: 1.1rem;
        margin-bottom: 20px;
    }

    /* Search Controls */
    .search-controls {
        margin-bottom: 30px;
    }

    .search-input-container {
        margin-bottom: 20px;
        display: flex;
        gap: 12px;
        align-items: stretch;
    }

    .search-input-wrapper {
        position: relative;
        width: 100%;
        max-width: 600px;
        flex: 1;
    }

    .search-input {
        width: 100%;
        padding: 15px 50px 15px 20px;
        background: rgba(20, 20, 30, 0.8);
        border: 2px solid rgba(0, 255, 255, 0.3);
        border-radius: 25px;
        color: #fff;
        font-size: 1.1rem;
        transition: all 0.3s ease;
    }

    .search-input:focus {
        outline: none;
        border-color: #00ffff;
        box-shadow: 0 0 20px rgba(0, 255, 255, 0.3);
    }

    .search-input::placeholder {
        color: #888;
    }

    .search-icon {
        position: absolute;
        right: 15px;
        top: 50%;
        transform: translateY(-50%);
        color: #00ffff;
        font-size: 1.2rem;
    }

    .clear-search-btn {
        position: absolute;
        right: 45px;
        top: 50%;
        transform: translateY(-50%);
        background: none;
        border: none;
        color: #ff6b9d;
        font-size: 1.2rem;
        cursor: pointer;
        padding: 5px;
        border-radius: 50%;
        transition: all 0.2s ease;
    }

    .clear-search-btn:hover {
        background: rgba(255, 107, 157, 0.2);
    }

    .search-submit-btn {
        background: linear-gradient(135deg, #ff6b9d, #00ffff);
        color: white;
        border: none;
        padding: 12px 24px;
        border-radius: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
        white-space: nowrap;
        min-width: 100px;
        height: 48px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .search-submit-btn:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 8px 25px rgba(0, 255, 255, 0.3);
    }

    .search-submit-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
        transform: none;
    }

    .search-loading {
        position: absolute;
        right: 80px;
        top: 50%;
        transform: translateY(-50%);
    }

    .loading-spinner {
        width: 20px;
        height: 20px;
        border: 2px solid rgba(0, 255, 255, 0.3);
        border-top: 2px solid #00ffff;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        0% {
            transform: rotate(0deg);
        }
        100% {
            transform: rotate(360deg);
        }
    }

    /* Search Filters */
    .search-filters {
        display: flex;
        justify-content: center;
        gap: 20px;
        flex-wrap: wrap;
    }

    .filter-group {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .filter-label {
        color: #ccc;
        font-size: 0.9rem;
        white-space: nowrap;
    }

    .filter-select {
        background: rgba(20, 20, 30, 0.8);
        border: 1px solid rgba(0, 255, 255, 0.3);
        border-radius: 8px;
        color: #fff;
        padding: 8px 12px;
        font-size: 0.9rem;
        cursor: pointer;
        transition: all 0.3s ease;
    }

    .filter-select:focus {
        outline: none;
        border-color: #00ffff;
    }

    /* Results */
    .results-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 20px;
        flex-wrap: wrap;
        gap: 15px;
    }

    .results-header h3 {
        color: #fff;
        font-size: 1.3rem;
        margin: 0;
    }

    .results-legend {
        display: flex;
        gap: 15px;
        font-size: 0.9rem;
        color: #ccc;
    }

    .legend-item {
        display: flex;
        align-items: center;
        gap: 5px;
    }

    .results-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
        gap: 20px;
    }

    .track-card {
        background: rgba(20, 20, 30, 0.9);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 20px;
        transition: all 0.3s ease;
        position: relative;
        overflow: hidden;
    }

    .track-card::before {
        content: "";
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 3px;
        background: linear-gradient(
            90deg,
            transparent,
            rgba(0, 255, 255, 0.5),
            transparent
        );
        opacity: 0;
        transition: opacity 0.3s ease;
    }

    .track-card:hover {
        transform: translateY(-5px);
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        border-color: rgba(0, 255, 255, 0.3);
    }

    .track-card:hover::before {
        opacity: 1;
    }

    .track-card.local {
        border-left: 3px solid #00ff88;
    }

    .track-card.external {
        border-left: 3px solid #ff6b9d;
    }

    .track-card.lidarr {
        border-left: 3px solid #ffa500;
    }

    .track-info {
        margin-bottom: 15px;
    }

    .track-title {
        font-size: 1.2rem;
        font-weight: bold;
        color: #fff;
        margin-bottom: 5px;
        line-height: 1.3;
    }

    .track-artist {
        font-size: 1rem;
        color: #00ffff;
        margin-bottom: 3px;
    }

    .track-album {
        font-size: 0.9rem;
        color: #ccc;
        margin-bottom: 8px;
    }

    .track-meta {
        display: flex;
        gap: 10px;
        flex-wrap: wrap;
    }

    .meta-item {
        font-size: 0.8rem;
        color: #999;
        background: rgba(255, 255, 255, 0.1);
        padding: 2px 8px;
        border-radius: 12px;
    }

    .meta-item.genre {
        background: rgba(255, 107, 157, 0.2);
        color: #ff6b9d;
    }

    .meta-item.quality {
        background: rgba(0, 255, 136, 0.2);
        color: #00ff88;
    }

    .meta-item.size {
        background: rgba(0, 255, 255, 0.2);
        color: #00ffff;
    }

    .meta-item.seeders {
        background: rgba(255, 165, 0, 0.2);
        color: #ffa500;
    }

    .source-info {
        position: absolute;
        top: 15px;
        right: 15px;
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 5px;
    }

    .source-badge {
        font-size: 0.8rem;
        padding: 4px 8px;
        border-radius: 12px;
        font-weight: bold;
        text-transform: uppercase;
    }

    .source-badge.local {
        background: rgba(0, 255, 136, 0.2);
        color: #00ff88;
        border: 1px solid #00ff88;
    }

    .source-badge.external {
        background: rgba(255, 107, 157, 0.2);
        color: #ff6b9d;
        border: 1px solid #ff6b9d;
    }

    .source-badge.lidarr {
        background: rgba(255, 165, 0, 0.2);
        color: #ffa500;
        border: 1px solid #ffa500;
    }

    .popularity {
        font-size: 0.7rem;
        color: #999;
    }

    .indexer {
        font-size: 0.7rem;
        color: #ffa500;
        background: rgba(255, 165, 0, 0.1);
        padding: 2px 6px;
        border-radius: 8px;
        border: 1px solid rgba(255, 165, 0, 0.3);
    }

    .track-actions {
        display: flex;
        gap: 10px;
        flex-wrap: wrap;
    }

    .action-btn {
        background: rgba(0, 255, 255, 0.1);
        border: 1px solid #00ffff;
        color: #00ffff;
        padding: 8px 12px;
        border-radius: 8px;
        cursor: pointer;
        font-size: 0.9rem;
        font-weight: bold;
        transition: all 0.2s ease;
        text-decoration: none;
        display: inline-flex;
        align-items: center;
        gap: 5px;
    }

    .action-btn:hover {
        background: rgba(0, 255, 255, 0.2);
        transform: translateY(-2px);
    }

    .action-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
        transform: none;
    }

    .action-btn.play-btn {
        background: rgba(0, 255, 136, 0.1);
        border-color: #00ff88;
        color: #00ff88;
    }

    .action-btn.play-btn:hover {
        background: rgba(0, 255, 136, 0.2);
    }

    .action-btn.queue-btn {
        background: rgba(255, 107, 157, 0.1);
        border-color: #ff6b9d;
        color: #ff6b9d;
    }

    .action-btn.queue-btn:hover {
        background: rgba(255, 107, 157, 0.2);
    }

    .action-btn.download-btn {
        background: rgba(255, 165, 0, 0.1);
        border-color: #ffa500;
        color: #ffa500;
    }

    .action-btn.download-btn:hover {
        background: rgba(255, 165, 0, 0.2);
    }

    .action-btn.requesting {
        background: rgba(255, 255, 0, 0.1);
        border-color: #ffff00;
        color: #ffff00;
    }

    .action-btn.monitoring {
        background: rgba(0, 255, 255, 0.1);
        border-color: #00ffff;
        color: #00ffff;
    }

    .action-btn.external-link {
        background: rgba(128, 128, 128, 0.1);
        border-color: #888;
        color: #888;
    }

    .action-btn.external-link:hover {
        background: rgba(128, 128, 128, 0.2);
    }

    .action-btn.lidarr-download {
        background: rgba(255, 165, 0, 0.1);
        border-color: #ffa500;
        color: #ffa500;
        font-weight: bold;
    }

    .action-btn.lidarr-download:hover {
        background: rgba(255, 165, 0, 0.2);
        box-shadow: 0 0 15px rgba(255, 165, 0, 0.3);
    }

    .action-btn.magnet-link {
        background: rgba(138, 43, 226, 0.1);
        border-color: #8a2be2;
        color: #8a2be2;
    }

    .action-btn.magnet-link:hover {
        background: rgba(138, 43, 226, 0.2);
    }

    /* Error and Empty States */
    .error-message,
    .no-results,
    .search-help {
        text-align: center;
        padding: 60px 20px;
        color: #ccc;
    }

    .error-icon,
    .no-results-icon,
    .help-icon {
        font-size: 3rem;
        margin-bottom: 20px;
        display: block;
    }

    .error-message h3,
    .no-results h3,
    .search-help h3 {
        color: #fff;
        font-size: 1.5rem;
        margin-bottom: 10px;
    }

    .retry-btn {
        background: rgba(255, 107, 157, 0.1);
        border: 1px solid #ff6b9d;
        color: #ff6b9d;
        padding: 10px 20px;
        border-radius: 8px;
        cursor: pointer;
        font-weight: bold;
        margin-top: 15px;
        transition: all 0.2s ease;
    }

    .retry-btn:hover {
        background: rgba(255, 107, 157, 0.2);
    }

    /* Mobile Responsive */
    @media (max-width: 768px) {
        .global-search {
            padding: 15px;
        }

        .search-title {
            font-size: 2rem;
            flex-direction: column;
            gap: 10px;
        }

        .search-filters {
            flex-direction: column;
            align-items: center;
        }

        .results-header {
            flex-direction: column;
            align-items: flex-start;
        }

        .results-legend {
            flex-direction: column;
            gap: 8px;
        }

        .results-grid {
            grid-template-columns: 1fr;
        }

        .track-actions {
            justify-content: center;
        }

        .action-btn {
            font-size: 0.8rem;
            padding: 6px 10px;
        }
    }
</style>
