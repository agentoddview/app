use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{
  menu::{IsMenuItem, Menu, MenuItem, Submenu},
  webview::WebviewUrl,
  AppHandle, Manager, Url, WebviewWindow, WebviewWindowBuilder,
};

const APP_NAME: &str = "NetDash";
const MAIN_WINDOW_LABEL: &str = "main";
const SETTINGS_WINDOW_LABEL: &str = "settings";
const SETTINGS_FILE: &str = "settings.json";

const DEFAULT_URL: &str = "https://dash.netransit.net";

const MENU_RELOAD: &str = "file.reload";
const MENU_BACK: &str = "file.back";
const MENU_FORWARD: &str = "file.forward";
const MENU_TOGGLE_DEVTOOLS: &str = "file.toggle-devtools";
const MENU_SETTINGS: &str = "file.settings";
const MENU_CLEAR_CACHE: &str = "file.clear-cache-cookies";
const MENU_QUIT: &str = "file.quit";

const MIN_WIDTH: f64 = 1100.0;
const MIN_HEIGHT: f64 = 700.0;
const START_WIDTH: f64 = 1400.0;
const START_HEIGHT: f64 = 900.0;

const ALLOWED_HOSTS: &[&str] = &[
  "dash.netransit.net",
  "dash-staging.netransit.net",
  "api.netransit.net",
  "avatar.netransit.net",
  "avatars.netransit.net",
  "cdn.netransit.net",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
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
struct Settings {
  target_url: String,
  window: WindowGeometry,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      target_url: DEFAULT_URL.to_string(),
      window: WindowGeometry::default(),
    }
  }
}

fn settings_path() -> PathBuf {
  let mut base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
  base.push("netdash-desktop");
  base.push(SETTINGS_FILE);
  base
}

fn read_settings() -> Settings {
  let path = settings_path();
  let mut file = match File::open(&path) {
    Ok(file) => file,
    Err(error) => {
      if error.kind() != std::io::ErrorKind::NotFound {
        eprintln!("Failed to open settings file {}: {}", path.display(), error);
      }
      return Settings::default();
    }
  };

  let mut data = String::new();
  if let Err(error) = file.read_to_string(&mut data) {
    eprintln!("Failed to read settings file {}: {}", path.display(), error);
    return Settings::default();
  }

  serde_json::from_str::<Settings>(&data).unwrap_or_else(|error| {
    eprintln!("Invalid settings file {}, resetting to default: {}", path.display(), error);
    Settings::default()
  })
}

fn persist_settings(settings: &Settings) {
  let path = settings_path();
  if let Some(parent) = path.parent() {
    let _ = fs::create_dir_all(parent);
  }

  match serde_json::to_vec_pretty(settings) {
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

fn is_allowed_host(host: &str) -> bool {
  ALLOWED_HOSTS.iter().any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
}

fn is_allowed_navigation(url: &Url) -> bool {
  if !matches!(url.scheme(), "https") {
    return false;
  }
  let Some(host) = url.host_str() else {
    return false;
  };
  is_allowed_host(host)
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
  .title("NetDash Settings")
  .inner_size(500.0, 270.0)
  .resizable(false)
  .minimizable(false)
  .maximizable(false)
  .build()
  {
    eprintln!("Failed to open settings window: {}", error);
  }
}

#[tauri::command]
fn get_settings() -> Settings {
  let mut settings = read_settings();
  if settings.target_url.is_empty() || !is_allowed_navigation(&Url::parse(&settings.target_url).unwrap_or_else(|_| {
    Url::parse(DEFAULT_URL).expect("hardcoded URL must parse")
  })) {
    settings.target_url = DEFAULT_URL.to_string();
  }
  settings
}

#[tauri::command]
fn set_target_url(app: AppHandle, target_url: String) -> Result<Settings, String> {
  let url = Url::parse(&target_url).map_err(|error| error.to_string())?;
  if !is_allowed_navigation(&url) {
    return Err("Target URL is not in NetDash allow-list".to_string());
  }
  let mut settings = read_settings();
  settings.target_url = target_url.clone();
  persist_settings(&settings);

  let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
    return Ok(settings);
  };
  if let Err(error) = main_window.navigate(url) {
    return Err(error.to_string());
  }
  Ok(settings)
}

#[tauri::command]
fn clear_cache_and_cookies(app: AppHandle) -> Result<String, String> {
  let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
    return Ok("main window not available".to_string());
  };
  main_window
    .clear_all_browsing_data()
    .map_err(|error| error.to_string())?;
  Ok("Session data was cleared".to_string())
}

fn allow_list_menu(app: &AppHandle) -> tauri::Result<Menu> {
  let reload = MenuItem::with_id(
    app,
    MENU_RELOAD,
    "Reload",
    true,
    Some("CmdOrControl+R"),
  )?;
  let back = MenuItem::with_id(
    app,
    MENU_BACK,
    "Back",
    true,
    Some("Alt+Left"),
  )?;
  let forward = MenuItem::with_id(
    app,
    MENU_FORWARD,
    "Forward",
    true,
    Some("Alt+Right"),
  )?;
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

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![get_settings, set_target_url, clear_cache_and_cookies])
    .menu(allow_list_menu)
    .setup(|app| {
      let mut settings = read_settings();
      settings.target_url = if is_allowed_navigation(&Url::parse(&settings.target_url).unwrap_or_else(|_| {
        Url::parse(DEFAULT_URL).expect("hardcoded URL must parse")
      })) {
        settings.target_url
      } else {
        DEFAULT_URL.to_string()
      };
      settings.window = normalize_geometry(&settings.window);
      persist_settings(&settings);

      let url = Url::parse(&settings.target_url).expect("validated target URL");
      let mut main_window = WebviewWindowBuilder::new(
        app,
        MAIN_WINDOW_LABEL,
        WebviewUrl::External(url.clone()),
      )
      .title(APP_NAME)
      .inner_size(settings.window.width, settings.window.height)
      .min_inner_size(MIN_WIDTH, MIN_HEIGHT)
      .resizable(true);
      if let (Some(x), Some(y)) = (settings.window.x, settings.window.y) {
        main_window = main_window.position(x, y);
      }
      let main_window = main_window
      .on_navigation(move |navigation_target| {
        if is_allowed_navigation(&navigation_target) {
          true
        } else {
          let _ = open::that(navigation_target.as_str());
          false
        }
      })
      .build()?;
      let app_handle = app.handle().clone();
      let main_window_for_state = main_window.clone();
      main_window.on_window_event(move |_event| {
        remember_window_geometry(&main_window_for_state);
      });
      main_window.on_menu_event(move |_, event| {
        match event.id().as_ref() {
          MENU_RELOAD => {
            let _ = main_window.eval("window.location.reload();");
          }
          MENU_BACK => {
            let _ = main_window.eval("window.history.back();");
          }
          MENU_FORWARD => {
            let _ = main_window.eval("window.history.forward();");
          }
          MENU_SETTINGS => {
            open_settings_window(&app_handle);
          }
          MENU_TOGGLE_DEVTOOLS => {
            #[cfg(debug_assertions)]
            {
              let _ = main_window.open_devtools();
            }
          }
          MENU_CLEAR_CACHE => {
            let _ = clear_cache_and_cookies(app_handle.clone());
          }
          MENU_QUIT => {
            app_handle.exit(0);
          }
          _ => {}
        }
      });
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
