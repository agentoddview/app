## Electron packaged fallback artifacts

If you are using the Electron fallback (because Tauri is unavailable), run:

```bash
npm run build:electron
```

This runs `npm install` in `electron-fallback`, compiles TS, and then builds both Windows x64 targets (`nsis` + `portable`) by default.

Output files:

- Installer: `electron-fallback/dist/out/NET-Control-Center-Setup-<version>.exe`
- Portable: `electron-fallback/dist/out/NET-Control-Center-Portable-<version>.exe`

To build only one target:

```bash
cd electron-fallback
npm run dist:portable   # only portable executable
npm run dist:installer  # only NSIS installer
npm run dist:both       # both portable and installer
npm run build:win       # both (alias)
npm run pack        # unpacked directory (diagnostics only)
```

From the repo root you can also run:

```bash
npm run build:electron:portable
npm run build:electron:installer
npm run build:electron:win
```

Why artifacts can still be large:

- Electron includes a full Chromium runtime, so each artifact is larger than a web app.
- The biggest bloat happens when you build installer + portable together because each has its own runtime copy.
- Build one target when possible to keep artifacts smaller.

If installer build fails with `failed creating mmap of ... .nsis.7z`:

1. Remove old outputs so stale `*.nsis.7z` files are not re-packaged:

```powershell
Remove-Item -Recurse -Force "C:\Users\Fritz\netgas\app\netdash-desktop\electron-fallback\dist\out"
```

2. Rebuild installer:

```powershell
Set-Location "C:\Users\Fritz\netgas\app\netdash-desktop"
npm run build:electron:installer
```

Replace the icons used by both Tauri and Electron by updating:

- `icons/icon.png` (1024x1024 PNG)
- `icons/icon.ico` (Windows icon: 16/24/32/48/64/128/256)

Then run:

```bash
npm run build:electron
```
