use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{
  menu::{IsMenuItem, Menu, MenuItem, Submenu},
  tray::{TrayIconBuilder, TrayIconEvent},
  AppHandle, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

const APP_NAME: &str = "NET Control Center";
const MAIN_WINDOW_LABEL: &str = "main";
const SETTINGS_WINDOW_LABEL: &str = "settings";
const SETTINGS_FILE: &str = "settings.json";

const DEFAULT_URL: &str = "https://dash.netransit.net";

const MENU_OPEN: &str = "app.open";
const MENU_RELOAD: &str = "file.reload";
const MENU_BACK: &str = "file.back";
const MENU_FORWARD: &str = "file.forward";
const MENU_TOGGLE_DEVTOOLS: &str = "file.toggle-devtools";
const MENU_SETTINGS: &str = "file.settings";
const MENU_CLEAR_CACHE: &str = "file.clear-cache-cookies";
const MENU_QUIT: &str = "app.quit";
const MENU_TRAY_HINT: &str = "tray.hint";

const MIN_WIDTH: f64 = 1100.0;
const MIN_HEIGHT: f64 = 700.0;
const START_WIDTH: f64 = 1400.0;
const START_HEIGHT: f64 = 900.0;

const DEFAULT_ALLOWED_HOSTS: &[&str] = &[
  "dash.netransit.net",
  "dash-staging.netransit.net",
  "api.netransit.net",
  "avatar.netransit.net",
  "avatars.netransit.net",
  "cdn.netransit.net",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
struct WindowGeometry {
  x: Option<i32>,
  y: Option<i32>,
  width: f64,
  height: f64,
}

impl Default for WindowGeometry {
  fn default() -> Self {
    Self {
      x: None,
      y: None,
      width: START_WIDTH,
      height: START_HEIGHT,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
struct Settings {
  target_url: String,
  launch_at_startup: bool,
  window: WindowGeometry,
  allowed_hosts: Vec<String>,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      target_url: DEFAULT_URL.to_string(),
      launch_at_startup: false,
      window: WindowGeometry::default(),
      allowed_hosts: DEFAULT_ALLOWED_HOSTS.iter().map(|value| value.to_string()).collect(),
    }
  }
}

fn settings_path() -> PathBuf {
  let mut base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
  base.push("net-control-center");
  base.push(SETTINGS_FILE);
  base
}

fn normalize_allowed_hosts(mut hosts: Vec<String>) -> Vec<String> {
  hosts.retain(|host| !host.trim().is_empty());
  if hosts.is_empty() {
    return DEFAULT_ALLOWED_HOSTS
      .iter()
      .map(|host| host.to_string())
      .collect();
  }
  hosts.sort_unstable();
  hosts.dedup();
  hosts
}

fn normalize_geometry(input: &WindowGeometry) -> WindowGeometry {
  WindowGeometry {
    x: input.x,
    y: input.y,
    width: if input.width >= MIN_WIDTH {
      input.width
    } else {
      START_WIDTH
    },
    height: if input.height >= MIN_HEIGHT {
      input.height
    } else {
      START_HEIGHT
    },
  }
}

fn normalize_settings(mut settings: Settings) -> Settings {
  settings.target_url = if is_allowed_navigation_url(&settings.target_url, &settings.allowed_hosts) {
    settings.target_url
  } else {
    DEFAULT_URL.to_string()
  };
  settings.window = normalize_geometry(&settings.window);
  settings.allowed_hosts = normalize_allowed_hosts(settings.allowed_hosts);
  settings
}

fn is_allowed_host(host: &str, allowed_hosts: &[String]) -> bool {
  allowed_hosts
    .iter()
    .any(|allowed| host == allowed || host.ends_with(&format!(".{allowed}")))
}

fn is_allowed_navigation_url(raw_url: &str, allowed_hosts: &[String]) -> bool {
  match Url::parse(raw_url) {
    Ok(url) => is_allowed_navigation(&url, allowed_hosts),
    Err(_) => false,
  }
}

fn is_allowed_navigation(url: &Url, allowed_hosts: &[String]) -> bool {
  if !matches!(url.scheme(), "https") {
    return false;
  }
  let Some(host) = url.host_str() else {
    return false;
  };
  is_allowed_host(host, allowed_hosts)
}

fn read_settings() -> Settings {
  let path = settings_path();
  let mut file = match File::open(&path) {
    Ok(file) => file,
    Err(error) => {
      if error.kind() != std::io::ErrorKind::NotFound {
        eprintln!("Failed to open settings file {}: {}", path.display(), error);
      }
      return normalize_settings(Settings::default());
    }
  };

  let mut data = String::new();
  if let Err(error) = file.read_to_string(&mut data) {
    eprintln!("Failed to read settings file {}: {}", path.display(), error);
    return normalize_settings(Settings::default());
  }

  serde_json::from_str::<Settings>(&data).unwrap_or_else(|error| {
    eprintln!(
      "Invalid settings file {}, resetting to default: {}",
      path.display(),
      error
    );
    normalize_settings(Settings::default())
  })
}

fn persist_settings(settings: &Settings) {
  let path = settings_path();
  if let Some(parent) = path.parent() {
    let _ = fs::create_dir_all(parent);
  }

  let normalized = normalize_settings(settings.clone());
  match serde_json::to_vec_pretty(&normalized) {
    Ok(bytes) => {
      if let Err(error) = File::create(&path).and_then(|mut file| file.write_all(&bytes)) {
        eprintln!("Failed to write settings file {}: {}", path.display(), error);
      }
    }
    Err(error) => {
      eprintln!("Failed to serialize settings: {}", error);
    }
  }
}

fn remember_window_geometry<R: tauri::Runtime>(window: &WebviewWindow<R>) {
  let mut settings = read_settings();
  let size = window.inner_size();
  let position = window.outer_position();

  if let (Ok(size), Ok(position)) = (size, position) {
    settings.window.width = size.width.max(MIN_WIDTH as u32) as f64;
    settings.window.height = size.height.max(MIN_HEIGHT as u32) as f64;
    settings.window.x = Some(position.x);
    settings.window.y = Some(position.y);
    persist_settings(&settings);
  }
}

fn open_settings_window(app: &AppHandle) {
  if let Some(existing) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
    if let Err(error) = existing.set_focus() {
      eprintln!("Failed to focus settings window: {}", error);
    }
    return;
  }

  if let Err(error) = WebviewWindowBuilder::new(
    app,
    SETTINGS_WINDOW_LABEL,
    WebviewUrl::App("settings.html".into()),
  )
  .title("NET Control Center Settings")
  .inner_size(500.0, 330.0)
  .resizable(false)
  .minimizable(false)
  .maximizable(false)
  .build()
  {
    eprintln!("Failed to open settings window: {}", error);
  }
}

fn execute_menu_action(app: &AppHandle, action: &str) {
  match action {
    MENU_OPEN => {
      let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
      };
      let _ = main_window.show();
      let _ = main_window.set_focus();
    }
    MENU_RELOAD => {
      let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
      };
      let _ = main_window.eval("window.location.reload();");
    }
    MENU_BACK => {
      let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
      };
      let _ = main_window.eval("window.history.back();");
    }
    MENU_FORWARD => {
      let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
      };
      let _ = main_window.eval("window.history.forward();");
    }
    MENU_SETTINGS => {
      open_settings_window(app);
    }
    MENU_CLEAR_CACHE => {
      let _ = clear_cache_and_cookies_internal(app);
    }
    MENU_TRAY_HINT => {}
    MENU_TOGGLE_DEVTOOLS => {
      #[cfg(debug_assertions)]
      {
        let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
          return;
        };
        let _ = main_window.open_devtools();
      }
    }
    MENU_QUIT => {
      app.exit(0);
    }
    _ => {}
  }
}

