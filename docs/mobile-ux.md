# FilmCraft mobile: screens and touch UX

> **Status:** design, with the first slice implemented (`crates/ui-egui/src/mobile/`). Progress is
> tracked in [mobile-roadmap.md](mobile-roadmap.md); build and install in [mobile.md](mobile.md).

The desktop UI is a Premiere-style dock of 28 panels, nine menus, 143 keyboard-only commands and
pointer gestures that need a mouse (hover, right click, middle drag, Ctrl/Alt modifiers). None of
that works on a 6-inch touch screen. This page redesigns the *shell* for touch and keeps
everything the project already documents: the same panels, the same commands, the same state.

## Principles

1. **One engine, one command set.** The mobile shell changes layout and input only. Every action
   is still a command id ([architecture §4](architecture.md)); UI state stays serde so the control
   channel, CLI and MCP drive the phone UI exactly like the desktop one
   ([agents.md](agents.md), [control-protocol.md](control-protocol.md)). Every new widget has an
   automation id (prefix `m.`).
2. **No hover, no right click, no keyboard.** Hover tooltips become labels; right click becomes
   long press (egui turns a long touch into `secondary_clicked`, so every existing context menu
   keeps working); shortcuts become buttons or palette entries. A hardware keyboard (tablet,
   Chromebook, DeX) still works with the Premiere / FilmCraft key presets ([keyboard.md](keyboard.md)).
3. **Thumb zone first.** Navigation and the most-used actions sit in the bottom third; the top bar
   holds only project-level actions. Nothing essential is in the top corners.
4. **48 dp targets, 8 dp gaps** (Material minimum). Visible glyphs can be smaller than the hit
   area. Trim handles and the playhead have 48 dp hit areas around a 2–4 dp line.
5. **One task per screen.** The phone shows the Program monitor plus Timeline plus a contextual
   toolbar; everything else is a sheet over that screen, never a permanent panel.
6. **Precision through zoom, not through pixels.** Frame-accurate work uses a centred playhead,
   pinch zoom and snapping, not smaller handles.
7. **Same app, three size classes.** Phone portrait gets the new shell; large tablets keep the
   docked layout with touch-sized controls.

## Size classes

Measured in egui points (1 pt = 1 dp on Android) from the window width, so they also apply to a
resized desktop window (preview the phone UI with `filmcraft --touch --window-size 393x852`).

| Class | Width | Shell | Typical device |
|---|---|---|---|
| Compact | < 600 pt | phone shell (this page), portrait | phone portrait |
| Medium | 600 – 839 pt | phone shell, landscape variant: monitor left, timeline right, toolbar as a side rail | phone landscape, small tablet |
| Expanded | ≥ 840 pt | desktop dock with touch sizing (48 dp targets, bigger handles, sheets for dialogs) | tablet, foldable, Chromebook |

`UiState.layout` (`auto` / `desktop` / `touch`) overrides the class; the Android host sets `touch`.

## Navigation map

```
                ┌──────────────────────────── Menu sheet (⋮) ───────────────────────────┐
                │ search all commands · File Edit Clip Sequence Markers Graphics View … │
                └───────────────────────────────────────────────────────────────────────┘
   Bottom bar          Media                  Edit                    Export
   (= Mode)       Project panel +        Program monitor           Export mode
                  Import cards           Timeline                  (Quick Export first,
                  Media Browser          Contextual toolbar         settings collapsed)
                                         Sheets: Effect Controls, Effects, Lumetri,
                                         Audio mixers, Essential Graphics/Sound, Text,
                                         Markers, History, Info, Scopes, Source…
```

The three bottom tabs are the three existing header modes (`Mode::Import`, `Edit`, `Export`;
automation ids `m.nav.media|edit|export`). Home is the Media tab with the project list and the
demo-project card.

## Screens

### Edit (the home of the app)

Portrait, top to bottom:

| Zone | Height | Content | Maps to |
|---|---|---|---|
| Top bar | 52 dp | ⋮ menu · project name (tap: Project sheet) · Undo · Redo · **Export** | `edit.undo`, `edit.redo`, header Quick Export |
| Program monitor | 16:9 of the width, at most 38 % of the body | picture and the monitor's own transport (play, frame steps, edit-point jumps, timecode, More) | Program panel; transport overflow already exists (UI.6) |
| Timeline | remaining height | tracks, clips, centred playhead, ruler | Timeline panel |
| Contextual toolbar | 64 dp | horizontally scrolling buttons that depend on the selection | below |
| Bottom bar | 56 dp + gesture inset | Media · Edit · Export | `Mode` |

