<script>
    import "../app.css";
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import MusicPlayer from "$lib/components/MusicPlayer.svelte";
    import FloatingPlayer from "$lib/components/FloatingPlayer.svelte";
    import { setMusicPlayer } from "$lib/stores/musicPlayer.js";

    let mounted = false;
    let currentTime = new Date().toLocaleTimeString();
    let musicPlayer;

    onMount(() => {
        mounted = true;

        // Update time every second
        const interval = setInterval(() => {
            currentTime = new Date().toLocaleTimeString();
        }, 1000);

        return () => clearInterval(interval);
    });

    // Set music player reference in store when it's available
    $: if (musicPlayer) {
        setMusicPlayer(musicPlayer);
    }

    // Navigation items
    const navItems = [
        { href: "/", label: "Dashboard", icon: "🏠" },
        { href: "/search", label: "Search", icon: "🔍" },
        { href: "/downloads", label: "Downloads", icon: "⬇️" },
        { href: "/discover", label: "Discover", icon: "🎲" },
        { href: "/recommendations", label: "Recommendations", icon: "🎵" },
        { href: "/playlists", label: "Playlists", icon: "📝" },
        { href: "/library", label: "Library", icon: "📚" },
        { href: "/stats", label: "Stats", icon: "📊" },
    ];

    $: currentPath = $page.url.pathname;
</script>

<svelte:head>
    <title>StepheyBot Music - AI-Powered Music Recommendations</title>
</svelte:head>

