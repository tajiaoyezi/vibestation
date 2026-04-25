//! MVP-11 Phase 3 · Native Context Menu + 快捷键
//!
//! 使用 Tauri v2 `tauri::menu` API · macOS 走 AppKit NSMenu · Linux 自动 GTK fallback。
//! 禁止 web div 模拟 context menu（spec §H.4）。

use tauri::menu::{Menu, MenuBuilder, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position, Runtime, Window};

// ─── Tab Context Menu IDs ──────────────────────────────────────────

const TAB_CLOSE: &str = "tab:close";
const TAB_CLOSE_OTHERS: &str = "tab:close_others";
const TAB_CLOSE_RIGHT: &str = "tab:close_right";
const TAB_RENAME: &str = "tab:rename";
const TAB_DUPLICATE: &str = "tab:duplicate";

// ─── Terminal Context Menu IDs ─────────────────────────────────────

const TERM_COPY: &str = "term:copy";
const TERM_PASTE: &str = "term:paste";
const TERM_CLEAR: &str = "term:clear";
const TERM_SELECT_ALL: &str = "term:select_all";

// ─── App Menu / Accelerator IDs ────────────────────────────────────

const APP_NEW_TAB: &str = "app:new_tab";
const APP_CLOSE_TAB: &str = "app:close_tab";
const APP_SPLIT_H: &str = "app:split_h";
const APP_SPLIT_V: &str = "app:split_v";
const APP_PREFS: &str = "app:prefs";

// ─── Event Payload ─────────────────────────────────────────────────

#[derive(Clone, serde::Serialize)]
struct MenuActionPayload {
    action: String,
}

// ─── Tab Context Menu (5 项) ───────────────────────────────────────

pub fn tab_context_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, String> {
    let close =
        MenuItem::with_id(app, TAB_CLOSE, "Close Tab", true, Some("Cmd+W")).map_err(|e| {
            format!("menu: failed to create Close Tab item: {e}")
        })?;
    let close_others =
        MenuItem::with_id(app, TAB_CLOSE_OTHERS, "Close Other Tabs", true, None::<&str>)
            .map_err(|e| format!("menu: failed to create Close Other Tabs item: {e}"))?;
    let close_right = MenuItem::with_id(
        app,
        TAB_CLOSE_RIGHT,
        "Close Tabs to the Right",
        true,
        None::<&str>,
    )
    .map_err(|e| format!("menu: failed to create Close Tabs to the Right item: {e}"))?;
    let rename =
        MenuItem::with_id(app, TAB_RENAME, "Rename Tab", true, None::<&str>).map_err(|e| {
            format!("menu: failed to create Rename Tab item: {e}")
        })?;
    let duplicate =
        MenuItem::with_id(app, TAB_DUPLICATE, "Duplicate Tab", true, None::<&str>).map_err(
            |e| format!("menu: failed to create Duplicate Tab item: {e}"),
        )?;

    MenuBuilder::new(app)
        .item(&close)
        .item(&close_others)
        .item(&close_right)
        .separator()
        .item(&rename)
        .item(&duplicate)
        .build()
        .map_err(|e| format!("menu: failed to build tab context menu: {e}"))
}

// ─── Terminal Context Menu (4 项) ──────────────────────────────────

pub fn terminal_context_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, String> {
    let copy = PredefinedMenuItem::copy(app, None)
        .map_err(|e| format!("menu: failed to create Copy item: {e}"))?;
    let paste = PredefinedMenuItem::paste(app, None)
        .map_err(|e| format!("menu: failed to create Paste item: {e}"))?;
    let clear = MenuItem::with_id(app, TERM_CLEAR, "Clear Terminal", true, None::<&str>)
        .map_err(|e| format!("menu: failed to create Clear Terminal item: {e}"))?;
    let select_all = PredefinedMenuItem::select_all(app, None)
        .map_err(|e| format!("menu: failed to create Select All item: {e}"))?;

    MenuBuilder::new(app)
        .item(&copy)
        .item(&paste)
        .separator()
        .item(&clear)
        .item(&select_all)
        .build()
        .map_err(|e| format!("menu: failed to build terminal context menu: {e}"))
}

// ─── App Menu (macOS menubar + 全局 accelerator) ───────────────────

