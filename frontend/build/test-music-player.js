// StepheyBot Music Player Testing Script
// This script helps debug and test the music player functionality
// Usage: Load this script in the browser console or include it in a page

(function() {
    'use strict';

    console.log('🎵 StepheyBot Music Player Test Script Loaded');

    // Test configuration
    const TEST_CONFIG = {
        apiBase: '/api/v1',
        testUserId: 'user1',
        maxRetries: 3,
        retryDelay: 1000
    };

    // Utility functions
    const utils = {
        log: (message, data = null) => {
            console.log(`🎵 Test: ${message}`, data || '');
        },

        error: (message, error = null) => {
            console.error(`❌ Test Error: ${message}`, error || '');
        },

        success: (message, data = null) => {
            console.log(`✅ Test Success: ${message}`, data || '');
        },

        wait: (ms) => new Promise(resolve => setTimeout(resolve, ms)),

        formatTrack: (track) => ({
            title: track.title,
            artist: track.artist,
            stream_url: track.stream_url,
            track_id: track.track_id
        })
    };

    // API Testing Functions
    const apiTests = {
        // Test health endpoint
        testHealth: async () => {
            utils.log('Testing health endpoint...');
            try {
                const response = await fetch(`${TEST_CONFIG.apiBase}/../health`);
                const data = await response.json();
                if (data.status === 'healthy') {
                    utils.success('Health check passed', data);
                    return true;
                } else {
                    utils.error('Health check failed', data);
                    return false;
                }
            } catch (error) {
                utils.error('Health check request failed', error);
                return false;
            }
        },

        // Test library stats
        testLibraryStats: async () => {
            utils.log('Testing library stats...');
            try {
                const response = await fetch(`${TEST_CONFIG.apiBase}/library/stats`);
                const data = await response.json();
                if (data.total_tracks > 0) {
                    utils.success(`Library stats retrieved: ${data.total_tracks} tracks, ${data.total_artists} artists`, data);
                    return data;
                } else {
                    utils.error('No tracks found in library', data);
                    return null;
                }
            } catch (error) {
                utils.error('Library stats request failed', error);
                return null;
            }
        },

        // Test recommendations
        testRecommendations: async (limit = 5) => {
            utils.log(`Testing recommendations (limit: ${limit})...`);
            try {
                const response = await fetch(`${TEST_CONFIG.apiBase}/recommendations/${TEST_CONFIG.testUserId}?limit=${limit}`);
                const data = await response.json();
                if (data.recommendations && data.recommendations.length > 0) {
                    utils.success(`Retrieved ${data.recommendations.length} recommendations`,
                        data.recommendations.map(utils.formatTrack));
                    return data.recommendations;
                } else {
                    utils.error('No recommendations found', data);
                    return [];
                }
            } catch (error) {
                utils.error('Recommendations request failed', error);
                return [];
            }
        },

        // Test streaming endpoint
        testStreamUrl: async (trackId) => {
            utils.log(`Testing stream URL for track: ${trackId}`);
            try {
                const response = await fetch(`${TEST_CONFIG.apiBase}/stream/${trackId}`, { method: 'HEAD' });
                if (response.ok) {
                    const contentType = response.headers.get('content-type');
                    const contentLength = response.headers.get('content-length');
                    utils.success(`Stream URL working - Type: ${contentType}, Size: ${contentLength} bytes`);
                    return true;
                } else {
                    utils.error(`Stream URL failed with status: ${response.status}`);
                    return false;
                }
            } catch (error) {
                utils.error('Stream URL test failed', error);
                return false;
            }
        }
    };

    // Music Player Testing Functions
    const playerTests = {
        // Check if music player store is available
        checkStore: () => {
            utils.log('Checking music player store...');
            if (typeof window !== 'undefined' && window.musicPlayerStore) {
                utils.success('Music player store found in window');
                return true;
            } else {
                utils.error('Music player store not found in window');
                return false;
            }
        },

        // Check if music player actions are available
        checkActions: () => {
            utils.log('Checking music player actions...');
            if (typeof window !== 'undefined' && window.musicPlayerActions) {
                utils.success('Music player actions found in window');
                return true;
            } else {
                utils.error('Music player actions not found in window');
                return false;
            }
        },

        // Get current music player instance
        getPlayerInstance: () => {
            return new Promise((resolve) => {
                if (typeof window !== 'undefined' && window.musicPlayerStore) {
                    window.musicPlayerStore.subscribe(player => {
                        resolve(player);
                    })();
                } else {
                    resolve(null);
                }
            });
        },

        // Test playing a track
        testPlayTrack: async (track) => {
            utils.log('Testing play track functionality...', utils.formatTrack(track));

            const player = await playerTests.getPlayerInstance();
            if (!player) {
                utils.error('No music player instance available');
                return false;
            }

            if (!player.playTrackFromParent) {
                utils.error('playTrackFromParent method not available on player');
                return false;
            }

            try {
                utils.log('Calling playTrackFromParent...');
                player.playTrackFromParent(track);
                utils.success('playTrackFromParent called successfully');
                return true;
            } catch (error) {
                utils.error('Failed to call playTrackFromParent', error);
                return false;
            }
        }
    };

    // DOM Testing Functions
    const domTests = {
        // Check if music player element exists
        checkPlayerElement: () => {
            utils.log('Checking for music player DOM element...');
            const playerElement = document.querySelector('.music-player');
            if (playerElement) {
                utils.success('Music player element found in DOM');
                return playerElement;
            } else {
                utils.error('Music player element not found in DOM');
                return null;
            }
        },

        // Check if audio element exists
        checkAudioElement: () => {
            utils.log('Checking for audio element...');
            const audioElement = document.querySelector('audio');
            if (audioElement) {
                utils.success('Audio element found', {
                    src: audioElement.src,
                    readyState: audioElement.readyState,
                    paused: audioElement.paused
                });
                return audioElement;
            } else {
                utils.error('Audio element not found');
                return null;
            }
        },

        // Check for play buttons
        checkPlayButtons: () => {
            utils.log('Checking for play buttons...');
            const playButtons = document.querySelectorAll('.play-btn, .action-btn.play-btn');
            if (playButtons.length > 0) {
                utils.success(`Found ${playButtons.length} play buttons`);
                return playButtons;
            } else {
                utils.error('No play buttons found');
                return [];
            }
        }
    };

    // Comprehensive test suite
    const runFullTest = async () => {
        console.log('🚀 Starting comprehensive music player test...');
        console.log('='.repeat(50));

        const results = {
            health: false,
            libraryStats: null,
            recommendations: [],
            streaming: false,
            playerElement: null,
            audioElement: null,
            playButtons: [],
            playerInstance: null
        };

        // Test API endpoints
        utils.log('Testing API endpoints...');
        results.health = await apiTests.testHealth();
        results.libraryStats = await apiTests.testLibraryStats();
        results.recommendations = await apiTests.testRecommendations(3);

        // Test streaming if we have recommendations
        if (results.recommendations.length > 0) {
            const firstTrack = results.recommendations[0];
            results.streaming = await apiTests.testStreamUrl(firstTrack.track_id);
        }

        // Test DOM elements
        utils.log('Testing DOM elements...');
        results.playerElement = domTests.checkPlayerElement();
        results.audioElement = domTests.checkAudioElement();
        results.playButtons = domTests.checkPlayButtons();

        // Test music player functionality
        utils.log('Testing music player functionality...');
        results.playerInstance = await playerTests.getPlayerInstance();

        if (results.playerInstance) {
            utils.success('Music player instance available');
        } else {
            utils.error('Music player instance not available');
        }

        // Test playing a track if everything is available
        if (results.recommendations.length > 0 && results.playerInstance) {
            await playerTests.testPlayTrack(results.recommendations[0]);
        }

        console.log('='.repeat(50));
        console.log('🏁 Test complete! Results summary:');
        console.table({
            'API Health': results.health ? '✅' : '❌',
            'Library Stats': results.libraryStats ? '✅' : '❌',
            'Recommendations': results.recommendations.length > 0 ? '✅' : '❌',
            'Streaming': results.streaming ? '✅' : '❌',
            'Player Element': results.playerElement ? '✅' : '❌',
            'Audio Element': results.audioElement ? '✅' : '❌',
            'Play Buttons': results.playButtons.length > 0 ? '✅' : '❌',
            'Player Instance': results.playerInstance ? '✅' : '❌'
        });

        return results;
    };

    // Individual test functions for manual testing
    const quickTests = {
        // Quick API test
        api: async () => {
            await apiTests.testHealth();
            await apiTests.testLibraryStats();
            const recs = await apiTests.testRecommendations(1);
            if (recs.length > 0) {
                await apiTests.testStreamUrl(recs[0].track_id);
            }
        },

        // Quick DOM test
        dom: () => {
            domTests.checkPlayerElement();
            domTests.checkAudioElement();
            domTests.checkPlayButtons();
        },

        // Quick player test
        player: async () => {
            const player = await playerTests.getPlayerInstance();
            if (player) {
                utils.success('Player available with methods:', Object.keys(player));
            }
        },

        // Test playing first recommendation
        playFirst: async () => {
            const recs = await apiTests.testRecommendations(1);
            if (recs.length > 0) {
                await playerTests.testPlayTrack(recs[0]);
            }
        }
    };

    // Expose testing functions to global scope
    window.StepheyBotMusicTest = {
        runFullTest,
        quick: quickTests,
        api: apiTests,
        player: playerTests,
        dom: domTests,
        utils
    };

    utils.success('Testing functions loaded! Use StepheyBotMusicTest.runFullTest() to run all tests');
    console.log('Available quick tests:', Object.keys(quickTests));
    console.log('Usage examples:');
    console.log('- StepheyBotMusicTest.runFullTest() - Run complete test suite');
    console.log('- StepheyBotMusicTest.quick.api() - Test API endpoints');
    console.log('- StepheyBotMusicTest.quick.dom() - Test DOM elements');
    console.log('- StepheyBotMusicTest.quick.player() - Test player instance');
    console.log('- StepheyBotMusicTest.quick.playFirst() - Try to play first recommendation');

})();
