//! The Menu sheet: command search plus the nine menus as a drill-down list (docs/mobile-ux.md).
//!
//! The list comes from `menus::menu_items`, the same tree as `ui.menu.list`, so every command with
//! a menu path is reachable and anything the registry gains appears here without further work.

use egui::{Align2, Rect, Sense, Stroke, pos2, vec2};

use super::chrome::run;
use crate::FilmcraftApp;
use crate::icons::{self, Icon};
use crate::menus::{self, MENUS, MenuItem};
use crate::theme::Tokens;

/// Rows shown for a search (a longer list is scrolling noise on a phone).
const MAX_RESULTS: usize = 60;
const ROW: f32 = 56.0;

/// One row of the sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    /// A submenu to drill into (its full path).
    Menu { label: String, path: Vec<String> },
    /// A command; `detail` is the menu path shown under a search result.
    Command { id: String, label: String, detail: String, enabled: bool, checked: Option<bool> },
}

fn command(item: &MenuItem, detail: String) -> Entry {
    Entry::Command { id: item.id.clone(), label: item.label.clone(), detail, enabled: item.enabled, checked: item.checked }
}

/// Case-insensitive containment on the lower-cased text (char-wise, so non-ASCII capitals match).
fn matches(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(needle)
}

/// The rows for `path` (empty = top level) or, with a non-blank `search`, every matching command.
/// `recent` are command ids, shown first at the top level.
pub fn entries(items: &[MenuItem], path: &[String], search: &str, recent: &[String]) -> Vec<Entry> {
    let needle = search.trim().to_lowercase();
    if !needle.is_empty() {
        return items
            .iter()
            .filter(|i| matches(&i.label, &needle) || matches(&i.id, &needle) || i.path.iter().any(|p| matches(p, &needle)))
            .take(MAX_RESULTS)
            .map(|i| command(i, i.path.join(" ▸ ")))
            .collect();
    }
    let mut out = Vec::new();
    if path.is_empty() {
        for id in recent {
            if let Some(i) = items.iter().find(|i| &i.id == id) {
                out.push(command(i, i.path.join(" ▸ ")));
            }
        }
        for m in MENUS {
            if items.iter().any(|i| i.path.first().is_some_and(|p| p == m)) {
                out.push(Entry::Menu { label: m.to_string(), path: vec![m.to_string()] });
            }
        }
        return out;
    }
    let mut seen: Vec<&str> = Vec::new();
    for i in items.iter().filter(|i| i.path.len() >= path.len() && i.path[..path.len()] == *path) {
        match i.path.get(path.len()) {
            None => out.push(command(i, String::new())),
            Some(sub) if !seen.contains(&sub.as_str()) => {
                seen.push(sub);
                let mut p = path.to_vec();
                p.push(sub.clone());
                out.push(Entry::Menu { label: sub.clone(), path: p });
            }
            Some(_) => {}
        }
    }
    out
}

