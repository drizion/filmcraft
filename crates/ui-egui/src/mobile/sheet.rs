//! Bottom sheets: panel bodies, the Menu and the Tool list over the Edit / Media screens.

use egui::{Align2, Color32, Rect, Sense, Stroke, pos2, vec2};

use super::chrome::button;
use super::{Sheet, SheetHeight, TARGET, menu_sheet};
use crate::FilmcraftApp;
use crate::icons::Icon;
use crate::panels;
use crate::state::Tool;
use crate::theme::Tokens;

/// Drag distance (points) that moves a sheet one snap height.
const DRAG_STEP: f32 = 48.0;
const HEADER: f32 = 52.0;

/// The tools the Tool sheet lists, with the icon and the label.
fn tools() -> [(Tool, Icon, &'static str); 12] {
    [
        (Tool::Selection, Icon::Selection, tl!("Selection")),
        (Tool::TrackSelectForward, Icon::TrackSelectFwd, tl!("Track Select Forward")),
        (Tool::Ripple, Icon::Ripple, tl!("Ripple Edit")),
        (Tool::Rolling, Icon::Rolling, tl!("Rolling Edit")),
        (Tool::RateStretch, Icon::RateStretch, tl!("Rate Stretch")),
        (Tool::Razor, Icon::Razor, tl!("Razor")),
        (Tool::Slip, Icon::Slip, tl!("Slip")),
        (Tool::Slide, Icon::Slide, tl!("Slide")),
        (Tool::Pen, Icon::Pen, tl!("Pen")),
        (Tool::Hand, Icon::Hand, tl!("Hand")),
        (Tool::Zoom, Icon::Zoom, tl!("Zoom")),
        (Tool::Type, Icon::Type, tl!("Type")),
    ]
}

/// The next snap height after a vertical drag of `dy` points (positive = down); `None` closes.
pub fn after_drag(height: SheetHeight, dy: f32) -> Option<SheetHeight> {
    if dy >= DRAG_STEP {
        height.prev()
    } else if dy <= -DRAG_STEP {
        Some(height.next())
    } else {
        Some(height)
    }
}

pub(super) fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, body: Rect) {
    let Some(kind) = app.ui.mobile.sheet.clone() else { return };
    let t = app.tokens;
    let height = (body.height() * app.ui.mobile.sheet_height.fraction()).clamp(HEADER * 2.0, body.height().max(HEADER * 2.0));
    let sheet = Rect::from_min_max(pos2(body.min.x, (body.max.y - height).max(body.min.y)), body.max);
    // The Menu and Tool sheets are modal: a tap on the scrim above closes them.
    if !matches!(kind, Sheet::Panels { .. }) && sheet.min.y > body.min.y {
        let scrim = Rect::from_min_max(body.min, pos2(body.max.x, sheet.min.y));
        ui.painter().rect_filled(scrim, 0.0, Color32::from_black_alpha(110));
        let resp = ui.interact(scrim, egui::Id::new(("m", "scrim")), Sense::click());
        app.auto.add("m.sheet.scrim", scrim, "");
        if resp.clicked() {
            app.ui.mobile.close_sheet();
            return;
        }
    }
    let p = ui.painter().clone();
    p.rect_filled(sheet, 0.0, t.panel_bg);
    p.rect_filled(Rect::from_min_size(sheet.min, vec2(sheet.width(), 14.0)), 0.0, t.panel_bg);
    p.line_segment([sheet.left_top(), sheet.right_top()], Stroke::new(1.0, t.separator));
    // Drag handle: drag down to lower or close, up to raise; tap to cycle.
    let handle = Rect::from_min_size(sheet.min, vec2(sheet.width(), 22.0));
    let resp = ui.interact(handle, egui::Id::new(("m", "sheet.handle")), Sense::click_and_drag());
    app.auto.add("m.sheet.handle", handle, tl!("Resize"));
    p.rect_filled(Rect::from_center_size(pos2(handle.center().x, handle.min.y + 10.0), vec2(36.0, 4.0)), 2.0, t.text_faint);
    let drag_id = egui::Id::new(("m", "sheet.drag"));
    if resp.dragged() {
        let total = ui.ctx().data(|d| d.get_temp::<f32>(drag_id)).unwrap_or(0.0) + resp.drag_delta().y;
        ui.ctx().data_mut(|d| d.insert_temp(drag_id, total));
    }
    if resp.drag_stopped() {
        let dy = ui.ctx().data_mut(|d| d.remove_temp::<f32>(drag_id)).unwrap_or(0.0);
        match after_drag(app.ui.mobile.sheet_height, dy) {
            Some(h) => app.ui.mobile.sheet_height = h,
            None => {
                app.ui.mobile.close_sheet();
                return;
            }
        }
    } else if resp.clicked() {
        app.ui.mobile.sheet_height = match app.ui.mobile.sheet_height {
            SheetHeight::Peek => SheetHeight::Half,
            SheetHeight::Half => SheetHeight::Full,
            SheetHeight::Full => SheetHeight::Half,
        };
    }
    // Header: segmented panel tabs (or a title) and Close
    let head = Rect::from_min_size(sheet.min + vec2(0.0, 18.0), vec2(sheet.width(), HEADER - 18.0));
    let close = Rect::from_center_size(pos2(head.max.x - TARGET / 2.0 - 4.0, head.center().y), vec2(TARGET, TARGET));
    if button(app, ui, "sheet.close", close, Icon::Close, "", false, true) {
        app.ui.mobile.close_sheet();
        return;
    }
    let content = Rect::from_min_max(pos2(sheet.min.x, sheet.min.y + HEADER), sheet.max);
    match kind {
        Sheet::Panels { kinds, active } => {
            let active = active.min(kinds.len().saturating_sub(1));
            let mut x = sheet.min.x + 12.0;
            let mut picked = None;
            for (i, k) in kinds.iter().enumerate() {
                let title = crate::i18n::t(k.title());
                let g = p.layout_no_wrap(title.to_string(), Tokens::semibold(14.0), t.text);
                let r = Rect::from_min_size(pos2(x, head.min.y + 2.0), vec2(g.size().x + 24.0, head.height() - 4.0));
                if r.max.x > close.min.x {
                    break;
                }
                let resp = ui.interact(r, egui::Id::new(("m", "sheet.tab", i)), Sense::click());
                app.auto.add(&format!("m.sheet.tab.{}", k.title().to_ascii_lowercase().replace(' ', "")), r, k.title());
                if i == active {
                    p.rect_filled(r, 16.0, t.pill_active_bg);
                }
                let col = if i == active { t.tab_text_active } else { t.tab_text };
                p.galley_with_override_text_color(pos2(r.min.x + 12.0, r.center().y - g.size().y / 2.0), g, col);
                if resp.clicked() {
                    picked = Some(i);
                }
                x = r.max.x + 4.0;
            }
            if let Some(i) = picked {
                app.ui.mobile.sheet = Some(Sheet::Panels { kinds: kinds.clone(), active: i });
            }
            if let Some(k) = kinds.get(active).copied() {
                app.ui.focused = k;
                panels::show(app, ui, k, content);
            }
        }
        Sheet::Menu => {
            p.text(pos2(sheet.min.x + 16.0, head.center().y), Align2::LEFT_CENTER, tl!("Menu"), Tokens::semibold(16.0), t.text);
            menu_sheet::show(app, ui, content);
        }
        Sheet::Tools => {
            p.text(pos2(sheet.min.x + 16.0, head.center().y), Align2::LEFT_CENTER, tl!("Tools"), Tokens::semibold(16.0), t.text);
            tool_list(app, ui, content);
        }
    }
}

