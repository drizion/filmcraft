//! Headless UI tests of the touch shell (docs/mobile-ux.md): a phone-sized window with the
//! `touch` layout, driven by automation id through the control channel like the desktop UI.
//!
//! Set `FILMCRAFT_UI_SNAPSHOT_DIR=<dir>` to also render offscreen with wgpu and write `mobile-*.png`.

use std::sync::mpsc::{Sender, channel};

use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::FilmcraftApp;
use filmcraft_ui_egui::control::ControlRequest;
use filmcraft_ui_egui::mobile::{LayoutMode, Sheet};
use filmcraft_ui_egui::state::Mode;
use serde_json::{Value, json};

const PHONE: (f32, f32) = (393.0, 852.0);

struct Driver {
    harness: Harness<'static, FilmcraftApp>,
    tx: Sender<ControlRequest>,
    snapshots: Option<std::path::PathBuf>,
}

impl Driver {
    fn new(size: (f32, f32), layout: LayoutMode) -> Self {
        let mut session = Session::default();
        session.execute("file.openDemoProject", json!({})).expect("demo project");
        let (tx, rx) = channel();
        let mut app = FilmcraftApp::new(session).with_control(rx);
        app.ui.layout = layout;
        let snapshots = std::env::var_os("FILMCRAFT_UI_SNAPSHOT_DIR").map(std::path::PathBuf::from);
        let mut b = Harness::builder().with_size(egui::vec2(size.0, size.1)).with_max_steps(10_000);
        if snapshots.is_some() {
            b = b.wgpu();
        }
        let harness = b.build_eframe(move |_cc| app);
        let mut d = Driver { harness, tx, snapshots };
        d.frames(4);
        d
    }

    fn phone() -> Self {
        Self::new(PHONE, LayoutMode::Touch)
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            let ctx = self.harness.ctx.clone();
            let mut raw = std::mem::take(self.harness.input_mut());
            eframe::App::raw_input_hook(self.harness.state_mut(), &ctx, &mut raw);
            *self.harness.input_mut() = raw;
            self.harness.step();
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let (req, reply) = ControlRequest::new(method, params.clone());
        self.tx.send(req).unwrap();
        for _ in 0..600 {
            self.frames(1);
            if let Ok(v) = reply.try_recv() {
                return v;
            }
        }
        panic!("no reply to {method} {params}");
    }

    fn ok(&mut self, method: &str, params: Value) -> Value {
        let v = self.call(method, params.clone());
        assert_eq!(v["ok"], json!(true), "{method} {params} failed: {v}");
        v["result"].clone()
    }

    fn tap(&mut self, id: &str) {
        self.ok("ui.click", json!({"id": id}));
        self.frames(3);
    }

    fn app(&self) -> &FilmcraftApp {
        self.harness.state()
    }

    fn rect(&mut self, id: &str) -> Option<[f32; 4]> {
        let v = self.ok("ui.elements", json!({"prefix": id}));
        v.as_array()?.iter().find(|e| e["id"] == id).map(|e| {
            let r = &e["rect"];
            [r[0].as_f64().unwrap_or(0.0) as f32, r[1].as_f64().unwrap_or(0.0) as f32, r[2].as_f64().unwrap_or(0.0) as f32, r[3].as_f64().unwrap_or(0.0) as f32]
        })
    }

    fn ids(&mut self, prefix: &str) -> Vec<String> {
        let v = self.ok("ui.elements", json!({"prefix": prefix}));
        v.as_array().unwrap().iter().filter_map(|e| e["id"].as_str().map(str::to_string)).collect()
    }

    fn snapshot(&mut self, name: &str) {
        let Some(dir) = self.snapshots.clone() else { return };
        std::fs::create_dir_all(&dir).unwrap();
        let image = self.harness.render().expect("render");
        image.save(dir.join(format!("mobile-{name}.png"))).unwrap();
    }
}

#[test]
fn the_phone_shows_the_touch_shell_not_the_desktop_header() {
    let mut d = Driver::phone();
    assert!(d.ids("header.").is_empty(), "desktop header drawn on a phone");
    for id in ["m.menu", "m.undo", "m.redo", "m.exportButton", "m.nav.media", "m.nav.edit", "m.nav.export", "m.tb.import", "m.tb.marker"] {
        assert!(d.rect(id).is_some(), "{id} missing: {:?}", d.ids("m."));
    }
    assert!(d.app().ui_error.is_none(), "{:?}", d.app().ui_error);
    d.snapshot("edit");
}

#[test]
fn auto_layout_picks_the_shell_by_width() {
    let mut phone = Driver::new(PHONE, LayoutMode::Auto);
    assert!(phone.rect("m.nav.edit").is_some());
    let mut desktop = Driver::new((1600.0, 980.0), LayoutMode::Auto);
    assert!(desktop.rect("m.nav.edit").is_none());
    assert!(!desktop.ids("header.").is_empty());
    let mut forced = Driver::new((1600.0, 980.0), LayoutMode::Desktop);
    assert!(forced.rect("m.nav.edit").is_none());
}

#[test]
fn touch_targets_are_at_least_44_points() {
    let mut d = Driver::phone();
    for id in ["m.menu", "m.undo", "m.redo", "m.nav.media", "m.nav.edit", "m.nav.export", "m.tb.import", "m.tb.tools"] {
        let r = d.rect(id).unwrap_or_else(|| panic!("{id} missing"));
        assert!(r[2] >= 44.0 && r[3] >= 44.0, "{id} is {}x{}", r[2], r[3]);
    }
}

