//! The Edit and Media screens: Program monitor, Timeline and the contextual toolbar.

use egui::{Rect, Sense, Stroke, pos2, vec2};

use super::chrome::{button, run};
use super::{SheetHeight, TOOLBAR, monitor_height};
use crate::FilmcraftApp;
use crate::dock::PanelKind;
use crate::icons::{self, Icon};
use crate::panels;

/// What a toolbar button does.
#[derive(Clone, Copy)]
enum Act {
    Command(&'static str),
    Sheet(&'static [PanelKind], SheetHeight),
    /// The Menu sheet opened on one of the nine menus.
    Menu(&'static str),
    Tools,
}

struct Item {
    id: &'static str,
    label: &'static str,
    icon: Icon,
    act: Act,
}

fn item(id: &'static str, label: &'static str, icon: Icon, act: Act) -> Item {
    Item { id, label, icon, act }
}

const TITLES: &[PanelKind] = &[PanelKind::EssentialGraphics, PanelKind::Properties];
const CAPTIONS: &[PanelKind] = &[PanelKind::Text];
const EFFECTS: &[PanelKind] = &[PanelKind::Effects];
const INSPECT: &[PanelKind] = &[PanelKind::EffectControls, PanelKind::Effects, PanelKind::Properties];
const COLOR: &[PanelKind] = &[PanelKind::LumetriColor, PanelKind::LumetriScopes];
const AUDIO: &[PanelKind] = &[PanelKind::AudioTrackMixer, PanelKind::AudioClipMixer, PanelKind::EssentialSound, PanelKind::AudioMeters];

/// The buttons for what is selected in the timeline (docs/mobile-ux.md, Contextual toolbar).
fn items(selected: usize) -> Vec<Item> {
    match selected {
        0 => vec![
            item("import", tl!("Import"), Icon::Import, Act::Command("file.import")),
            item("titles", tl!("Titles"), Icon::Type, Act::Sheet(TITLES, SheetHeight::Half)),
            item("captions", tl!("Captions"), Icon::Captions, Act::Sheet(CAPTIONS, SheetHeight::Half)),
            item("effects", tl!("Effects"), Icon::Fx, Act::Sheet(EFFECTS, SheetHeight::Half)),
            item("color", tl!("Color"), Icon::Adjust, Act::Sheet(COLOR, SheetHeight::Half)),
            item("audio", tl!("Audio"), Icon::Audio, Act::Sheet(AUDIO, SheetHeight::Half)),
            item("marker", tl!("Marker"), Icon::Marker, Act::Command("markers.add")),
            item("tools", tl!("Tools"), Icon::Wrench, Act::Tools),
        ],
        1 => vec![
            item("split", tl!("Split"), Icon::Razor, Act::Command("sequence.addEdit")),
            item("delete", tl!("Delete"), Icon::Trash, Act::Command("edit.rippleDelete")),
            item("duplicate", tl!("Duplicate"), Icon::NewItem, Act::Command("edit.duplicate")),
            item("speed", tl!("Speed"), Icon::Stopwatch, Act::Command("clip.speedDuration")),
            item("volume", tl!("Volume"), Icon::Speaker, Act::Command("clip.audioGain")),
            item("inspect", tl!("Effects"), Icon::Fx, Act::Sheet(INSPECT, SheetHeight::Half)),
            item("color", tl!("Color"), Icon::Adjust, Act::Sheet(COLOR, SheetHeight::Half)),
            item("more", tl!("More"), Icon::Hamburger, Act::Menu("Clip")),
        ],
        _ => vec![
            item("delete", tl!("Delete"), Icon::Trash, Act::Command("edit.rippleDelete")),
            item("duplicate", tl!("Duplicate"), Icon::NewItem, Act::Command("edit.duplicate")),
            item("group", tl!("Group"), Icon::Link, Act::Command("clip.group")),
            item("link", tl!("Link"), Icon::Link, Act::Command("clip.link")),
            item("nest", tl!("Nest"), Icon::Nest, Act::Command("clip.nest")),
            item("more", tl!("More"), Icon::Hamburger, Act::Menu("Clip")),
        ],
    }
}

fn toolbar(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let ctx = ui.ctx().clone();
    ui.painter().rect_filled(rect, 0.0, t.tab_bg);
    ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, t.separator));
    let items = items(app.session.state.selection.len());
    let w = 72.0;
    let mut tapped = None;
    let mut scroll = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
    egui::ScrollArea::horizontal()
        .id_salt("m-toolbar")
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut scroll, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for it in &items {
                let (r, _) = ui.allocate_exact_size(vec2(w, rect.height()), Sense::hover());
                if button(app, ui, &format!("tb.{}", it.id), r, it.icon, it.label, false, true) {
                    tapped = Some(it.act);
                }
            }
        });
    match tapped {
        Some(Act::Command(id)) => run(app, &ctx, id),
        Some(Act::Sheet(kinds, height)) => app.ui.mobile.open_panels(kinds, height),
        Some(Act::Menu(menu)) => {
            app.ui.mobile.menu_path = vec![menu.to_string()];
            app.ui.mobile.menu_search.clear();
            app.ui.mobile.sheet = Some(super::Sheet::Menu);
            app.ui.mobile.sheet_height = SheetHeight::Full;
        }
        Some(Act::Tools) => {
            app.ui.mobile.sheet = Some(super::Sheet::Tools);
            app.ui.mobile.sheet_height = SheetHeight::Half;
        }
        None => {}
    }
}

