import { app, BrowserWindow, globalShortcut, shell } from 'electron';
import { readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const DEFAULT_TARGET = 'https://dash.netransit.net';
const ALLOWED_HOSTS = [
  'dash.netransit.net',
  'dash-staging.netransit.net',
  'api.netransit.net',
  'avatar.netransit.net',
  'avatars.netransit.net',
  'cdn.netransit.net'
];

type NetDashConfig = {
  targetUrl: string;
};

const configPath = join(app.getPath('userData'), 'net-control-center.config.json');

function readConfig(): NetDashConfig {
  if (!existsSync(configPath)) {
    return { targetUrl: DEFAULT_TARGET };
  }

  try {
    const raw = readFileSync(configPath, 'utf8');
    const parsed = JSON.parse(raw) as NetDashConfig;
    const hasTarget = parsed.targetUrl === DEFAULT_TARGET;
    return hasTarget ? parsed : { targetUrl: DEFAULT_TARGET };
  } catch {
    return { targetUrl: DEFAULT_TARGET };
  }
}

function isAllowedUrl(raw: string): boolean {
  try {
    const parsed = new URL(raw);
    if (!['http:', 'https:'].includes(parsed.protocol)) {
      return false;
    }
    return ALLOWED_HOSTS.some((host) => parsed.host === host || parsed.host.endsWith(`.${host}`));
  } catch {
    return false;
  }
}

let mainWindow: BrowserWindow | null = null;

function getTargetUrl(): string {
  return readConfig().targetUrl;
}

function openExternal(url: string): void {
  if (url) {
    void shell.openExternal(url);
  }
}

function createMainWindow(): BrowserWindow {
  const browser = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 1100,
    minHeight: 700,
    title: 'NET Control Center',
    autoHideMenuBar: true,
    menuBarVisible: false,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
    }
  });

  void browser.loadURL(getTargetUrl());

  browser.webContents.on('will-navigate', (event, url) => {
    if (!isAllowedUrl(url)) {
      event.preventDefault();
      openExternal(url);
    }
  });

  browser.webContents.setWindowOpenHandler(({ url }) => {
    if (isAllowedUrl(url)) {
      if (mainWindow) {
        mainWindow.loadURL(url);
      } else {
        openExternal(url);
      }
    } else {
      openExternal(url);
    }
    return { action: 'deny' };
  });

  return browser;
}

function registerShortcuts(window: BrowserWindow): void {
  globalShortcut.register('CmdOrControl+R', () => {
    window.reload();
  });

  globalShortcut.register('Alt+Left', () => {
    if (window.webContents.canGoBack()) {
      window.webContents.goBack();
    }
  });

  globalShortcut.register('Alt+Right', () => {
    if (window.webContents.canGoForward()) {
      window.webContents.goForward();
    }
  });

  if (process.env.NODE_ENV !== 'production') {
    globalShortcut.register('CmdOrControl+Shift+I', () => {
      window.webContents.openDevTools();
    });
  }

  globalShortcut.register('CmdOrControl+Q', () => {
    app.quit();
  });
}

app.whenReady().then(() => {
  mainWindow = createMainWindow();
  registerShortcuts(mainWindow);

  app.on('will-quit', () => {
    globalShortcut.unregisterAll();
  });

  app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
      app.quit();
    }
  });
});
