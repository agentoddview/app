import { app, BrowserWindow, Menu, MenuItemConstructorOptions, shell } from 'electron';
import { writeFileSync, readFileSync, mkdirSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';

const DEFAULT_TARGET = 'https://dash.netransit.net';
const STAGING_TARGET = 'https://dash-staging.netransit.net';
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

const configPath = join(app.getPath('userData'), 'netdash-desktop.config.json');

function readConfig(): NetDashConfig {
  if (!existsSync(configPath)) {
    return { targetUrl: DEFAULT_TARGET };
  }

  try {
    const raw = readFileSync(configPath, 'utf8');
    const parsed = JSON.parse(raw) as NetDashConfig;
    const hasTarget = parsed.targetUrl === DEFAULT_TARGET || parsed.targetUrl === STAGING_TARGET;
    return hasTarget ? parsed : { targetUrl: DEFAULT_TARGET };
  } catch {
    return { targetUrl: DEFAULT_TARGET };
  }
}

function writeConfig(config: NetDashConfig): void {
  const dir = dirname(configPath);
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  writeFileSync(configPath, JSON.stringify(config, null, 2), 'utf8');
}

function switchTarget(targetUrl: string, window: BrowserWindow): void {
  const config: NetDashConfig = { targetUrl };
  writeConfig(config);
  void window.loadURL(targetUrl);
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
    title: 'NetDash',
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

function attachMenu(window: BrowserWindow): void {
  const currentTarget = getTargetUrl();
  const menuTemplate: MenuItemConstructorOptions[] = [
    {
      label: 'File',
      submenu: [
        { label: 'Reload', accelerator: 'CmdOrControl+R', click: () => window.reload() },
        { label: 'Back', accelerator: 'Alt+Left', click: () => window.webContents.goBack() },
        { label: 'Forward', accelerator: 'Alt+Right', click: () => window.webContents.goForward() },
        {
          label: 'Target',
          submenu: [
            {
              label: 'Production',
              type: 'radio',
              checked: currentTarget === DEFAULT_TARGET,
              click: () => switchTarget(DEFAULT_TARGET, window),
            },
            {
              label: 'Staging',
              type: 'radio',
              checked: currentTarget === STAGING_TARGET,
              click: () => switchTarget(STAGING_TARGET, window),
            },
          ],
        },
        {
          label: 'Toggle DevTools (Dev Only)',
          accelerator: 'CmdOrControl+Shift+I',
          visible: process.env.NODE_ENV !== 'production',
          click: () => window.webContents.openDevTools()
        },
        { label: 'Clear Cache + Cookies', click: () => clearCacheAndCookies(window) },
        { type: 'separator' },
        { label: 'Quit', role: 'quit' }
      ]
    }
  ];
  Menu.setApplicationMenu(Menu.buildFromTemplate(menuTemplate));
}

async function clearCacheAndCookies(window: BrowserWindow): Promise<void> {
  await window.webContents.session.clearStorageData();
  await window.webContents.session.clearCache();
}

app.whenReady().then(() => {
  mainWindow = createMainWindow();
  attachMenu(mainWindow);

  app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
      app.quit();
    }
  });
});
