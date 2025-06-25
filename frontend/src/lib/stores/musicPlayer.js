import { writable } from "svelte/store";

// Debug: Log when store file is loaded
console.log("🔧 MusicPlayer Store: File loading...");

// Create a writable store for the music player instance
export const musicPlayerStore = writable(null);

console.log("🔧 MusicPlayer Store: musicPlayerStore created");

// Helper functions to interact with the music player
const playTrack = (track) => {
    console.log("🎵 Store: playTrack called with:", track?.title);
    musicPlayerStore.update((player) => {
        if (player && player.playTrackFromParent) {
            player.playTrackFromParent(track);
        } else {
            console.warn("⚠️ Store: Music player not available for playTrack");
        }
        return player;
    });
};

const addToQueue = (track) => {
    console.log("📝 Store: addToQueue called with:", track?.title);
    musicPlayerStore.update((player) => {
        if (player && player.addTrackToQueue) {
            player.addTrackToQueue(track);
        } else {
            console.warn("⚠️ Store: Music player not available for addToQueue");
        }
        return player;
    });
};

const setQueue = (tracks, startIndex = 0) => {
    console.log(
        "🎼 Store: setQueue called with",
        tracks?.length || 0,
        "tracks",
    );
    musicPlayerStore.update((player) => {
        if (player && player.setQueue) {
            player.setQueue(tracks, startIndex);
        } else {
            console.warn("⚠️ Store: Music player not available for setQueue");
        }
        return player;
    });
};

const isPlayerAvailable = () => {
    let available = false;
    const unsubscribe = musicPlayerStore.subscribe((player) => {
        available = player !== null;
    });
    unsubscribe();
    return available;
};

export const musicPlayerActions = {
    playTrack,
    addToQueue,
    setQueue,
    isPlayerAvailable,
};

// Set the music player instance (called from layout)
export const setMusicPlayer = (playerInstance) => {
    console.log("🔗 Store: setMusicPlayer called");
    musicPlayerStore.set(playerInstance);
    console.log("✅ Store: Music player instance set in store");
};

console.log("🔧 MusicPlayer Store: All exports ready");
