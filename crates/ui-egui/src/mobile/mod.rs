//! The touch shell: FilmCraft's phone / tablet layout (docs/mobile-ux.md).
//!
//! The same engine, panels and commands as the desktop UI, arranged for a touch screen: a top bar,
//! three bottom tabs (the three header modes: Media = Import, Edit, Export), a contextual toolbar
//! under the timeline and bottom sheets in place of docked panels. Chosen by [`UiState::layout`]
//! (`auto` follows the window width). Everything here is serde state plus automation ids with the
//! prefix `m.`, so agents drive it through the control channel like the rest of the UI.

mod chrome;
mod edit;
mod menu_sheet;
mod sheet;

use egui::{Rect, vec2};
use serde::{Deserialize, Serialize};

use crate::FilmcraftApp;
use crate::dock::PanelKind;

/// Window width (points) below which `auto` picks the phone shell.
pub const COMPACT_MAX: f32 = 600.0;
/// Window width (points) from which a touch screen keeps the desktop dock (tablets).
pub const EXPANDED_MIN: f32 = 840.0;

/// Smallest comfortable touch target (Material: 48 dp).
pub const TARGET: f32 = 48.0;
pub const TOP_BAR: f32 = 52.0;
pub const NAV_BAR: f32 = 56.0;
pub const TOOLBAR: f32 = 64.0;

/// Which layout the window uses (`UiState::layout`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LayoutMode {
    /// Phone shell below [`COMPACT_MAX`] points wide, the desktop dock otherwise.
    #[default]
    Auto,
    Desktop,
    /// Always the touch shell (the Android host sets this).
    Touch,
}

/// Material window size classes, from the window width in points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeClass {
    Compact,
    Medium,
    Expanded,
}

impl SizeClass {
    pub fn of_width(width: f32) -> SizeClass {
        if width.is_nan() || width < COMPACT_MAX {
            SizeClass::Compact
        } else if width < EXPANDED_MIN {
            SizeClass::Medium
        } else {
            SizeClass::Expanded
        }
    }
}

/// Whether a window `width` points wide gets the touch shell under `mode`.
pub fn uses_touch_shell(mode: LayoutMode, width: f32) -> bool {
    match mode {
        LayoutMode::Desktop => false,
        LayoutMode::Touch => true,
        LayoutMode::Auto => SizeClass::of_width(width) == SizeClass::Compact,
    }
}

/// Heights a sheet snaps to, as a share of the body.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SheetHeight {
    /// 30 %: the monitor and playhead stay visible for live adjustments.
    Peek,
    #[default]
    Half,
    Full,
}

impl SheetHeight {
    pub fn fraction(self) -> f32 {
        match self {
            SheetHeight::Peek => 0.30,
            SheetHeight::Half => 0.60,
            SheetHeight::Full => 1.0,
        }
    }
    pub fn next(self) -> SheetHeight {
        match self {
            SheetHeight::Peek => SheetHeight::Half,
            SheetHeight::Half | SheetHeight::Full => SheetHeight::Full,
        }
    }
    pub fn prev(self) -> Option<SheetHeight> {
        match self {
            SheetHeight::Full => Some(SheetHeight::Half),
            SheetHeight::Half => Some(SheetHeight::Peek),
            SheetHeight::Peek => None,
        }
    }
}

/// The sheet over the Edit / Media screens, if any.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Sheet {
    /// One or more panel bodies, shown as a segmented control when there are several.
    Panels { kinds: Vec<PanelKind>, active: usize },
    /// All commands: search plus the nine menus.
    Menu,
    /// The tool list (Selection, Ripple, Roll, Slip, Slide, Razor, Hand, Zoom, Type…).
    Tools,
}

/// Touch shell state; saved with the UI state so agents can read and set it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MobileState {
    pub sheet: Option<Sheet>,
    pub sheet_height: SheetHeight,
    /// Menu sheet: the search text, the open submenu path and the commands run last.
    pub menu_search: String,
    pub menu_path: Vec<String>,
    pub recent_commands: Vec<String>,
    /// The desktop workspace last mapped to a sheet, so a change opens its sheet once.
    pub seen_workspace: String,
}

impl MobileState {
    pub fn open_panels(&mut self, kinds: &[PanelKind], height: SheetHeight) {
        self.sheet = Some(Sheet::Panels { kinds: kinds.to_vec(), active: 0 });
        self.sheet_height = height;
    }
    pub fn close_sheet(&mut self) {
        self.sheet = None;
    }
    /// Remember a command the user ran from the palette (most recent first, five kept).
    pub fn remember(&mut self, id: &str) {
        self.recent_commands.retain(|c| c != id);
        self.recent_commands.insert(0, id.to_string());
        self.recent_commands.truncate(5);
    }
}

