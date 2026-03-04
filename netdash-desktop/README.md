# NetDash Desktop

NetDash Desktop wraps the existing web dashboard at `https://dash.netransit.net` in a native desktop shell.

This repo intentionally does not re-implement the web UI. It simply hosts the live site inside an embedded browser window.

## Tech stack

- Preferred: [Tauri v2](https://tauri.app/) + [Vite](https://vitejs.dev/) + TypeScript
- Fallback: Electron + TypeScript (`electron-fallback/`)

## Prerequisites

- Node.js 20+
- Rust toolchain (for Tauri)
- Windows x64 for primary output
- For macOS/Linux: same codebase works, bundle targets in `src-tauri/tauri.conf.json` can be extended

## Repo layout

- `src/` - Vite pages used by the desktop shell and native settings window
- `src-tauri/` - Tauri app crate (Rust), config, and icons
- `electron-fallback/` - Electron fallback implementation
- `icons/` - placeholder assets

## Installation

```bash
npm install
```

## Development

Run the Tauri shell:

```bash
npm run dev
```

This starts the Vite dev server (`npm run dev:web`) and opens the Tauri app loading:

- `https://dash.netransit.net` by default
- allow-listing for NetDash domains only
- custom File menu and shortcuts:
  - Reload (`Ctrl+R`)
  - Back (`Alt+Left`)
  - Forward (`Alt+Right`)
  - Clear Cache + Cookies
  - Settings...
  - Quit
  - Toggle DevTools (`Ctrl+Shift+I`, dev-only)

## Build

```bash
npm run build
```

This runs:

- `npm run build:web`
- `tauri build`

Tauri output is generated under `src-tauri/target/release/bundle/` (Windows by default: `msi`).

## Build fallback (if Tauri setup fails)

Use the Electron fallback app:

```bash
npm run dev:electron
```

```bash
npm run build:electron
```

The fallback keeps the same navigation guard + menu behavior and is intentionally lightweight.

## Window behavior

- Default size: `1400x900`
- Minimum size: `1100x700`
- Window title: `NetDash`
- Last window size/position is stored in local config and restored on startup:
  - `%APPDATA%\\netdash-desktop\\settings.json` on Windows
  - equivalent user config directory on other platforms

## Navigation/security behavior

- Navigation is allowed only for:
  - `dash.netransit.net`
  - `dash-staging.netransit.net`
  - `api.netransit.net`
  - `avatar.netransit.net`
  - `avatars.netransit.net`
  - `cdn.netransit.net`
- Any other domain is blocked in the app and opened in the default browser.

## Cookie/session behavior

- Browser state is kept in the embedded webview profile.
- Use **File -> Clear Cache + Cookies** to reset embedded web data.

## Native settings screen

Use **File -> Settings...** to open the native settings window and choose:

- `https://dash.netransit.net` (default)
- `https://dash-staging.netransit.net`

The selection is stored locally and reused at startup.

Settings are persisted in:

- `%APPDATA%\\netdash-desktop\\settings.json`

## Auto-update scaffold

`src-tauri/tauri.conf.json` contains an updater placeholder under `plugins.updater`.
To wire auto-update locally:

1. Host a JSON update manifest and binaries at a URL you control.
2. Populate `endpoints` and `pubkey` in `src-tauri/tauri.conf.json`.
3. Enable signing for Windows installers when you release signed production builds.

The scaffold is disabled by default to avoid requiring signing for local development.

## Icon assets

Placeholder icons are provided as:

- `icons/icon.ico`
- `icons/icon.png`
- `icons/icon.icns`

Replace these with your production icons before release:

- Keep file names the same for a drop-in replacement.
- Rebuild after replacement.

## Security notes

- URL allow-list is enforced in the Rust webview navigation handler.
- External destinations are sent to the default browser via OS shell.
- The desktop app intentionally does not re-implement the dashboard UI to keep parity with the live web app.