fn tool_list(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let cols = 3usize;
    let w = rect.width() / cols as f32;
    let mut chosen = None;
    for (n, (tool, icon, label)) in tools().into_iter().enumerate() {
        let (c, r) = (n % cols, n / cols);
        let cell = Rect::from_min_size(pos2(rect.min.x + w * c as f32, rect.min.y + 84.0 * r as f32), vec2(w, 84.0));
        if cell.max.y > rect.max.y {
            break;
        }
        if button(app, ui, &format!("tool.{tool:?}").to_ascii_lowercase(), cell, icon, label, app.ui.tool == tool, true) {
            chosen = Some(tool);
        }
    }
    if let Some(tool) = chosen {
        app.ui.tool = tool;
        app.ui.mobile.close_sheet();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_a_sheet_down_lowers_then_closes_it() {
        assert_eq!(after_drag(SheetHeight::Full, 60.0), Some(SheetHeight::Half));
        assert_eq!(after_drag(SheetHeight::Half, 60.0), Some(SheetHeight::Peek));
        assert_eq!(after_drag(SheetHeight::Peek, 60.0), None);
    }

    #[test]
    fn dragging_up_raises_and_small_drags_do_nothing() {
        assert_eq!(after_drag(SheetHeight::Peek, -60.0), Some(SheetHeight::Half));
        assert_eq!(after_drag(SheetHeight::Full, -60.0), Some(SheetHeight::Full));
        assert_eq!(after_drag(SheetHeight::Half, 10.0), Some(SheetHeight::Half));
        assert_eq!(after_drag(SheetHeight::Half, f32::NAN), Some(SheetHeight::Half));
    }
}