#[cfg(target_os = "windows")]
fn sync_startup_registration(
  _app: &AppHandle,
  _enabled: bool,
) {
  // Implemented as scaffold for v1.2.
  // For production:
  // - add the `tauri-plugin-autostart` dependency
  // - register/unregister via the plugin at startup change time and on launch.
}

#[cfg(not(target_os = "windows"))]
fn sync_startup_registration(_app: &AppHandle, _enabled: bool) {}

fn clear_cache_and_cookies_internal(app: &AppHandle) -> Result<Settings, String> {
  let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
    return Err("Main window is not available".to_string());
  };
  main_window
    .clear_all_browsing_data()
    .map_err(|error| error.to_string())?;
  let settings = read_settings();
  let url = Url::parse(&settings.target_url).map_err(|error| error.to_string())?;
  main_window
    .navigate(url)
    .map_err(|error| error.to_string())?;
  Ok(settings)
}

fn build_app_menu(app: &AppHandle) -> tauri::Result<Menu> {
  let open = MenuItem::with_id(
    app,
    MENU_OPEN,
    "Open NET Control Center",
    true,
    None::<&str>,
  )?;
  let reload = MenuItem::with_id(app, MENU_RELOAD, "Reload", true, Some("CmdOrControl+R"))?;
  let back = MenuItem::with_id(app, MENU_BACK, "Back", true, Some("Alt+Left"))?;
  let forward = MenuItem::with_id(app, MENU_FORWARD, "Forward", true, Some("Alt+Right"))?;
  let settings = MenuItem::with_id(app, MENU_SETTINGS, "Settings...", true, None::<&str>)?;
  let clear_cache = MenuItem::with_id(
    app,
    MENU_CLEAR_CACHE,
    "Clear Cache + Cookies",
    true,
    None::<&str>,
  )?;
  let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, Some("CmdOrControl+Q"))?;

  let mut items: Vec<&dyn IsMenuItem> = vec![
    &open,
    &reload,
    &back,
    &forward,
    &settings,
    &clear_cache,
  ];

  #[cfg(debug_assertions)]
  {
    let devtools = MenuItem::with_id(
      app,
      MENU_TOGGLE_DEVTOOLS,
      "Toggle DevTools (Dev Only)",
      true,
      Some("CmdOrControl+Shift+I"),
    )?;
    items.push(&devtools);
  }

  items.push(&quit);
  let file = Submenu::with_id_and_items(app, "file", "File", true, &items)?;
  Menu::with_items(app, &[&file])
}

