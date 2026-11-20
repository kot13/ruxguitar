//! Native menu bar integration ([`muda`]). Falls back to in-window controls when
//! installation fails (e.g. Linux CI without GTK menu wiring).

use std::sync::OnceLock;

#[cfg(windows)]
use iced::window::Window;
#[cfg(windows)]
use iced::window::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

static SETTINGS_ID: OnceLock<muda::MenuId> = OnceLock::new();

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Holds the [`Menu`] so item ids stay valid for the process lifetime.
pub struct NativeMenuBar {
    #[expect(dead_code, reason = "menu must stay alive for native item ids")]
    menu: Menu,
}

impl std::fmt::Debug for NativeMenuBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeMenuBar")
    }
}

/// Whether `event` is the Settings item.
pub fn is_settings_event(event: &MenuEvent) -> bool {
    SETTINGS_ID.get().is_some_and(|id| id == event.id())
}

fn about_item() -> PredefinedMenuItem {
    PredefinedMenuItem::about(
        None,
        Some(AboutMetadata {
            name: Some(APP_NAME.to_string()),
            version: Some(APP_VERSION.to_string()),
            ..AboutMetadata::default()
        }),
    )
}

fn settings_item() -> MenuItem {
    #[cfg(target_os = "macos")]
    let accelerator = Some(Accelerator::new(Some(Modifiers::SUPER), Code::Comma));
    #[cfg(not(target_os = "macos"))]
    let accelerator = None;

    MenuItem::new("Settings…", true, accelerator)
}

fn remember_settings_id(item: &MenuItem) {
    let _ = SETTINGS_ID.set(item.id().clone());
}

/// macOS application menu (replaces the default winit app menu).
#[cfg(target_os = "macos")]
pub fn install_app_menu() -> Result<NativeMenuBar, String> {
    let settings = settings_item();
    remember_settings_id(&settings);

    let app_submenu = Submenu::with_items(
        APP_NAME,
        true,
        &[
            &settings,
            &PredefinedMenuItem::separator(),
            &about_item(),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ],
    )
    .map_err(|e| e.to_string())?;

    let menu = Menu::new();
    menu.append(&app_submenu).map_err(|e| e.to_string())?;
    menu.init_for_nsapp();
    Ok(NativeMenuBar { menu })
}

/// Windows menu bar on the main window.
#[cfg(windows)]
pub fn install_window_menu(window: &dyn Window) -> Result<NativeMenuBar, String> {
    let settings = settings_item();
    remember_settings_id(&settings);

    let file_submenu =
        Submenu::with_items("File", true, &[&settings]).map_err(|e| e.to_string())?;

    let menu = Menu::new();
    menu.append(&file_submenu).map_err(|e| e.to_string())?;

    let handle = window
        .window_handle()
        .map_err(|e| format!("window handle: {e}"))?;
    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return Err("expected Win32 window handle".to_string());
    };
    let hwnd = win32.hwnd.get();
    // SAFETY: `hwnd` is the live window owned by iced/winit.
    unsafe {
        menu.init_for_hwnd(hwnd).map_err(|e| e.to_string())?;
    }
    Ok(NativeMenuBar { menu })
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn install_app_menu() -> Result<NativeMenuBar, String> {
    Err("native menu is not wired on this platform".to_string())
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn install_window_menu(_window: &dyn Window) -> Result<NativeMenuBar, String> {
    Err("native menu is not wired on this platform".to_string())
}
