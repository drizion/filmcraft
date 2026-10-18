//! Top bar and bottom navigation of the touch shell, plus the shared touch button.

use egui::{Align2, Color32, Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use super::{NAV_BAR, Sheet, TARGET};
use crate::FilmcraftApp;
use crate::icons::{self, Icon};
use crate::state::Mode;
use crate::theme::Tokens;

/// A touch button: icon over an optional label, registered for automation as `m.<id>`. Returns
/// true when tapped. `active` paints the pressed / selected state; a disabled button is dimmed and
/// ignores taps.
pub(super) fn button(app: &mut FilmcraftApp, ui: &mut egui::Ui, id: &str, rect: Rect, icon: Icon, label: &str, active: bool, enabled: bool) -> bool {
    let t = app.tokens;
    let resp = ui.interact(rect, egui::Id::new(("m", id)), if enabled { Sense::click() } else { Sense::hover() });
    app.auto.add(&format!("m.{id}"), rect, label);
    let p = ui.painter();
    if active {
        p.rect_filled(rect.shrink2(vec2(4.0, 6.0)), 12.0, t.pill_active_bg);
    } else if resp.is_pointer_button_down_on() && enabled {
        p.rect_filled(rect.shrink2(vec2(4.0, 6.0)), 12.0, t.pressed);
    }
    let color = if !enabled {
        t.text_faint
    } else if active {
        t.tab_text_active
    } else {
        t.text_dim
    };
    if label.is_empty() {
        icons::paint(p, Rect::from_center_size(rect.center(), vec2(22.0, 22.0)), icon, color);
    } else {
        icons::paint(p, Rect::from_center_size(pos2(rect.center().x, rect.center().y - 9.0), vec2(22.0, 22.0)), icon, color);
        p.text(pos2(rect.center().x, rect.center().y + 14.0), Align2::CENTER_CENTER, label, Tokens::ui(11.0), color);
    }
    enabled && resp.clicked()
}

/// Run a command from a touch control: report an error in the status line and carry on.
pub(super) fn run(app: &mut FilmcraftApp, ctx: &egui::Context, id: &str) {
    if let Err(e) = crate::menus::invoke(app, ctx, id, json!({})) {
        app.ui.status = e;
    }
}

pub(super) fn top_bar(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let ctx = ui.ctx().clone();
    ui.painter().rect_filled(rect, 0.0, t.header_bg);
    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.separator));
    let cy = rect.center().y;
    // Menu: every command and the nine menus
    let menu = Rect::from_center_size(pos2(rect.min.x + TARGET / 2.0 + 4.0, cy), vec2(TARGET, TARGET));
    if button(app, ui, "menu", menu, Icon::Hamburger, "", app.ui.mobile.sheet == Some(Sheet::Menu), true) {
        app.ui.mobile.sheet = if app.ui.mobile.sheet == Some(Sheet::Menu) { None } else { Some(Sheet::Menu) };
        app.ui.mobile.sheet_height = super::SheetHeight::Full;
    }
    // Export on the far right, then Redo and Undo
    let w = 84.0;
    let export = Rect::from_center_size(pos2(rect.max.x - 8.0 - w / 2.0, cy), vec2(w, 36.0));
    let resp = ui.interact(export, egui::Id::new(("m", "export")), Sense::click());
    app.auto.add("m.exportButton", export, tl!("Export"));
    ui.painter().rect_filled(export, 18.0, if resp.is_pointer_button_down_on() { t.accent_hover } else { t.accent });
    ui.painter().text(export.center(), Align2::CENTER_CENTER, tl!("Export"), Tokens::semibold(14.0), Color32::WHITE);
    if resp.clicked() {
        app.ui.mode = Mode::Export;
        app.ui.mobile.close_sheet();
    }
    let redo = Rect::from_center_size(pos2(export.min.x - 4.0 - TARGET / 2.0, cy), vec2(TARGET, TARGET));
    let can_redo = app.session.history.can_redo();
    if button(app, ui, "redo", redo, Icon::Redo, "", false, can_redo) {
        run(app, &ctx, "edit.redo");
    }
    let undo = Rect::from_center_size(pos2(redo.min.x - TARGET / 2.0, cy), vec2(TARGET, TARGET));
    let can_undo = app.session.history.can_undo();
    if button(app, ui, "undo", undo, Icon::Undo, "", false, can_undo) {
        run(app, &ctx, "edit.undo");
    }
    // Project name: tap for the Project panel
    let title_rect = Rect::from_min_max(pos2(menu.max.x + 2.0, rect.min.y), pos2(undo.min.x - 2.0, rect.max.y));
    let name = if app.session.is_dirty() { tlf!("{name} - Edited", name = app.session.project.name) } else { app.session.project.name.clone() };
    let resp = ui.interact(title_rect, egui::Id::new(("m", "title")), Sense::click());
    app.auto.add("m.title", title_rect, &name);
    let mut job = egui::text::LayoutJob::simple_singleline(name, Tokens::semibold(15.0), t.tab_text_active);
    job.wrap.max_width = title_rect.width().max(0.0);
    job.wrap.max_rows = 1;
    let galley = ui.painter().layout_job(job);
    ui.painter().with_clip_rect(title_rect).galley(pos2(title_rect.min.x + 4.0, cy - galley.size().y / 2.0), galley, t.tab_text_active);
    if resp.clicked() {
        app.ui.mobile.open_panels(&[crate::dock::PanelKind::Project], super::SheetHeight::Full);
    }
}

/// The three tabs are the header modes: Media (Import), Edit, Export.
pub(super) fn nav_bar(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    ui.painter().rect_filled(rect, 0.0, t.header_bg);
    ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, t.separator));
    let tabs = [
        (Mode::Import, "nav.media", Icon::Folder, tl!("Media")),
        (Mode::Edit, "nav.edit", Icon::Sequence, tl!("Edit")),
        (Mode::Export, "nav.export", Icon::Export, tl!("Export")),
    ];
    let w = rect.width() / tabs.len() as f32;
    for (i, (mode, id, icon, label)) in tabs.into_iter().enumerate() {
        let r = Rect::from_min_size(pos2(rect.min.x + w * i as f32, rect.min.y), vec2(w, NAV_BAR.min(rect.height())));
        if button(app, ui, id, r, icon, label, app.ui.mode == mode, true) && app.ui.mode != mode {
            app.ui.mode = mode;
            app.ui.mobile.close_sheet();
        }
    }
}