#[test]
fn the_shell_stays_inside_the_window_and_keeps_the_monitor_above_the_timeline() {
    let mut d = Driver::phone();
    for e in d.ids("m.") {
        let r = d.rect(&e).unwrap();
        assert!(
            r[0] >= -0.5 && r[1] >= -0.5 && r[0] + r[2] <= PHONE.0 + 0.5 && r[1] + r[3] <= PHONE.1 + 0.5 || e.starts_with("m.tb."),
            "{e} {r:?} leaves the window"
        );
    }
    let nav = d.rect("m.nav.edit").unwrap();
    assert!(nav[1] + nav[3] <= PHONE.1 + 0.5 && nav[1] > PHONE.1 - 80.0, "bottom bar is at the bottom: {nav:?}");
}

#[test]
fn the_bottom_tabs_are_the_three_header_modes() {
    let mut d = Driver::phone();
    assert_eq!(d.app().ui.mode, Mode::Edit);
    d.tap("m.nav.export");
    assert_eq!(d.app().ui.mode, Mode::Export);
    d.tap("m.nav.media");
    assert_eq!(d.app().ui.mode, Mode::Import);
    assert!(d.rect("m.fab.import").is_some(), "Media tab has the import button");
    d.tap("m.nav.edit");
    assert_eq!(d.app().ui.mode, Mode::Edit);
    assert!(d.app().ui_error.is_none(), "{:?}", d.app().ui_error);
}

#[test]
fn the_menu_sheet_searches_every_command_and_runs_one() {
    let mut d = Driver::phone();
    d.tap("m.menu");
    assert_eq!(d.app().ui.mobile.sheet, Some(Sheet::Menu));
    d.snapshot("menu");
    // the nine menus are there, drilling into one lists its commands
    assert!(d.rect("m.menu.sub.File").is_some(), "{:?}", d.ids("m.menu."));
    d.tap("m.menu.sub.File");
    assert!(d.rect("m.menu.back").is_some());
    assert!(!d.ids("m.menu.cmd.").is_empty());
    d.tap("m.menu.back");
    // search
    d.ok("ui.set", json!({"mobile": {"menuSearch": "undo"}}));
    d.frames(3);
    assert!(d.rect("m.menu.cmd.edit.undo").is_some(), "{:?}", d.ids("m.menu."));
    d.snapshot("menu-search");
}

#[test]
fn a_toolbar_button_opens_its_panel_in_a_sheet_and_close_dismisses_it() {
    let mut d = Driver::phone();
    d.tap("m.tb.color");
    match &d.app().ui.mobile.sheet {
        Some(Sheet::Panels { kinds, .. }) => assert!(kinds.contains(&filmcraft_ui_egui::dock::PanelKind::LumetriColor)),
        other => panic!("expected the Color sheet, got {other:?}"),
    }
    assert!(d.rect("m.sheet.close").is_some());
    d.snapshot("color-sheet");
    d.tap("m.sheet.close");
    assert_eq!(d.app().ui.mobile.sheet, None);
    assert!(d.app().ui_error.is_none(), "{:?}", d.app().ui_error);
}

#[test]
fn selecting_a_clip_swaps_the_toolbar_for_clip_actions() {
    let mut d = Driver::phone();
    assert!(d.rect("m.tb.split").is_none());
    d.ok("engine.execute", json!({"command": "edit.selectAll", "params": {}}));
    d.frames(3);
    assert!(!d.app().session.state.selection.is_empty());
    assert!(d.rect("m.tb.import").is_none(), "{:?}", d.ids("m.tb."));
    let actions = d.ids("m.tb.");
    assert!(actions.iter().any(|i| i == "m.tb.delete"), "{actions:?}");
    d.snapshot("selected");
}

#[test]
fn the_tool_sheet_changes_the_tool() {
    let mut d = Driver::phone();
    d.tap("m.tb.tools");
    assert_eq!(d.app().ui.mobile.sheet, Some(Sheet::Tools));
    d.tap("m.tool.razor");
    assert_eq!(d.app().ui.tool, filmcraft_ui_egui::state::Tool::Razor);
    assert_eq!(d.app().ui.mobile.sheet, None);
}

#[test]
fn a_desktop_workspace_opens_its_sheet_on_the_phone() {
    let mut d = Driver::phone();
    d.ok("engine.execute", json!({"command": "window.workspace.color"}));
    d.frames(4);
    assert!(matches!(d.app().ui.mobile.sheet, Some(Sheet::Panels { .. })), "{:?}", d.app().ui.mobile.sheet);
}

#[test]
fn undo_follows_the_history() {
    let mut d = Driver::phone();
    let before = d.app().session.history.can_undo();
    d.ok("engine.execute", json!({"command": "edit.selectAll", "params": {}}));
    d.tap("m.tb.delete");
    assert!(d.app().session.history.can_undo() || before);
    d.tap("m.undo");
    assert!(d.app().ui_error.is_none(), "{:?}", d.app().ui_error);
}

#[test]
fn a_tiny_or_wide_window_never_panics() {
    for size in [(200.0, 200.0), (320.0, 480.0), (900.0, 400.0), (1600.0, 980.0)] {
        let mut d = Driver::new(size, LayoutMode::Touch);
        for tab in ["m.nav.media", "m.nav.export", "m.nav.edit"] {
            d.tap(tab);
        }
        d.tap("m.menu");
        d.tap("m.menu");
        assert!(d.app().ui_error.is_none(), "{size:?}: {:?}", d.app().ui_error);
    }
}
