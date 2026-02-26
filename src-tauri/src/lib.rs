mod clipboard;
mod commands;
mod engine;
mod models;
pub mod scheduler;
pub mod server;
pub mod speed_limiter;
mod state;

use engine::ActiveDownloads;
use state::AppState;
#[cfg(not(mobile))]
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
use tauri::{Manager, RunEvent, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let app_state = AppState::new(data_dir);
            app.manage(app_state);
            app.manage(ActiveDownloads::new());

            #[cfg(not(mobile))]
            {
                // ── System Tray ──────────────────────────────────────────
                let show = MenuItemBuilder::with_id("show", "Show Deras").build(app)?;
                let pause_all = MenuItemBuilder::with_id("pause_all", "Pause All").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

                let menu = MenuBuilder::new(app)
                    .item(&show)
                    .separator()
                    .item(&pause_all)
                    .separator()
                    .item(&quit)
                    .build()?;

                let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

                TrayIconBuilder::new()
                    .icon(icon)
                    .tooltip("Deras Download Manager")
                    .menu(&menu)
                    .on_menu_event(move |app, event| match event.id().as_ref() {
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "pause_all" => {
                            // Pause all active downloads
                            let active = app.state::<ActiveDownloads>();
                            let tokens = active.tokens.lock().unwrap();
                            for token in tokens.values() {
                                token.cancel();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click { .. } = event {
                            let app = tray.app_handle();
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            // ── Clipboard Monitoring ─────────────────────────────────
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                clipboard::start_clipboard_monitor(handle).await;
            });

            // ── Scheduler Monitoring ─────────────────────────────────
            let handle2 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler::start_scheduler_loop(handle2).await;
            });

            // ── Local API Server ─────────────────────────────────────
            #[cfg(not(mobile))]
            {
                let handle3 = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    server::start_local_server(handle3).await;
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_downloads,
            commands::add_download,
            commands::remove_download,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::move_download,
            commands::force_start,
            commands::verify_checksum,
            commands::set_global_speed_limit,
            commands::set_download_speed_limit,
            commands::set_schedule_config,
            commands::update_download_url,
            commands::save_credential,
            commands::delete_credential,
            commands::get_credentials,
            commands::get_task_logs,
            commands::open_folder,
            commands::redownload_task,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // Intercept window close → hide to tray instead of quitting
        #[cfg(not(mobile))]
        if let RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } = &event
        {
            if label == "main" {
                api.prevent_close();
                if let Some(win) = app_handle.get_webview_window("main") {
                    let _ = win.hide();
                }
            }
        }
    });
}