pub(super) fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let ctx = ui.ctx().clone();
    let items = menus::menu_items(app);
    // search field
    let search_rect = Rect::from_min_size(rect.min + vec2(12.0, 8.0), vec2(rect.width() - 24.0, 44.0));
    app.auto.add("m.menu.search", search_rect, tl!("Search commands…"));
    ui.painter().rect_filled(search_rect, 22.0, t.field_bg);
    let mut field =
        ui.new_child(egui::UiBuilder::new().max_rect(search_rect.shrink2(vec2(16.0, 4.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
    field.add(
        egui::TextEdit::singleline(&mut app.ui.mobile.menu_search)
            .hint_text(tl!("Search commands…"))
            .desired_width(f32::INFINITY)
            .frame(egui::Frame::NONE)
            .font(Tokens::ui(15.0)),
    );
    let mut y = search_rect.max.y + 4.0;
    // back row inside a menu
    if app.ui.mobile.menu_search.trim().is_empty() && !app.ui.mobile.menu_path.is_empty() {
        let back = Rect::from_min_size(pos2(rect.min.x, y), vec2(rect.width(), 48.0));
        let resp = ui.interact(back, egui::Id::new(("m", "menu.back")), Sense::click());
        app.auto.add("m.menu.back", back, tl!("Back"));
        icons::paint(ui.painter(), Rect::from_center_size(pos2(back.min.x + 28.0, back.center().y), vec2(18.0, 18.0)), Icon::ChevronLeft, t.text_dim);
        ui.painter().text(pos2(back.min.x + 52.0, back.center().y), Align2::LEFT_CENTER, app.ui.mobile.menu_path.join(" ▸ "), Tokens::semibold(14.0), t.text);
        if resp.clicked() {
            app.ui.mobile.menu_path.pop();
        }
        y = back.max.y;
    }
    let rows = entries(&items, &app.ui.mobile.menu_path, &app.ui.mobile.menu_search, &app.ui.mobile.recent_commands);
    let list = Rect::from_min_max(pos2(rect.min.x, y), rect.max);
    let mut picked: Option<Entry> = None;
    let mut scroll = ui.new_child(egui::UiBuilder::new().max_rect(list).layout(egui::Layout::top_down(egui::Align::Min)));
    egui::ScrollArea::vertical().id_salt("m-menu").auto_shrink([false, false]).show(&mut scroll, |ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        if rows.is_empty() {
            ui.add_space(24.0);
            ui.label(egui::RichText::new(tl!("No matching commands")).color(t.text_dim));
        }
        for (n, row) in rows.iter().enumerate() {
            let (r, resp) = ui.allocate_exact_size(vec2(list.width(), ROW), Sense::click());
            let (label, detail, enabled, checked, chevron) = match row {
                Entry::Menu { label, .. } => (label.clone(), String::new(), true, None, true),
                Entry::Command { label, detail, enabled, checked, .. } => (label.clone(), detail.clone(), *enabled, *checked, false),
            };
            let id = match row {
                Entry::Menu { path, .. } => format!("m.menu.sub.{}", path.join(".")),
                Entry::Command { id, .. } => format!("m.menu.cmd.{id}"),
            };
            app.auto.add(&id, r, &label);
            let p = ui.painter();
            if resp.is_pointer_button_down_on() {
                p.rect_filled(r, 0.0, t.pressed);
            }
            if n > 0 {
                p.line_segment([pos2(r.min.x + 16.0, r.min.y), pos2(r.max.x - 16.0, r.min.y)], Stroke::new(1.0, t.separator));
            }
            let color = if enabled { t.text } else { t.text_faint };
            let ty = if detail.is_empty() { r.center().y } else { r.center().y - 9.0 };
            p.text(pos2(r.min.x + 20.0, ty), Align2::LEFT_CENTER, &label, Tokens::ui(16.0), color);
            if !detail.is_empty() {
                p.text(pos2(r.min.x + 20.0, r.center().y + 12.0), Align2::LEFT_CENTER, &detail, Tokens::ui(12.0), t.text_dim);
            }
            if chevron {
                icons::paint(p, Rect::from_center_size(pos2(r.max.x - 28.0, r.center().y), vec2(18.0, 18.0)), Icon::ChevronRight, t.text_dim);
            } else if checked == Some(true) {
                p.circle_filled(pos2(r.max.x - 28.0, r.center().y), 6.0, t.accent);
            }
            if resp.clicked() {
                picked = Some(row.clone());
            }
        }
    });
    match picked {
        Some(Entry::Menu { path, .. }) => {
            app.ui.mobile.menu_path = path;
            app.ui.mobile.menu_search.clear();
        }
        Some(Entry::Command { id, enabled, .. }) => {
            if enabled {
                app.ui.mobile.remember(&id);
                app.ui.mobile.close_sheet();
                run(app, &ctx, &id);
            } else {
                app.ui.status = tl!("This command is not available right now").into();
            }
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, label: &str, path: &[&str], enabled: bool) -> MenuItem {
        MenuItem { id: id.into(), label: label.into(), path: path.iter().map(|s| s.to_string()).collect(), shortcut: None, enabled, checked: None }
    }

    fn sample() -> Vec<MenuItem> {
        vec![
            item("file.save", "Save", &["File"], true),
            item("file.exportMedia", "Media…", &["File", "Export"], true),
            item("file.exportEdl", "EDL…", &["File", "Export"], false),
            item("edit.undo", "Undo", &["Edit"], true),
            item("view.zoomIn", "Zoom In", &["View"], true),
        ]
    }

    #[test]
    fn the_top_level_lists_the_menus_that_have_commands_in_menu_order() {
        let rows = entries(&sample(), &[], "", &[]);
        let names: Vec<_> = rows.iter().filter_map(|r| if let Entry::Menu { label, .. } = r { Some(label.as_str()) } else { None }).collect();
        assert_eq!(names, ["File", "Edit", "View"]);
    }

    #[test]
    fn a_menu_lists_its_commands_and_submenus_once() {
        let rows = entries(&sample(), &["File".into()], "", &[]);
        assert_eq!(rows.len(), 2);
        assert!(matches!(&rows[0], Entry::Command { id, .. } if id == "file.save"));
        assert!(matches!(&rows[1], Entry::Menu { label, path } if label == "Export" && path == &["File", "Export"]));
        let sub = entries(&sample(), &["File".into(), "Export".into()], "", &[]);
        assert_eq!(sub.len(), 2);
        assert!(matches!(&sub[1], Entry::Command { enabled: false, .. }));
    }

    #[test]
    fn search_matches_label_id_and_menu_path_ignoring_case() {
        assert_eq!(entries(&sample(), &[], "ZOOM", &[]).len(), 1);
        assert_eq!(entries(&sample(), &[], "exportedl", &[]).len(), 1);
        assert_eq!(entries(&sample(), &[], "export", &[]).len(), 2);
        assert!(entries(&sample(), &[], "zzz", &[]).is_empty());
    }

    #[test]
    fn search_matches_non_ascii_capitals() {
        let items = vec![item("a", "Écran", &["View"], true)];
        assert_eq!(entries(&items, &[], "écran", &[]).len(), 1);
        assert_eq!(entries(&items, &[], "ÉCRAN", &[]).len(), 1);
    }

    #[test]
    fn search_is_capped() {
        let items: Vec<_> = (0..500).map(|n| item(&format!("c.{n}"), "Same", &["View"], true)).collect();
        assert_eq!(entries(&items, &[], "same", &[]).len(), MAX_RESULTS);
    }

    #[test]
    fn recent_commands_come_first_and_unknown_ones_are_skipped() {
        let rows = entries(&sample(), &[], "", &["edit.undo".into(), "gone.command".into()]);
        assert!(matches!(&rows[0], Entry::Command { id, detail, .. } if id == "edit.undo" && detail == "Edit"));
        assert_eq!(rows.len(), 4);
    }
}
