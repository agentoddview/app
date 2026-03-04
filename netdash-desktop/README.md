# NET Control Center

NET Control Center is a native desktop wrapper around the live dashboard at `https://dash.netransit.net`.
It loads the remote website inside a native WebView without re-implementing dashboard UI.

## Tech stack

- Preferred: [Tauri v2](https://v2.tauri.app/) + [Vite](https://vite.dev/) + TypeScript
- Fallback: Electron + TypeScript (`electron-fallback/`) if Tauri tooling is unavailable

## What changed in this version

- Branded as **NET Control Center**
- Window title: `NET Control Center`
- Default URL: `https://dash.netransit.net`
- Configurable staging URL: `https://dash-staging.netransit.net`
- Window size: `1400x900` default, `1100x700` minimum
- Window size and position are persisted locally
- External link allow-list with external links opened in system browser
- Native settings screen for URL + startup preference
- Tray-first behavior and quality-of-life menu actions

## Prerequisites

- Node.js 20+
- Rust toolchain (for Tauri build)
- Windows x64 for requested installer/portable output

## Repository layout

- `src/` – shell pages (`index.html`, `settings.html`) and TS front-end
- `src-tauri/` – Rust app crate, config, and Tauri metadata
- `icons/` – app icons (`icon.png`, `icon.ico`, optional `icon.icns`)
- `electron-fallback/` – backup shell when Tauri is not available

## Install

```bash
npm install
```

## Run in development

```bash
npm run dev
```

This starts the Vite dev server and launches the desktop shell loading the target URL.

## Build

```bash
npm run build
```

- Runs the Vite build and then `tauri build`.
- Produces the configured bundle artifacts under `src-tauri/target/release/bundle/...`.

Windows explicit build:

```bash
npm run build:win
```

Expected outputs (Windows):

- Portable executable: `src-tauri/target/release/bundle/app/net-control-center.exe`
- Installer:
  - `src-tauri/target/release/bundle/nsis/NET Control Center_x.y.z_x64-setup.exe`
  - `src-tauri/target/release/bundle/msi/NET Control Center_x.y.z_x64.msi`

If only one installer format is generated, this is controlled by your Tauri version and platform support.

## Runtime behavior and features

### Menu

- File ? Open NET Control Center
- File ? Reload
- File ? Back
- File ? Forward
- File ? Settings...
- File ? Clear Cache + Cookies
- File ? Quit
- File ? Toggle DevTools (dev only)

### Keyboard shortcuts

- `Ctrl+R` reload
- `Alt+Left` back
- `Alt+Right` forward
- `Ctrl+Shift+I` toggle DevTools (dev only)
- `Ctrl+Q` quit

### Tray

- App creates a tray icon.
- Closing the app window hides it to tray (it does not quit).
- Tray menu:
  - Open NET Control Center
  - Reload
  - Clear Cache + Cookies
  - Quit
- Tray icon tooltip includes: **"App is running in tray"**

### Settings

Open settings from **File ? Settings...**.

It supports:

- Production URL (`https://dash.netransit.net`) [default]
- Staging URL (`https://dash-staging.netransit.net`)
- "Launch on startup" toggle (stored locally, scaffolded)

Settings are saved at:

- `%APPDATA%\net-control-center\settings.json` (Windows)

## Session/cookies

Session storage is preserved automatically through the embedded webview profile.

Use **Clear Cache + Cookies** from menu or tray to clear browser storage; it then reloads the shell to the configured URL.

## Security and allow-list

Navigation in the webview is restricted to allow-listed hosts. External destinations are opened in your default browser and blocked from in-app navigation.

Default allow-list:

- `dash.netransit.net`
- `dash-staging.netransit.net`
- `api.netransit.net`
- `avatar.netransit.net`
- `avatars.netransit.net`
- `cdn.netransit.net`

To update allow-list for a proxy/avatar domain, add it to `allowed_hosts` in:

- `%APPDATA%\net-control-center\settings.json`

If this file does not exist, defaults are regenerated on first launch.

## Icons

Expected source files:

- `icons/icon.png` (1024x1024)
- `icons/icon.ico` (multi-size: 16,24,32,48,64,128,256)

They are wired via `src-tauri/tauri.conf.json`:

- `bundle.icon`
- Windows NSIS installer icon via `bundle.windows.nsis.installerIcon`

To replace icons:

1. Replace `icon.png` and `icon.ico` with your assets using the same filenames.
2. Rebuild: `npm run build:win`.
3. If you only have a PNG, regenerate ico with ImageMagick:

   ```bash
   npm run icons:generate
   ```

If ImageMagick is unavailable, install it or use your preferred icon tool and keep the same file names.

## Auto-update

The scaffold in `src-tauri/tauri.conf.json` keeps updater disabled for local dev.
To enable:

1. Add signed/update endpoints to `plugins.updater`
2. Provide manifest and binaries
3. Re-enable updater in config

For local development and testing, updater remains off by design.

## Electron fallback

If Tauri is unavailable:

```bash
npm run dev:electron
npm run build:electron
```

## Troubleshooting

- If Tauri commands fail due to missing Rust tooling (`cargo metadata`), ensure Rust is installed and in PATH.
- If the desktop shell fails to launch, confirm that `npm install` has been run in both repo root and `electron-fallback/` for fallback testing.
