import { appWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

interface SettingsState {
  target_url: string;
  launch_at_startup: boolean;
}

const productionUrl = 'https://dash.netransit.net';
const stagingUrl = 'https://dash-staging.netransit.net';
const launchCheckbox = document.querySelector<HTMLInputElement>('#launch-on-startup');
const form = document.querySelector<HTMLFormElement>('#settings-form');
const statusText = document.querySelector<HTMLDivElement>('#status');
const closeButton = document.querySelector<HTMLButtonElement>('#close');

const updateStatus = (message: string): void => {
  if (statusText) {
    statusText.textContent = message;
  }
};

const setSelectedTarget = (value: string): void => {
  const radio = document.querySelector<HTMLInputElement>(
    `input[name="target-url"][value="${value}"]`
  );
  if (radio) {
    radio.checked = true;
  }
};

const readCurrentSelection = (): string => {
  const selected = document.querySelector<HTMLInputElement>('input[name="target-url"]:checked');
  if (selected) {
    return selected.value;
  }
  return productionUrl;
};

const init = async (): Promise<void> => {
  if (!form) return;

  const settings = await invoke<SettingsState>('get_settings');
  setSelectedTarget(settings.target_url === stagingUrl ? stagingUrl : productionUrl);
  if (launchCheckbox) {
    launchCheckbox.checked = settings.launch_at_startup;
  }
  updateStatus('Loaded saved target URL.');

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    const target_url = readCurrentSelection();
    const launch_at_startup = launchCheckbox?.checked ?? false;
    updateStatus('Saving target URL...');
    try {
      await invoke('set_target_url', { target_url });
      await invoke('set_launch_at_startup', { launch_at_startup });
      updateStatus('Saved. Main window and startup preferences were updated.');
      window.setTimeout(() => appWindow.close(), 350);
    } catch {
      updateStatus('Could not save target URL.');
    }
  });

  if (closeButton) {
    closeButton.addEventListener('click', () => {
      appWindow.close();
    });
  }
};

void init();
