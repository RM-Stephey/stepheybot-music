export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["favicon.svg","test-music-player.js"]),
	mimeTypes: {".svg":"image/svg+xml",".js":"text/javascript"},
	_: {
		client: {start:"_app/immutable/entry/start.RCBeBFaD.js",app:"_app/immutable/entry/app.Bp1ozdMQ.js",imports:["_app/immutable/entry/start.RCBeBFaD.js","_app/immutable/chunks/BlGQVF5x.js","_app/immutable/chunks/Y-CuHk7n.js","_app/immutable/chunks/pifoKX6I.js","_app/immutable/entry/app.Bp1ozdMQ.js","_app/immutable/chunks/Y-CuHk7n.js","_app/immutable/chunks/DKhF1pO8.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js')),
			__memo(() => import('./nodes/3.js'))
		],
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/discover",
				pattern: /^\/discover\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
