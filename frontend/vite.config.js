import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
    plugins: [sveltekit()],
    server: {
        host: "0.0.0.0",
        port: 5173,
        proxy: {
            "/api": {
                target: "http://localhost:8083",
                changeOrigin: true,
                secure: false,
            },
            "/health": {
                target: "http://localhost:8083",
                changeOrigin: true,
                secure: false,
            },
            "/admin": {
                target: "http://localhost:8083",
                changeOrigin: true,
                secure: false,
            },
        },
    },
    preview: {
        host: "0.0.0.0",
        port: 4173,
    },
    build: {
        rollupOptions: {
            external: [],
        },
        assetsInlineLimit: 0,
    },
    optimizeDeps: {
        include: ["svelte/store"],
    },
});