/// The phone workspace each desktop workspace opens (docs/mobile-ux.md, Sheets).
pub fn workspace_sheet(workspace: &str) -> Option<(&'static [PanelKind], SheetHeight)> {
    match workspace {
        "Color" => Some((&[PanelKind::LumetriColor, PanelKind::LumetriScopes], SheetHeight::Half)),
        "Audio" => Some((&[PanelKind::AudioTrackMixer, PanelKind::AudioClipMixer, PanelKind::EssentialSound, PanelKind::AudioMeters], SheetHeight::Half)),
        "Effects" => Some((&[PanelKind::Effects, PanelKind::EffectControls], SheetHeight::Half)),
        "Captions and Graphics" => Some((&[PanelKind::EssentialGraphics, PanelKind::Text, PanelKind::Properties], SheetHeight::Half)),
        _ => None,
    }
}

/// Padding the shell leaves around the screen for system bars and cut-outs.
fn safe_rect(ctx: &egui::Context) -> Rect {
    ctx.input(|i| i.content_rect())
}

/// Draw the touch shell over `full` (the whole window). Called from the frame loop instead of the
/// desktop header, dock and status bar.
pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, full: Rect) {
    let t = app.tokens;
    let area = {
        let safe = safe_rect(ui.ctx());
        // a host without insets (desktop preview) reports the whole window
        let r = full.intersect(safe);
        if r.is_positive() { r } else { full }
    };
    ui.painter().rect_filled(full, 0.0, t.app_bg);
    if app.ui.mobile.seen_workspace != app.ui.workspace {
        app.ui.mobile.seen_workspace = app.ui.workspace.clone();
        match workspace_sheet(&app.ui.workspace) {
            Some((kinds, height)) => app.ui.mobile.open_panels(kinds, height),
            None => app.ui.mobile.close_sheet(),
        }
    }
    let top = Rect::from_min_size(area.min, vec2(area.width(), TOP_BAR));
    let nav = Rect::from_min_max(egui::pos2(area.min.x, area.max.y - NAV_BAR), area.max);
    let body = Rect::from_min_max(egui::pos2(area.min.x, top.max.y), egui::pos2(area.max.x, nav.min.y));
    match app.ui.mode {
        crate::state::Mode::Edit => edit::show(app, ui, body),
        crate::state::Mode::Import => edit::media(app, ui, body),
        crate::state::Mode::Export => crate::panels::export_mode::show(app, ui, body),
    }
    chrome::top_bar(app, ui, top);
    chrome::nav_bar(app, ui, nav);
    sheet::show(app, ui, body);
}

/// The Program monitor's rectangle for a body of this size: 16:9 plus its transport, at most
/// 42 % of the body in portrait.
pub fn monitor_height(width: f32, body_height: f32) -> f32 {
    let wanted = width * 9.0 / 16.0 + 64.0;
    wanted.min(body_height * 0.42).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_classes_follow_the_width() {
        assert_eq!(SizeClass::of_width(393.0), SizeClass::Compact);
        assert_eq!(SizeClass::of_width(599.9), SizeClass::Compact);
        assert_eq!(SizeClass::of_width(600.0), SizeClass::Medium);
        assert_eq!(SizeClass::of_width(839.0), SizeClass::Medium);
        assert_eq!(SizeClass::of_width(840.0), SizeClass::Expanded);
    }

    #[test]
    fn a_hostile_width_is_compact_not_a_panic() {
        assert_eq!(SizeClass::of_width(f32::NAN), SizeClass::Compact);
        assert_eq!(SizeClass::of_width(-5.0), SizeClass::Compact);
        assert_eq!(SizeClass::of_width(f32::INFINITY), SizeClass::Expanded);
    }

    #[test]
    fn layout_modes_choose_the_shell() {
        assert!(uses_touch_shell(LayoutMode::Touch, 1600.0));
        assert!(!uses_touch_shell(LayoutMode::Desktop, 300.0));
        assert!(uses_touch_shell(LayoutMode::Auto, 393.0));
        assert!(!uses_touch_shell(LayoutMode::Auto, 1600.0));
    }

    #[test]
    fn recent_commands_keep_five_most_recent_first() {
        let mut m = MobileState::default();
        for id in ["a", "b", "c", "d", "e", "f", "c"] {
            m.remember(id);
        }
        assert_eq!(m.recent_commands, ["c", "f", "e", "d", "b"]);
    }

    #[test]
    fn monitor_height_is_bounded_by_the_body() {
        assert!(monitor_height(393.0, 600.0) <= 600.0 * 0.42 + 0.01);
        assert_eq!(monitor_height(393.0, 0.0), 0.0);
    }

    #[test]
    fn sheet_heights_step_up_and_down() {
        assert_eq!(SheetHeight::Peek.next(), SheetHeight::Half);
        assert_eq!(SheetHeight::Full.prev(), Some(SheetHeight::Half));
        assert_eq!(SheetHeight::Peek.prev(), None);
    }

    #[test]
    fn mobile_state_round_trips_and_loads_from_nothing() {
        let mut m = MobileState::default();
        m.open_panels(&[PanelKind::LumetriColor], SheetHeight::Peek);
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(serde_json::from_value::<MobileState>(v).unwrap(), m);
        assert_eq!(serde_json::from_value::<MobileState>(serde_json::json!({})).unwrap(), MobileState::default());
    }
}
