## Electron packaged fallback artifacts

If you are using the Electron fallback (because Tauri is unavailable), run:

```bash
npm run build:electron
```

This runs `npm install` in `electron-fallback`, compiles TS, and then runs `electron-builder` for Windows x64 `nsis` + `portable` targets.

Output files:

- Installer: `electron-fallback/dist/out/NET-Control-Center-Setup-<version>.exe`
- Portable: `electron-fallback/dist/out/NET-Control-Center-Portable-<version>.exe`

To build only one target:

```bash
cd electron-fallback
npm run build:win   # installer (NSIS)
npm run dist        # alias (installer + portable)
npm run pack        # unpacked directory (diagnostics only)
```

Replace the icons used by both Tauri and Electron by updating:

- `icons/icon.png` (1024x1024 PNG)
- `icons/icon.ico` (Windows icon: 16/24/32/48/64/128/256)

Then run:

```bash
npm run build:electron
```