/// The Edit tab: Program monitor over Timeline over the toolbar (portrait), or monitor beside
/// timeline (wide windows).
pub(super) fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, body: Rect) {
    if !matches!(app.ui.focused, PanelKind::Program | PanelKind::Timeline) {
        app.ui.focused = PanelKind::Timeline;
    }
    let bar = Rect::from_min_max(pos2(body.min.x, (body.max.y - TOOLBAR).max(body.min.y)), body.max);
    let stage = Rect::from_min_max(body.min, pos2(body.max.x, bar.min.y));
    let (monitor, timeline) = if body.width() > body.height() {
        // landscape: monitor left, timeline right
        let split = body.min.x + stage.width() * 0.45;
        (Rect::from_min_max(stage.min, pos2(split, stage.max.y)), Rect::from_min_max(pos2(split + 1.0, stage.min.y), stage.max))
    } else {
        let h = monitor_height(stage.width(), stage.height());
        (Rect::from_min_size(stage.min, vec2(stage.width(), h)), Rect::from_min_max(pos2(stage.min.x, stage.min.y + h + 1.0), stage.max))
    };
    panels::show(app, ui, PanelKind::Program, monitor);
    panels::show(app, ui, PanelKind::Timeline, timeline);
    toolbar(app, ui, bar);
}

/// The Media tab: the Project panel with an Import action button.
pub(super) fn media(app: &mut FilmcraftApp, ui: &mut egui::Ui, body: Rect) {
    app.ui.focused = PanelKind::Project;
    panels::show(app, ui, PanelKind::Project, body);
    let ctx = ui.ctx().clone();
    let t = app.tokens;
    let fab = Rect::from_center_size(pos2(body.max.x - 16.0 - 28.0, body.max.y - 16.0 - 28.0), vec2(56.0, 56.0));
    let resp = ui.interact(fab, egui::Id::new(("m", "fab.import")), Sense::click());
    app.auto.add("m.fab.import", fab, tl!("Import"));
    ui.painter().circle_filled(fab.center(), 28.0, if resp.is_pointer_button_down_on() { t.accent_hover } else { t.accent });
    icons::paint(ui.painter(), Rect::from_center_size(fab.center(), vec2(24.0, 24.0)), Icon::Plus, egui::Color32::WHITE);
    if resp.clicked() {
        run(app, &ctx, "file.import");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_toolbar_changes_with_the_selection() {
        let none: Vec<_> = items(0).iter().map(|i| i.id).collect();
        let one: Vec<_> = items(1).iter().map(|i| i.id).collect();
        let many: Vec<_> = items(3).iter().map(|i| i.id).collect();
        assert!(none.contains(&"import") && !none.contains(&"split"));
        assert!(one.contains(&"split") && one.contains(&"delete"));
        assert!(many.contains(&"group") && !many.contains(&"split"));
    }

    #[test]
    fn toolbar_ids_are_unique_per_state() {
        for n in [0, 1, 2] {
            let ids: Vec<_> = items(n).iter().map(|i| i.id).collect();
            let mut sorted = ids.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(ids.len(), sorted.len(), "duplicate toolbar id for {n} selected");
        }
    }
}
