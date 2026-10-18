//! FilmCraft on Android.
//!
//! The same engine and egui UI as the desktop app, in the touch layout, on eframe's winit Android
//! backend (`NativeActivity`) and wgpu. What the desktop shell gets from the OS, this crate gets
//! from Android (see docs/mobile-roadmap.md for what is done):
//!
//! | Desktop | Android |
//! |---|---|
//! | window, input | `NativeActivity` through android-activity / winit; touch becomes egui pointer events, long press becomes a secondary click |
//! | data directory | the app's internal storage (`AndroidApp::internal_data_path`) |
//! | log file | `logcat` (tag `FilmCraft`) |
//! | file dialogs (rfd) | Storage Access Framework (not yet) |
//! | cpal audio output | AAudio / Oboe (not yet) |
//!
//! Everything here is Android-only; on other targets the crate is empty.
#![cfg(target_os = "android")]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable))]

use android_activity::AndroidApp;
use filmcraft_engine::Session;
use filmcraft_engine::autosave::AutosaveConfig;
use filmcraft_ui_egui::FilmcraftApp;
use filmcraft_ui_egui::mobile::LayoutMode;
use serde_json::json;

/// The entry point android-activity's glue calls on its own thread (the symbol must be exported
/// unmangled, which `unsafe_code` treats as unsafe: docs/adr/0002-android-entry-point.md).
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("FilmCraft"));
    log::info!("FilmCraft {} (android {})", env!("CARGO_PKG_VERSION"), std::env::consts::ARCH);
    let data_dir = app.internal_data_path();
    // Panics go to <data dir>/Logs/crash-<day>.log (and logcat); the UI pass catches them.
    filmcraft_ui_egui::crash::install(data_dir.as_ref().map(|d| d.join("Logs")));
    let options = eframe::NativeOptions { android_app: Some(app), renderer: eframe::Renderer::Wgpu, ..Default::default() };
    let result = eframe::run_native("FilmCraft", options, Box::new(move |cc| Ok(Box::new(create_app(cc, data_dir.clone())))));
    if let Err(e) = result {
        let msg = format!("FilmCraft could not start its window: {e}");
        log::error!("{msg}");
        filmcraft_ui_egui::crash::record(&msg);
    }
}

fn create_app(cc: &eframe::CreationContext<'_>, data_dir: Option<std::path::PathBuf>) -> FilmcraftApp {
    let mut session = Session::default();
    if let Some(dir) = data_dir
        && let Err(e) = session.start_autosave(AutosaveConfig::new(dir))
    {
        log::warn!("auto-save and crash recovery unavailable: {e}");
    }
    // Until the Storage Access Framework picker lands, the app opens the demo project so the
    // touch UI has something to edit (docs/mobile-roadmap.md, M2).
    if let Err(e) = session.execute("file.openDemoProject", json!({})) {
        log::error!("demo project: {e}");
    }
    let mut app = FilmcraftApp::new(session);
    app.ui.layout = LayoutMode::Touch;
    if let Some(rs) = cc.wgpu_render_state.clone() {
        app.set_wgpu(rs);
    }
    app
}
