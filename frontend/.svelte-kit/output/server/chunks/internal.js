import { c as create_ssr_component, s as setContext, v as validate_component, m as missing_component } from "./ssr.js";
import { a as afterUpdate } from "./ssr2.js";
let base = "";
let assets = base;
const app_dir = "_app";
const initial = { base, assets };
function override(paths) {
  base = paths.base;
  assets = paths.assets;
}
function reset() {
  base = initial.base;
  assets = initial.assets;
}
function set_assets(path) {
  assets = initial.assets = path;
}
let public_env = {};
let safe_public_env = {};
function set_private_env(environment) {
}
function set_public_env(environment) {
  public_env = environment;
}
function set_safe_public_env(environment) {
  safe_public_env = environment;
}
let read_implementation = null;
function set_read_implementation(fn) {
  read_implementation = fn;
}
function set_manifest(_) {
}
let prerendering = false;
function set_building() {
}
function set_prerendering() {
  prerendering = true;
}
const Root = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { stores } = $$props;
  let { page } = $$props;
  let { constructors } = $$props;
  let { components = [] } = $$props;
  let { form } = $$props;
  let { data_0 = null } = $$props;
  let { data_1 = null } = $$props;
  {
    setContext("__svelte__", stores);
  }
  afterUpdate(stores.page.notify);
  if ($$props.stores === void 0 && $$bindings.stores && stores !== void 0) $$bindings.stores(stores);
  if ($$props.page === void 0 && $$bindings.page && page !== void 0) $$bindings.page(page);
  if ($$props.constructors === void 0 && $$bindings.constructors && constructors !== void 0) $$bindings.constructors(constructors);
  if ($$props.components === void 0 && $$bindings.components && components !== void 0) $$bindings.components(components);
  if ($$props.form === void 0 && $$bindings.form && form !== void 0) $$bindings.form(form);
  if ($$props.data_0 === void 0 && $$bindings.data_0 && data_0 !== void 0) $$bindings.data_0(data_0);
  if ($$props.data_1 === void 0 && $$bindings.data_1 && data_1 !== void 0) $$bindings.data_1(data_1);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    {
      stores.page.set(page);
    }
    $$rendered = `  ${constructors[1] ? `${validate_component(constructors[0] || missing_component, "svelte:component").$$render(
      $$result,
      { data: data_0, this: components[0] },
      {
        this: ($$value) => {
          components[0] = $$value;
          $$settled = false;
        }
      },
      {
        default: () => {
          return `${validate_component(constructors[1] || missing_component, "svelte:component").$$render(
            $$result,
            { data: data_1, form, this: components[1] },
            {
              this: ($$value) => {
                components[1] = $$value;
                $$settled = false;
              }
            },
            {}
          )}`;
        }
      }
    )}` : `${validate_component(constructors[0] || missing_component, "svelte:component").$$render(
      $$result,
      { data: data_0, form, this: components[0] },
      {
        this: ($$value) => {
          components[0] = $$value;
          $$settled = false;
        }
      },
      {}
    )}`} ${``}`;
  } while (!$$settled);
  return $$rendered;
});
const options = {
  app_template_contains_nonce: false,
  csp: { "mode": "auto", "directives": { "upgrade-insecure-requests": false, "block-all-mixed-content": false }, "reportOnly": { "upgrade-insecure-requests": false, "block-all-mixed-content": false } },
  csrf_check_origin: true,
  embedded: false,
  env_public_prefix: "PUBLIC_",
  env_private_prefix: "",
  hash_routing: false,
  hooks: null,
  // added lazily, via `get_hooks`
  preload_strategy: "modulepreload",
  root: Root,
  service_worker: false,
  templates: {
    app: ({ head, body, assets: assets2, nonce, env }) => '<!doctype html>\n<html lang="en" data-theme="neon">\n    <head>\n        <meta charset="utf-8" />\n        <link rel="icon" href="' + assets2 + '/favicon.svg" />\n        <meta name="viewport" content="width=device-width, initial-scale=1" />\n        <meta\n            name="description"\n            content="StepheyBot Music - AI-powered music recommendations with a neon-themed interface"\n        />\n        <meta name="author" content="Stephey" />\n        <meta name="theme-color" content="#00FFFF" />\n\n        <!-- Open Graph / Facebook -->\n        <meta property="og:type" content="website" />\n        <meta property="og:title" content="StepheyBot Music" />\n        <meta\n            property="og:description"\n            content="AI-powered music recommendations with a stunning neon interface"\n        />\n        <meta property="og:image" content="' + assets2 + '/og-image.png" />\n\n        <!-- Twitter -->\n        <meta property="twitter:card" content="summary_large_image" />\n        <meta property="twitter:title" content="StepheyBot Music" />\n        <meta\n            property="twitter:description"\n            content="AI-powered music recommendations with a stunning neon interface"\n        />\n        <meta\n            property="twitter:image"\n            content="' + assets2 + '/twitter-image.png"\n        />\n\n        <!-- Fonts -->\n        <link rel="preconnect" href="https://fonts.googleapis.com" />\n        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />\n        <link\n            href="https://fonts.googleapis.com/css2?family=Orbitron:wght@400;500;600;700;800;900&family=Rajdhani:wght@300;400;500;600;700&display=swap"\n            rel="stylesheet"\n        />\n\n        <!-- CSS Custom Properties for Neon Theme -->\n        <style>\n            :root {\n                /* Neon Colors */\n                --neon-cyan: #00ffff;\n                --neon-pink: #ff00ff;\n                --neon-purple: #8000ff;\n                --neon-blue: #0080ff;\n                --neon-green: #00ff80;\n                --neon-orange: #ff8000;\n\n                /* Color Variations */\n                --neon-cyan-bright: #40ffff;\n                --neon-cyan-dim: #008080;\n                --neon-pink-bright: #ff40ff;\n                --neon-pink-dim: #800080;\n                --neon-purple-bright: #a040ff;\n                --neon-purple-dim: #400080;\n\n                /* Background Colors */\n                --bg-primary: #0a0a0f;\n                --bg-secondary: #1a1a2e;\n                --bg-tertiary: #16213e;\n                --bg-card: rgba(26, 26, 46, 0.8);\n                --bg-card-hover: rgba(26, 26, 46, 0.95);\n\n                /* Text Colors */\n                --text-primary: #ffffff;\n                --text-secondary: #b8b8cc;\n                --text-muted: #8888aa;\n                --text-glow: var(--neon-cyan);\n\n                /* Border and Shadow */\n                --border-neon: 1px solid var(--neon-cyan);\n                --border-radius: 8px;\n                --border-radius-lg: 16px;\n                --shadow-neon: 0 0 20px var(--neon-cyan);\n                --shadow-neon-strong: 0 0 40px var(--neon-cyan);\n                --shadow-pink: 0 0 20px var(--neon-pink);\n                --shadow-purple: 0 0 20px var(--neon-purple);\n\n                /* Fonts */\n                --font-primary: "Orbitron", "Courier New", monospace;\n                --font-secondary: "Rajdhani", "Arial", sans-serif;\n\n                /* Spacing */\n                --spacing-xs: 0.25rem;\n                --spacing-sm: 0.5rem;\n                --spacing-md: 1rem;\n                --spacing-lg: 2rem;\n                --spacing-xl: 3rem;\n\n                /* Transitions */\n                --transition-fast: 0.15s ease-in-out;\n                --transition-normal: 0.3s ease-in-out;\n                --transition-slow: 0.5s ease-in-out;\n            }\n\n            * {\n                box-sizing: border-box;\n            }\n\n            html {\n                height: 100%;\n                scroll-behavior: smooth;\n            }\n\n            body {\n                margin: 0;\n                padding: 0;\n                min-height: 100vh;\n                font-family: var(--font-secondary);\n                background: linear-gradient(\n                    135deg,\n                    var(--bg-primary) 0%,\n                    var(--bg-secondary) 50%,\n                    var(--bg-tertiary) 100%\n                );\n                background-attachment: fixed;\n                color: var(--text-primary);\n                overflow-x: hidden;\n                line-height: 1.6;\n            }\n\n            /* Neon glow animation */\n            @keyframes neon-pulse {\n                0%,\n                100% {\n                    text-shadow:\n                        0 0 5px var(--neon-cyan),\n                        0 0 10px var(--neon-cyan),\n                        0 0 15px var(--neon-cyan),\n                        0 0 20px var(--neon-cyan);\n                }\n                50% {\n                    text-shadow:\n                        0 0 2px var(--neon-cyan),\n                        0 0 5px var(--neon-cyan),\n                        0 0 8px var(--neon-cyan),\n                        0 0 12px var(--neon-cyan);\n                }\n            }\n\n            /* Neon border animation */\n            @keyframes neon-border {\n                0%,\n                100% {\n                    box-shadow:\n                        0 0 5px var(--neon-cyan),\n                        0 0 10px var(--neon-cyan),\n                        0 0 15px var(--neon-cyan),\n                        inset 0 0 15px rgba(0, 255, 255, 0.1);\n                }\n                50% {\n                    box-shadow:\n                        0 0 2px var(--neon-cyan),\n                        0 0 5px var(--neon-cyan),\n                        0 0 8px var(--neon-cyan),\n                        inset 0 0 8px rgba(0, 255, 255, 0.05);\n                }\n            }\n\n            /* Background animation */\n            @keyframes bg-shift {\n                0% {\n                    background-position: 0% 50%;\n                }\n                50% {\n                    background-position: 100% 50%;\n                }\n                100% {\n                    background-position: 0% 50%;\n                }\n            }\n\n            /* Custom scrollbar */\n            ::-webkit-scrollbar {\n                width: 8px;\n            }\n\n            ::-webkit-scrollbar-track {\n                background: var(--bg-secondary);\n            }\n\n            ::-webkit-scrollbar-thumb {\n                background: linear-gradient(\n                    var(--neon-cyan),\n                    var(--neon-purple)\n                );\n                border-radius: 4px;\n            }\n\n            ::-webkit-scrollbar-thumb:hover {\n                background: linear-gradient(\n                    var(--neon-cyan-bright),\n                    var(--neon-purple-bright)\n                );\n            }\n\n            /* Selection styling */\n            ::selection {\n                background: var(--neon-cyan);\n                color: var(--bg-primary);\n            }\n\n            /* Focus styles */\n            *:focus {\n                outline: 2px solid var(--neon-cyan);\n                outline-offset: 2px;\n            }\n\n            /* Loading indicator */\n            .loading {\n                display: inline-block;\n                width: 20px;\n                height: 20px;\n                border: 3px solid var(--neon-cyan-dim);\n                border-radius: 50%;\n                border-top-color: var(--neon-cyan);\n                animation: spin 1s ease-in-out infinite;\n            }\n\n            @keyframes spin {\n                to {\n                    transform: rotate(360deg);\n                }\n            }\n        </style>\n\n        ' + head + '\n    </head>\n    <body data-sveltekit-preload-data="hover">\n        <div style="display: contents">' + body + "</div>\n    </body>\n</html>\n",
    error: ({ status, message }) => '<!doctype html>\n<html lang="en">\n	<head>\n		<meta charset="utf-8" />\n		<title>' + message + `</title>

		<style>
			body {
				--bg: white;
				--fg: #222;
				--divider: #ccc;
				background: var(--bg);
				color: var(--fg);
				font-family:
					system-ui,
					-apple-system,
					BlinkMacSystemFont,
					'Segoe UI',
					Roboto,
					Oxygen,
					Ubuntu,
					Cantarell,
					'Open Sans',
					'Helvetica Neue',
					sans-serif;
				display: flex;
				align-items: center;
				justify-content: center;
				height: 100vh;
				margin: 0;
			}

			.error {
				display: flex;
				align-items: center;
				max-width: 32rem;
				margin: 0 1rem;
			}

			.status {
				font-weight: 200;
				font-size: 3rem;
				line-height: 1;
				position: relative;
				top: -0.05rem;
			}

			.message {
				border-left: 1px solid var(--divider);
				padding: 0 0 0 1rem;
				margin: 0 0 0 1rem;
				min-height: 2.5rem;
				display: flex;
				align-items: center;
			}

			.message h1 {
				font-weight: 400;
				font-size: 1em;
				margin: 0;
			}

			@media (prefers-color-scheme: dark) {
				body {
					--bg: #222;
					--fg: #ddd;
					--divider: #666;
				}
			}
		</style>
	</head>
	<body>
		<div class="error">
			<span class="status">` + status + '</span>\n			<div class="message">\n				<h1>' + message + "</h1>\n			</div>\n		</div>\n	</body>\n</html>\n"
  },
  version_hash: "g11fsj"
};
async function get_hooks() {
  let handle;
  let handleFetch;
  let handleError;
  let init;
  let reroute;
  let transport;
  return {
    handle,
    handleFetch,
    handleError,
    init,
    reroute,
    transport
  };
}
export {
  assets as a,
  base as b,
  app_dir as c,
  read_implementation as d,
  options as e,
  set_private_env as f,
  get_hooks as g,
  prerendering as h,
  set_public_env as i,
  set_safe_public_env as j,
  set_read_implementation as k,
  set_assets as l,
  set_building as m,
  set_manifest as n,
  override as o,
  public_env as p,
  set_prerendering as q,
  reset as r,
  safe_public_env as s
};