fn build_tray_menu(app: &AppHandle) -> tauri::Result<Menu> {
  let open = MenuItem::with_id(
    app,
    MENU_OPEN,
    "Open NET Control Center",
    true,
    None::<&str>,
  )?;
  let hint = MenuItem::with_id(
    app,
    MENU_TRAY_HINT,
    "App is running in tray",
    false,
    None::<&str>,
  )?;
  let reload = MenuItem::with_id(app, MENU_RELOAD, "Reload", true, Some("CmdOrControl+R"))?;
  let clear_cache = MenuItem::with_id(app, MENU_CLEAR_CACHE, "Clear Cache + Cookies", true, None::<&str>)?;
  let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, Some("CmdOrControl+Q"))?;

  Menu::with_items(app, &[&open, &hint, &reload, &clear_cache, &quit])
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
  let tray_menu = build_tray_menu(app)?;
  let mut tray_builder = TrayIconBuilder::new()
    .menu(&tray_menu)
    .tooltip("App is running in tray");
  if let Some(icon) = app.default_window_icon() {
    tray_builder = tray_builder.icon(icon.clone());
  }
  let _tray = tray_builder
    .on_menu_event(|app, event| {
      execute_menu_action(app, event.id().as_ref());
    })
    .on_tray_icon_event(|tray, _event| {
      let Some(main_window) = tray.app_handle().get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
      };
      let _ = main_window.show();
      let _ = main_window.set_focus();
    })
    .build(app)?;
  let _ = _tray;
  Ok(())
}

#[tauri::command]
fn get_settings() -> Settings {
  read_settings()
}

#[tauri::command]
fn set_target_url(app: AppHandle, target_url: String) -> Result<Settings, String> {
  let mut settings = read_settings();
  if !is_allowed_navigation_url(&target_url, &settings.allowed_hosts) {
    return Err("Target URL is not in NET Control Center allow-list".to_string());
  }

  settings.target_url = target_url;
  persist_settings(&settings);

  if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
    let url = Url::parse(&settings.target_url).map_err(|error| error.to_string())?;
    main_window
      .navigate(url)
      .map_err(|error| error.to_string())?;
  }

  Ok(settings)
}

#[tauri::command]
fn set_launch_at_startup(app: AppHandle, launch_at_startup: bool) -> Result<Settings, String> {
  let mut settings = read_settings();
  settings.launch_at_startup = launch_at_startup;
  persist_settings(&settings);
  sync_startup_registration(&app, launch_at_startup);
  Ok(settings)
}

#[tauri::command]
fn clear_cache_and_cookies(app: AppHandle) -> Result<Settings, String> {
  clear_cache_and_cookies_internal(&app)
}

fn main() {
  tauri::Builder::default()
    .menu(build_app_menu)
    .on_menu_event(|app, event| {
      execute_menu_action(app, event.id().as_ref());
    })
    .invoke_handler(tauri::generate_handler![
      get_settings,
      set_target_url,
      set_launch_at_startup,
      clear_cache_and_cookies,
    ])
    .setup(|app| {
      let settings = normalize_settings(read_settings());
      persist_settings(&settings);

      let allowed_hosts = settings.allowed_hosts.clone();
      let url = Url::parse(&settings.target_url).expect("validated target URL");
      let mut window_builder = WebviewWindowBuilder::new(
        app,
        MAIN_WINDOW_LABEL,
        WebviewUrl::External(url),
      )
      .title(APP_NAME)
      .inner_size(settings.window.width, settings.window.height)
      .min_inner_size(MIN_WIDTH, MIN_HEIGHT)
      .resizable(true);

      if let (Some(x), Some(y)) = (settings.window.x, settings.window.y) {
        window_builder = window_builder.position(x, y);
      }

      let main_window = window_builder
        .on_navigation(move |navigation_target| {
          if is_allowed_navigation(&navigation_target, &allowed_hosts) {
            true
          } else {
            let _ = open::that(navigation_target.as_str());
            false
          }
        })
        .build()?;

      let main_window_for_state = main_window.clone();
      main_window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
          api.prevent_close();
          let _ = main_window_for_state.hide();
          remember_window_geometry(&main_window_for_state);
        }
      });

      setup_tray(app.handle())?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