Landscape (Medium): Program monitor on the left (45 %), Timeline on the right, the contextual
toolbar as a vertical rail on the far right edge, the bottom bar becomes a left rail.

### Contextual toolbar

| Selection | Buttons (left to right) | Commands |
|---|---|---|
| Nothing | Import · Title · Captions · Effects · Color · Audio · Marker · Tools | `file.import`; sheets Text / Essential Graphics, Effects, Lumetri, Audio Track Mixer; `markers.add`; Tool popover |
| One clip | **Split** · Delete · Duplicate · Speed · Volume · Effects · Transform · More | `sequence.addEdit`, `edit.rippleDelete`, `edit.duplicate`, `clip.speedDuration`, `clip.audioGain`, sheet Effect Controls, Program-monitor transform handles, Clip menu |
| Several clips | Group · Link · Delete · Duplicate · Nest · More | `clip.group`, `clip.link`, `edit.rippleDelete`, `edit.duplicate`, `clip.nest` |
| Caption / graphic | Edit text · Style · Duration · Delete | Text / Properties sheets |
| Transition | Duration · Alignment · Replace · Delete | Effect Controls transition settings |

Split at the playhead replaces the Razor tool for touch; the centred playhead makes the cut point
explicit.

### Timeline interaction

| Gesture | Action | Desktop equivalent |
|---|---|---|
| Drag on empty timeline / ruler | scroll time; the playhead stays centred (**Centered playhead**, default on; `timeline.follow`) | playhead drag, scroll wheel |
| Pinch (two fingers) | horizontal time zoom about the playhead | `view.zoomIn/Out`, Alt+wheel |
| Two-finger vertical drag | scroll tracks | wheel |
| Tap clip | select (shows trim handles, shows the contextual toolbar) | click |
| Tap empty space | deselect | click empty (#683) |
| Long press clip | pick up: haptic tick, then drag to move across time and tracks, snapping on | drag |
| Drag a trim handle (48 dp zone) | trim; edge snaps to the playhead and neighbours | trim |
| Double tap clip | open in Source monitor (sheet) | double click |
| Long press empty / track header | context menu | right click |
| Tap track header | toggle track target; icons for mute / solo / lock open a track sheet | click |
| Tap marker | marker sheet (name, colour, duration, delete) | double click marker |

Snapping is on by default and shown with a haptic tick and a highlighted guide line. Trim mode
(ripple, roll, slip, slide) is chosen in the Tool popover; the toolbar shows the active tool as a
chip.

### Program monitor

Tap: show / hide the monitor controls. Double tap: Fit ↔ 100 %. Pinch and two-finger drag: zoom
and pan (the monitor already zooms with a pinch). With a graphic or clip selected, transform
handles appear with 48 dp grips (G8 / #639 in [ui-parity.md](ui-parity.md)); until those land,
Transform opens the Effect Controls sheet. Full screen: rotate to landscape or the expand button.

### Sheets (replace docked panels)

A sheet is a panel body in a bottom drawer: a drag handle, a title, a close button, and three
snap heights (peek 30 %, half 60 %, full). Swipe down to dismiss, tap the scrim to dismiss. The
Program monitor and playhead stay visible at peek and half height so adjustments show live.

| Sheet | Panel (`PanelKind`) | Opened from |
|---|---|---|
| Effect Controls | `EffectControls` | Effects / Transform buttons on a selected clip |
| Effects | `Effects` | Effects button |
| Color | `LumetriColor`, `LumetriScopes` tab | Color button |
| Audio | `AudioTrackMixer`, `AudioClipMixer`, `EssentialSound`, `AudioMeters` tabs | Audio button |
| Titles | `EssentialGraphics`, `Text` (Transcript / Captions / Graphics tabs), `Properties` | Title / Captions buttons |
| Source | `Source` | double tap clip, Project item ▸ Open |
| Markers · History · Info · Notes · Metadata · Events · Progress | same names | Menu sheet ▸ Window |

A sheet with several panels shows them as a segmented control at the top (the dock tab strip,
touch-sized). Workspaces map onto this: Editing = no sheet, Color = Color sheet at half height,
Audio = Audio sheet, Effects = Effects sheet, Captions and Graphics = Titles sheet,
Assembly = Media tab. `window.workspace.*` commands keep working and open the matching sheet.

### Menu sheet (the nine menus)

Opened by ⋮ (top bar) or by the Search button. A full-height sheet with:

1. **Command search** at the top: matches the label (translated or English) and the menu path of
   every entry returned by `menus::menu_items`, i.e. the same list as `ui.menu.list`. This is the
   command palette; any of the ~800 commands is two taps away.
2. **Recent and suggested**: last five commands, plus commands enabled for the current selection.
3. **The nine menus** (File, Edit, Clip, Sequence, Markers, Graphics and Titles, View, Window,
   Help) as expandable groups, drilling into submenus as a push stack with a breadcrumb. Rows are
   56 dp, disabled items stay visible but dimmed with the reason in the status line, checkable
   items show a switch. The shortcut column is hidden unless a hardware keyboard is attached.

The macOS-only native menu hooks are not used. `Settings` is a row in File (as on desktop) and
opens the settings pages as full-screen pages with a back arrow.

### Media (Import / Project)

Project panel as a grid of poster frames (the Icon view), a search field, bins as a breadcrumb,
a floating **+** action button (Import from device, Camera roll picker, Demo footage, Bars and
Tone, New bin, New sequence). Tap an item: preview in the Source sheet with Mark In / Out and
**Add to timeline** (insert at playhead). Long press: multi-select mode with a contextual app bar
(Add, Proxy, Rename, Delete). Media Browser is the entry "Browse device storage".

### Export

Quick Export first: preset cards (Share – H.264 1080p, High quality, Audio only, Frame still),
destination, Export. "Advanced" expands the full Export mode (video, audio, captions, effects,
metadata, queue). The export runs as a foreground service with a notification, survives the
screen turning off and reports progress in the status chip.

### Dialogs

The 37 `egui::Window` dialogs become bottom sheets (short forms) or full-screen pages (long
forms: Sequence Settings, Project Settings, Settings) in the touch shell, through one wrapper so
the call sites change once. Text fields raise the Android soft keyboard and the sheet lifts above
it. Destructive dialogs keep an explicit confirm button; Cancel is the system Back gesture.

### Keyboard-only commands on touch

Each documented keyboard-only group gets a touch affordance; everything else is in the palette.

| Group ([keyboard.md](keyboard.md)) | Touch affordance |
|---|---|
| J K L shuttle, Space | Play button; long press Play for the speed chip (½×, 1×, 2×, 4×, reverse) |
| Frame step, edit-point navigation | monitor transport buttons; swipe on the timecode readout to jog |
| I / O marks, Go to In / Out | Source sheet buttons; Mark In / Out in the clip long-press menu |
| Add marker (M) | Marker button in the toolbar |
| Select clip at playhead / next / previous | tap in the timeline; Prev / Next buttons in the More row |
| Nudge, slip, slide | Tool popover + drag; ± frame buttons in the Transform row |
| Target and source-patch toggles | tap the track header |
| Zoom to sequence, track heights | pinch; Fit button; track sheet |
| Frame maximize, panel cycling | sheet snap heights; rotate to landscape |
| Text size / leading / alignment | Properties sheet controls |

## Visual design

Same tokens and themes as the desktop ([theme.rs](../crates/ui-egui/src/theme.rs): Darkest, Dark,
Light, Sync with system), scaled for touch in the compact shell: text 15 pt body / 13 pt minimum,
`interact_size` 48 dp, item spacing 8 dp, corner radius 12 dp on sheets, floating scroll bars.
Icons are the existing code-drawn set (original work, [ATTRIBUTION.md](../ATTRIBUTION.md)); new
icons are drawn from scratch, never taken from other editors' artwork ([AGENTS.md](../AGENTS.md) §1).
Safe-area insets (status bar, gesture bar, camera cut-out) are applied by the host and pad the
shell. Colour is never the only state indicator.

## Performance rules that shape the UX

- Phone default playback resolution is ½ (Settings ▸ Playback), paused resolution Full.
- Imports create proxies automatically on devices with < 8 GB RAM (the proxy workflow exists,
  M11); the Media tab shows a proxy badge.
- The UI runs at the display's refresh rate only while something changes (egui repaints on
  demand), to protect battery and thermals; a thermal warning lowers playback resolution.
- Hardware decode through MediaCodec (`crates/platform`) is the target; until it lands, only our
  decoders run and the app warns on clips above 1080p30.

## Out of scope for the first release

Multicam live switching on phone, Trim Monitor dual view, Essential Sound Dialogue chain
authoring, scopes beyond Waveform, AAF / OMF export, plugin hosting. They stay reachable through
the palette on tablets and are tracked in [mobile-roadmap.md](mobile-roadmap.md).
