import { w as writable } from "./index.js";
console.log("🔧 MusicPlayer Store: File loading...");
console.log("🔧 MusicPlayer Store: Environment:", {
  typeof_window: typeof window,
  typeof_document: typeof document,
  location: typeof window !== "undefined" ? window.location.href : "no window"
});
const musicPlayerStore = writable(null);
musicPlayerStore.subscribe((player) => {
  console.log("🔧 MusicPlayer Store: State changed:", {
    hasPlayer: !!player,
    playerMethods: player ? Object.keys(player).filter(
      (key) => typeof player[key] === "function"
    ) : [],
    timestamp: (/* @__PURE__ */ new Date()).toISOString()
  });
});
console.log("🔧 MusicPlayer Store: musicPlayerStore created");
const playTrack = (track) => {
  console.log("🎵 Store: playTrack called with:", track);
  console.log("🎵 Store: Track details:", {
    title: track?.title,
    artist: track?.artist,
    stream_url: track?.stream_url,
    hasAllRequired: !!(track?.title && track?.artist && track?.stream_url)
  });
  if (!track) {
    console.error("❌ Store: playTrack called with null/undefined track");
    return;
  }
  musicPlayerStore.update((player) => {
    console.log("🎵 Store: Current player state:", {
      hasPlayer: !!player,
      hasPlayMethod: !!player?.playTrackFromParent,
      playerType: typeof player,
      playerMethods: player ? Object.keys(player).filter(
        (key) => typeof player[key] === "function"
      ) : []
    });
    if (player && player.playTrackFromParent) {
      console.log(
        "✅ Store: Music player available, calling playTrackFromParent"
      );
      try {
        player.playTrackFromParent(track);
        console.log(
          "✅ Store: playTrackFromParent completed successfully"
        );
      } catch (error) {
        console.error("❌ Store: Error in playTrackFromParent:", error);
      }
    } else {
      console.warn("⚠️ Store: Music player not available for playTrack");
      console.warn("⚠️ Store: Player details:", {
        player,
        hasPlayMethod: !!player?.playTrackFromParent,
        playerKeys: player ? Object.keys(player) : []
      });
    }
    return player;
  });
};
const addToQueue = (track) => {
  console.log("📝 Store: addToQueue called with:", track);
  console.log("📝 Store: Track details:", {
    title: track?.title,
    artist: track?.artist,
    stream_url: track?.stream_url,
    hasAllRequired: !!(track?.title && track?.artist && track?.stream_url)
  });
  if (!track) {
    console.error("❌ Store: addToQueue called with null/undefined track");
    return;
  }
  musicPlayerStore.update((player) => {
    console.log("📝 Store: Current player state:", {
      hasPlayer: !!player,
      hasAddMethod: !!player?.addTrackToQueue,
      playerType: typeof player,
      playerMethods: player ? Object.keys(player).filter(
        (key) => typeof player[key] === "function"
      ) : []
    });
    if (player && player.addTrackToQueue) {
      console.log(
        "✅ Store: Music player available, calling addTrackToQueue"
      );
      try {
        player.addTrackToQueue(track);
        console.log("✅ Store: addTrackToQueue completed successfully");
      } catch (error) {
        console.error("❌ Store: Error in addTrackToQueue:", error);
      }
    } else {
      console.warn("⚠️ Store: Music player not available for addToQueue");
      console.warn("⚠️ Store: Player details:", {
        player,
        hasAddMethod: !!player?.addTrackToQueue,
        playerKeys: player ? Object.keys(player) : []
      });
    }
    return player;
  });
};
const setQueue = (tracks, startIndex = 0) => {
  console.log(
    "🎼 Store: setQueue called with",
    tracks?.length || 0,
    "tracks, startIndex:",
    startIndex
  );
  console.log("🎼 Store: Tracks details:", {
    isArray: Array.isArray(tracks),
    length: tracks?.length || 0,
    firstTrack: tracks?.[0] ? {
      title: tracks[0].title,
      artist: tracks[0].artist,
      stream_url: tracks[0].stream_url
    } : null
  });
  if (!tracks || !Array.isArray(tracks) || tracks.length === 0) {
    console.error("❌ Store: setQueue called with invalid tracks:", tracks);
    return;
  }
  musicPlayerStore.update((player) => {
    console.log("🎼 Store: Current player state:", {
      hasPlayer: !!player,
      hasSetQueueMethod: !!player?.setQueue,
      playerType: typeof player,
      playerMethods: player ? Object.keys(player).filter(
        (key) => typeof player[key] === "function"
      ) : []
    });
    if (player && player.setQueue) {
      console.log("✅ Store: Music player available, calling setQueue");
      try {
        player.setQueue(tracks, startIndex);
        console.log("✅ Store: setQueue completed successfully");
      } catch (error) {
        console.error("❌ Store: Error in setQueue:", error);
      }
    } else {
      console.warn("⚠️ Store: Music player not available for setQueue");
      console.warn("⚠️ Store: Player details:", {
        player,
        hasSetQueueMethod: !!player?.setQueue,
        playerKeys: player ? Object.keys(player) : []
      });
    }
    return player;
  });
};
const isPlayerAvailable = () => {
  let available = false;
  const unsubscribe = musicPlayerStore.subscribe((player) => {
    available = player !== null;
    console.log("🔍 Store: isPlayerAvailable check:", {
      available,
      hasPlayer: !!player,
      playerType: typeof player
    });
  });
  unsubscribe();
  return available;
};
const musicPlayerActions = {
  playTrack,
  addToQueue,
  setQueue,
  isPlayerAvailable
};
if (typeof window !== "undefined") {
  window.musicPlayerActionsDebug = {
    actions: musicPlayerActions,
    store: musicPlayerStore,
    testActions: () => {
      console.log("🧪 Store: Testing actions availability:", {
        playTrack: typeof musicPlayerActions.playTrack,
        addToQueue: typeof musicPlayerActions.addToQueue,
        setQueue: typeof musicPlayerActions.setQueue,
        isPlayerAvailable: typeof musicPlayerActions.isPlayerAvailable
      });
      try {
        const unsubscribe = musicPlayerStore.subscribe((player) => {
          console.log(
            "🧪 Store: Test subscription successful, player:",
            !!player
          );
        });
        unsubscribe();
      } catch (error) {
        console.error("🧪 Store: Test subscription failed:", error);
      }
    }
  };
  setTimeout(() => {
    window.musicPlayerActionsDebug.testActions();
  }, 1e3);
}
const setMusicPlayer = (playerInstance) => {
  console.log(
    "🔗 Store: setMusicPlayer called with player instance:",
    playerInstance
  );
  console.log("🔗 Store: Player instance details:", {
    hasInstance: !!playerInstance,
    instanceType: typeof playerInstance,
    availableMethods: playerInstance ? Object.keys(playerInstance).filter(
      (key) => typeof playerInstance[key] === "function"
    ) : [],
    hasRequiredMethods: !!(playerInstance?.playTrackFromParent && playerInstance?.addTrackToQueue && playerInstance?.setQueue)
  });
  if (!playerInstance) {
    console.warn(
      "⚠️ Store: setMusicPlayer called with null/undefined instance"
    );
  }
  musicPlayerStore.set(playerInstance);
  console.log("✅ Store: Music player instance set in store");
  const unsubscribe = musicPlayerStore.subscribe((player) => {
    console.log("🔗 Store: Store updated, new state:", {
      hasPlayer: !!player,
      isInstanceSet: player === playerInstance
    });
  });
  unsubscribe();
};
console.log(
  "🔧 MusicPlayer Store: musicPlayerActions created with methods:",
  Object.keys(musicPlayerActions)
);
console.log("🔧 MusicPlayer Store: All exports ready");
console.log("🔧 MusicPlayer Store: Export validation:", {
  musicPlayerActions: !!musicPlayerActions,
  musicPlayerStore: !!musicPlayerStore,
  setMusicPlayer: !!setMusicPlayer,
  actionsType: typeof musicPlayerActions,
  methodsAvailable: Object.keys(musicPlayerActions).map((key) => ({
    name: key,
    type: typeof musicPlayerActions[key]
  }))
});
export {
  musicPlayerStore as a,
  musicPlayerActions as m,
  setMusicPlayer as s
};