pub fn app_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, String> {
    let pkg_info = app.package_info();

    // Vibestation submenu
    let prefs = MenuItem::with_id(app, APP_PREFS, "Preferences...", true, Some("Cmd+,"))
        .map_err(|e| format!("menu: failed to create Preferences item: {e}"))?;
    let sep1 = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("menu: failed to create separator: {e}"))?;
    let hide = PredefinedMenuItem::hide(app, None)
        .map_err(|e| format!("menu: failed to create Hide item: {e}"))?;
    let hide_others = PredefinedMenuItem::hide_others(app, None)
        .map_err(|e| format!("menu: failed to create Hide Others item: {e}"))?;
    let show_all = MenuItem::with_id(app, "app:show_all", "Show All", true, None::<&str>)
        .map_err(|e| format!("menu: failed to create Show All item: {e}"))?;
    let quit = PredefinedMenuItem::quit(app, None)
        .map_err(|e| format!("menu: failed to create Quit item: {e}"))?;

    let app_submenu = Submenu::with_items(
        app,
        pkg_info.name.clone(),
        true,
        &[&prefs, &sep1, &hide, &hide_others, &show_all, &quit],
    )
    .map_err(|e| format!("menu: failed to create app submenu: {e}"))?;

    // Tab submenu
    let new_tab =
        MenuItem::with_id(app, APP_NEW_TAB, "New Tab", true, Some("Cmd+T")).map_err(|e| {
            format!("menu: failed to create New Tab item: {e}")
        })?;
    let close_tab =
        MenuItem::with_id(app, APP_CLOSE_TAB, "Close Tab", true, Some("Cmd+W")).map_err(|e| {
            format!("menu: failed to create Close Tab (app) item: {e}")
        })?;
    let sep2 = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("menu: failed to create separator: {e}"))?;
    let split_h = MenuItem::with_id(
        app,
        APP_SPLIT_H,
        "Split Horizontally",
        true,
        Some("Cmd+D"),
    )
    .map_err(|e| format!("menu: failed to create Split Horizontally item: {e}"))?;
    let split_v = MenuItem::with_id(
        app,
        APP_SPLIT_V,
        "Split Vertically",
        true,
        Some("Cmd+Shift+D"),
    )
    .map_err(|e| format!("menu: failed to create Split Vertically item: {e}"))?;

    let tab_submenu = Submenu::with_items(
        app,
        "Tab",
        true,
        &[&new_tab, &close_tab, &sep2, &split_h, &split_v],
    )
    .map_err(|e| format!("menu: failed to create Tab submenu: {e}"))?;

    // Window submenu
    let close_window = PredefinedMenuItem::close_window(app, None)
        .map_err(|e| format!("menu: failed to create Close Window item: {e}"))?;
    let minimize = PredefinedMenuItem::minimize(app, None)
        .map_err(|e| format!("menu: failed to create Minimize item: {e}"))?;
    let maximize = PredefinedMenuItem::maximize(app, None)
        .map_err(|e| format!("menu: failed to create Maximize item: {e}"))?;
    let sep3 = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("menu: failed to create separator: {e}"))?;
    let fullscreen = PredefinedMenuItem::fullscreen(app, None)
        .map_err(|e| format!("menu: failed to create Fullscreen item: {e}"))?;

    let window_submenu = Submenu::with_items(
        app,
        "Window",
        true,
        &[&close_window, &minimize, &maximize, &sep3, &fullscreen],
    )
    .map_err(|e| format!("menu: failed to create Window submenu: {e}"))?;

    Menu::with_items(app, &[&app_submenu, &tab_submenu, &window_submenu])
        .map_err(|e| format!("menu: failed to build app menu: {e}"))
}

// ─── Menu Event Handler ────────────────────────────────────────────

pub fn setup_menu_events<R: Runtime>(app: &AppHandle<R>) {
    let handle = app.clone();
    app.on_menu_event(move |_app, event| {
        let action = match event.id().as_ref() {
            TAB_CLOSE | APP_CLOSE_TAB => "close_tab",
            TAB_CLOSE_OTHERS => "close_other_tabs",
            TAB_CLOSE_RIGHT => "close_tabs_to_right",
            TAB_RENAME => "rename_tab",
            TAB_DUPLICATE => "duplicate_tab",
            TERM_COPY => "copy",
            TERM_PASTE => "paste",
            TERM_CLEAR => "clear_terminal",
            TERM_SELECT_ALL => "select_all",
            APP_NEW_TAB => "new_tab",
            APP_SPLIT_H => "split_horizontal",
            APP_SPLIT_V => "split_vertical",
            APP_PREFS => "preferences",
            _ => return,
        };
        if let Err(e) = handle.emit(
            "menu:action",
            MenuActionPayload {
                action: action.to_string(),
            },
        ) {
            eprintln!("[mvp-11] emit menu:action failed: {e}");
        }
    });
}

// ─── IPC Commands ──────────────────────────────────────────────────

#[tauri::command]
pub fn menu_show_tab(window: Window, x: f64, y: f64) -> Result<(), String> {
    let menu = tab_context_menu(window.app_handle())?;
    menu.popup_at(
        &window,
        Position::Physical(PhysicalPosition {
            x: x as i32,
            y: y as i32,
        }),
    )
    .map_err(|e| format!("menu: popup failed: {e}"))
}

#[tauri::command]
pub fn menu_show_terminal(window: Window, x: f64, y: f64) -> Result<(), String> {
    let menu = terminal_context_menu(window.app_handle())?;
    menu.popup_at(
        &window,
        Position::Physical(PhysicalPosition {
            x: x as i32,
            y: y as i32,
        }),
    )
    .map_err(|e| format!("menu: popup failed: {e}"))
}

#[tauri::command]
pub fn menu_register_shortcuts(app: AppHandle) -> Result<(), String> {
    let menu = app_menu(&app)?;
    app.set_menu(menu)
        .map_err(|e| format!("menu: set_menu failed: {e}"))
}

#[tauri::command]
pub fn menu_item_clicked() -> Result<(), String> {
    Ok(())
}