<div class="app-layout" class:loaded={mounted}>
    <!-- Header -->
    <header class="header">
        <div class="container">
            <div class="header-content">
                <!-- Logo/Brand -->
                <div class="brand">
                    <h1 class="brand-title">
                        <span class="brand-icon">🤖</span>
                        <span class="brand-text">StepheyBot</span>
                        <span class="brand-subtitle">Music</span>
                    </h1>
                </div>

                <!-- Navigation -->
                <nav class="nav" aria-label="Main navigation">
                    <ul class="nav-list">
                        {#each navItems as item}
                            <li class="nav-item">
                                <a
                                    href={item.href}
                                    class="nav-link"
                                    class:active={currentPath === item.href}
                                    aria-current={currentPath === item.href
                                        ? "page"
                                        : undefined}
                                >
                                    <span class="nav-icon" aria-hidden="true"
                                        >{item.icon}</span
                                    >
                                    <span class="nav-label">{item.label}</span>
                                </a>
                            </li>
                        {/each}
                    </ul>
                </nav>

                <!-- Status Info -->
                <div class="status-info">
                    <div class="status-time">
                        <span class="status-label">System Time</span>
                        <span class="status-value text-glow">{currentTime}</span
                        >
                    </div>
                    <div class="status-indicator">
                        <div class="pulse-dot"></div>
                        <span class="status-text">Online</span>
                    </div>
                </div>
            </div>
        </div>
    </header>

    <!-- Main Content -->
    <main class="main-content">
        <div class="content-wrapper">
            <slot />
        </div>
    </main>

    <!-- Footer -->
    <footer class="footer">
        <div class="container">
            <div class="footer-content">
                <div class="footer-info">
                    <p class="footer-text">
                        Powered by <span class="text-neon-cyan"
                            >StepheyBot AI</span
                        >
                        • Built with <span class="text-neon-pink">Svelte</span>
                        &
                        <span class="text-neon-purple">Rust</span>
                    </p>
                </div>
                <div class="footer-links">
                    <a href="/health" class="footer-link">System Health</a>
                    <a href="/api/v1/status" class="footer-link">API Status</a>
                    <a
                        href="https://github.com/RM-Stephey/stepheybot-music"
                        class="footer-link"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        GitHub
                    </a>
                </div>
            </div>
        </div>
    </footer>

    <!-- Floating Music Player -->
    <FloatingPlayer {musicPlayer} />

    <!-- Hidden Music Player for functionality -->
    <div style="display: none;">
        <MusicPlayer bind:this={musicPlayer} />
    </div>
</div>

<style>
    .app-layout {
        min-height: 100vh;
        display: flex;
        flex-direction: column;
        opacity: 0;
        transition: opacity 0.5s ease-in-out;
    }

    .app-layout.loaded {
        opacity: 1;
    }

    /* Header Styles */
    .header {
        background: rgba(10, 10, 15, 0.95);
        backdrop-filter: blur(10px);
        border-bottom: 1px solid rgba(0, 255, 255, 0.2);
        position: sticky;
        top: 0;
        z-index: 100;
        box-shadow: 0 2px 20px rgba(0, 255, 255, 0.1);
    }

    .header-content {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: var(--spacing-md) 0;
        gap: var(--spacing-lg);
    }

    /* Brand Styles */
    .brand {
        flex-shrink: 0;
    }

    .brand-title {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        margin: 0;
        font-size: 1.5rem;
        font-weight: 800;
        font-family: var(--font-primary);
    }

    .brand-icon {
        font-size: 2rem;
        filter: drop-shadow(0 0 10px var(--neon-cyan));
    }

    .brand-text {
        background: linear-gradient(45deg, var(--neon-cyan), var(--neon-pink));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
        text-shadow: 0 0 20px var(--neon-cyan);
    }

    .brand-subtitle {
        color: var(--neon-purple);
        font-weight: 600;
        text-shadow: 0 0 15px var(--neon-purple);
    }

    /* Navigation Styles */
    .nav-list {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        list-style: none;
        margin: 0;
        padding: 0;
    }

    .nav-link {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        padding: var(--spacing-sm) var(--spacing-md);
        font-family: var(--font-primary);
        font-weight: 500;
        font-size: 0.875rem;
        color: var(--text-secondary);
        text-decoration: none;
        border-radius: var(--border-radius);
        transition: all var(--transition-normal);
        position: relative;
        overflow: hidden;
    }

    .nav-link::before {
        content: "";
        position: absolute;
        top: 0;
        left: -100%;
        width: 100%;
        height: 100%;
        background: linear-gradient(
            90deg,
            transparent,
            rgba(0, 255, 255, 0.1),
            transparent
        );
        transition: left var(--transition-slow);
    }

    .nav-link:hover::before {
        left: 100%;
    }

    .nav-link:hover {
        color: var(--neon-cyan);
        background: rgba(0, 255, 255, 0.1);
        text-shadow: 0 0 10px var(--neon-cyan);
        transform: translateY(-2px);
    }

    .nav-link.active {
        color: var(--neon-cyan);
        background: rgba(0, 255, 255, 0.15);
        text-shadow: 0 0 10px var(--neon-cyan);
        border: 1px solid rgba(0, 255, 255, 0.3);
    }

    .nav-icon {
        font-size: 1.1rem;
        filter: drop-shadow(0 0 5px currentColor);
    }

    .nav-label {
        font-weight: 600;
        letter-spacing: 0.05em;
    }

    /* Status Info */
    .status-info {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
        flex-shrink: 0;
    }

    .status-time {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 2px;
    }

    .status-label {
        font-size: 0.75rem;
        color: var(--text-muted);
        font-family: var(--font-primary);
        text-transform: uppercase;
        letter-spacing: 0.1em;
    }

    .status-value {
        font-family: var(--font-primary);
        font-weight: 600;
        font-size: 0.875rem;
    }

    .status-indicator {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        padding: var(--spacing-xs) var(--spacing-sm);
        background: rgba(0, 255, 0, 0.1);
        border: 1px solid rgba(0, 255, 0, 0.3);
        border-radius: var(--border-radius);
    }

    .pulse-dot {
        width: 8px;
        height: 8px;
        background: #00ff00;
        border-radius: 50%;
        animation: pulse 2s ease-in-out infinite;
        box-shadow: 0 0 10px #00ff00;
    }

    .status-text {
        font-size: 0.75rem;
        color: #00ff00;
        font-weight: 600;
        font-family: var(--font-primary);
        text-transform: uppercase;
        letter-spacing: 0.1em;
    }

    @keyframes pulse {
        0%,
        100% {
            opacity: 1;
            transform: scale(1);
        }
        50% {
            opacity: 0.5;
            transform: scale(0.8);
        }
    }

    /* Main Content */
    .main-content {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    .content-wrapper {
        flex: 1;
        padding: var(--spacing-lg) 0;
    }

    /* Footer */
    .footer {
        background: rgba(10, 10, 15, 0.8);
        backdrop-filter: blur(10px);
        border-top: 1px solid rgba(0, 255, 255, 0.2);
        margin-top: auto;
    }

    .footer-content {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: var(--spacing-md) 0;
        gap: var(--spacing-md);
    }

    .footer-text {
        font-size: 0.875rem;
        color: var(--text-muted);
        margin: 0;
    }

    .footer-links {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
    }

    .footer-link {
        font-size: 0.875rem;
        color: var(--text-secondary);
        text-decoration: none;
        transition: color var(--transition-fast);
    }

    .footer-link:hover {
        color: var(--neon-cyan);
        text-shadow: 0 0 10px var(--neon-cyan);
    }

    /* Responsive Design */
    @media (max-width: 768px) {
        .header-content {
            flex-direction: column;
            gap: var(--spacing-md);
        }

        .brand-title {
            font-size: 1.2rem;
        }

        .brand-icon {
            font-size: 1.5rem;
        }

        .brand-text {
            font-size: 1.2rem;
        }

        .brand-subtitle {
            font-size: 0.8rem;
        }

        .nav-list {
            flex-wrap: wrap;
            justify-content: center;
        }

        .nav-link {
            padding: var(--spacing-xs) var(--spacing-sm);
            font-size: 0.8rem;
        }

        .nav-label {
            display: none;
        }

        .nav-icon {
            font-size: 1.2rem;
        }

        .status-info {
            flex-direction: column;
            gap: var(--spacing-sm);
        }

        .footer-content {
            flex-direction: column;
            text-align: center;
            gap: var(--spacing-sm);
        }

        .footer-links {
            justify-content: center;
        }
    }

    @media (max-width: 480px) {
        .brand-title {
            font-size: 1rem;
        }

        .brand-icon {
            font-size: 1.2rem;
        }

        .brand-text {
            font-size: 1rem;
        }

        .brand-subtitle {
            font-size: 0.7rem;
        }

        .header-content {
            padding: var(--spacing-sm) 0;
        }
    }

    @media (max-width: 480px) {
        .brand-title {
            font-size: 1.25rem;
        }

        .brand-subtitle {
            display: none;
        }

        .nav-list {
            gap: var(--spacing-xs);
        }

        .nav-link {
            padding: var(--spacing-xs);
        }

        .content-wrapper {
            padding: var(--spacing-md) 0;
        }
    }
</style>
