// SPDX-FileCopyrightText: 2019-2022, The Tauri Programme in the Commons Conservancy
// SPDX-FileCopyrightText: Copyright (c) 2022-2026 trobonox <hello@trobo.dev>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_window_state::StateFlags;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/dev/dri").exists()
            && std::env::var("WAYLAND_DISPLAY").is_err()
            && std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11"
        {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::all() & !StateFlags::VISIBLE)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            #[cfg(desktop)]
            let _ = app
                .handle()
                .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }));

            let app_handle = app.handle().clone();
            let store_path = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir")
                .join(".kanri.dat");

            std::thread::spawn(move || {
                let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
                let mut watcher =
                    notify::recommended_watcher(tx).expect("failed to create watcher");
                watcher
                    .watch(&store_path, RecursiveMode::NonRecursive)
                    .expect("failed to watch store path");

                let mut last_emit = Instant::now() - Duration::from_secs(10);

                for res in rx {
                    if let Ok(event) = res {
                        if matches!(
                            event.kind,
                            EventKind::Modify(_) | EventKind::Create(_)
                        ) {
                            let now = Instant::now();
                            // 5s cooldown: swallows the auto-save Kanri fires after a reload
                            if now.duration_since(last_emit) > Duration::from_secs(5) {
                                last_emit = now;
                                app_handle.emit("kanri-file-changed", ()).ok();
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
