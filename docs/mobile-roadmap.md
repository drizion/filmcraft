# Mobile roadmap and progress

> **Last updated:** 2026-10-18 · Android first, iOS kept open · design: [mobile-ux.md](mobile-ux.md) ·
> build and install: [mobile.md](mobile.md)

The checklist below is the single source of truth for mobile progress. The table is generated from
it: edit the checkboxes, then run `cargo xtask mobile --write` (`cargo xtask mobile --check` fails
when the table is stale). A phase is *items done ÷ items*; the overall figure weights phases by
their `weight` (the weights add up to 100 and reflect the share of the total effort, not the item count). An item is
checked only when it works and is covered by a test, or, for things that need hardware, when it was
seen working on a device.

<!-- progress:start -->
| Phase | Weight | Done | Progress |
|---|---|---|---|
| M0 Android build and APK | 8 | 9/13 | ██████░░░░ 69% |
| M1 Touch shell and menus | 20 | 13/23 | █████░░░░░ 57% |
| M2 Timeline and monitor by touch | 17 | 0/13 | ░░░░░░░░░░ 0% |
| M3 Media, storage and projects | 14 | 0/9 | ░░░░░░░░░░ 0% |
| M4 Playback, audio and performance | 15 | 0/9 | ░░░░░░░░░░ 0% |
| M5 Panels adapted to touch | 9 | 0/8 | ░░░░░░░░░░ 0% |
| M6 Export and share | 8 | 0/6 | ░░░░░░░░░░ 0% |
| M7 Release and quality | 5 | 0/7 | ░░░░░░░░░░ 0% |
| M8 iOS readiness | 4 | 0/6 | ░░░░░░░░░░ 0% |
| **Overall (weighted)** | 100 | 22/94 | **█░░░░░░░░░ 17%** |
<!-- progress:end -->

How to read it: **100 %** of phase M0–M1 means "an APK that installs, boots into the touch
shell and can be demoed"; the product (import your own clips, edit by touch, export) is M2–M5.
Estimates are in Opus 5.5 agent-hours, same calibration as [target-app-parity.md](target-app-parity.md#calibration).

## M0 — Android build and APK · weight 8

Estimate: 8–14 h left, most of it first-run debugging on a real device.

- [x] `apps/filmcraft-android` crate (cdylib, `android_main`, NativeActivity), empty on other targets
- [x] APK metadata for cargo-apk (package id, SDK 28–34, permissions, launcher icon, debug signing)
- [x] Launcher icons from the existing app icon, with attribution sidecars
- [x] `cargo xtask android doctor | check | build | install | run | logs`
- [x] `accesskit` kept off Android so the NativeActivity build compiles
- [x] ADR 0002 for the `android_main` entry point (proposed; needs owner approval)
- [x] GitHub Actions workflow builds the APK and uploads it as an artifact
- [x] Manifest metadata accepted by cargo-apk (dry run with a stub SDK)
- [x] `cargo xtask android check` green (engine, touch UI and app compile for `aarch64-linux-android`)
- [ ] CI run produced an APK (first green `Android` workflow run)
- [ ] APK installs and starts on a real phone (needs a device)
- [ ] Release signing with an owner-held key and an `.aab` for Play (M7)
- [ ] Control channel on the device (`adb forward`, debug builds only)

## M1 — Touch shell and menus · weight 20

Estimate: 40–70 h. This is the redesign described in [mobile-ux.md](mobile-ux.md).

- [x] Layout modes (`auto` / `desktop` / `touch`) and Material size classes, saved UI state, control-channel `ui.set`
- [x] Top bar: menu, project name, Undo, Redo, Export
- [x] Bottom tabs = the three header modes (Media, Edit, Export)
- [x] Edit screen, portrait: Program monitor over Timeline over the toolbar
- [x] Edit screen, landscape: monitor left, timeline right
- [x] Contextual toolbar that follows the selection (nothing / one clip / several clips)
- [x] Bottom sheets with snap heights and panel tabs, drag to resize or dismiss
- [x] Menu sheet: command search over every menu entry, the nine menus as drill-down, recent commands
- [x] Tool sheet (Selection, Track Select, Ripple, Roll, Rate Stretch, Razor, Slip, Slide, Pen, Hand, Zoom, Type)
- [x] Desktop workspaces open their sheet (`window.workspace.*` keeps working)
- [x] Media tab: Project panel with an Import button
- [x] Phone preview on the desktop: `filmcraft --touch --window-size 393x852`
- [x] Headless tests of the shell (`tests/mobile_ui.rs`) and of the pure logic
- [ ] Safe-area insets from the Android host (status bar, gesture bar, cut-out)
- [ ] Soft keyboard: sheet lifts above it, focus rules
- [ ] 48 dp style for egui widgets inside panels (spacing, interact size, fonts)
- [ ] Dialogs (37 windows) as bottom sheets or full-screen pages through one wrapper
- [ ] Settings as full-screen pages with a back arrow
- [ ] Landscape toolbar as a side rail; bottom bar as a left rail
- [ ] Tablet (Expanded) layout: docked panels with touch-sized targets
- [ ] Context menus on long press verified in every panel
- [ ] Pt-BR and Spanish strings for the shell reviewed by a native speaker
- [ ] Hardware keyboard on tablets: shortcuts shown and working

## M2 — Timeline and monitor by touch · weight 17

Estimate: 50–90 h. The timeline is 3,400 lines of pointer-first code.

- [ ] Centred playhead: dragging the timeline scrolls time under it
- [ ] Pinch zoom (time) and two-finger vertical scroll (tracks)
- [ ] Tap to select, tap empty space to deselect, selection feedback
- [ ] Long press picks up a clip (haptic), drag moves it across time and tracks
- [ ] 48 dp trim handles with snapping to playhead and neighbours, haptic tick
- [ ] Split at playhead, delete, duplicate from the toolbar verified on touch
- [ ] Track header sheet (target, mute, solo, lock, height)
- [ ] Marker sheet, tap a marker to edit
- [ ] Double tap a clip to open it in the Source sheet with Mark In / Out and Add to timeline
- [ ] Program monitor: tap to toggle controls, double tap fit/100 %, pinch zoom, two-finger pan
- [ ] Transform handles with 48 dp grips on the monitor (desktop issue #639)
- [ ] Jog by dragging the timecode, speed chip on long press of Play
- [ ] Ripple, roll, slip, slide and rate stretch usable by touch

## M3 — Media, storage and projects · weight 14

Estimate: 40–80 h. Needs JNI wrappers; raw FFI stays out of the app crate (ADR 0002).

- [ ] Storage Access Framework picker for media (`HostHooks::pick_files`)
- [ ] Android Photo Picker for camera-roll video and photos
- [ ] Read `content://` media without copying (file descriptors through `ByteReader`)
- [ ] Open and save `.fcproj` through the document picker; recent projects list on the Media tab
- [ ] Projects list as the home of the Media tab (new, open, demo, duplicate, delete)
- [ ] Save and auto-save on `onPause`; recover after the system kills the app
- [ ] Automatic proxies for imports on low-memory devices
- [ ] Share sheet: send an export to other apps
- [ ] Media permissions flow (Android 13+ granular, ≤ 12 legacy)

## M4 — Playback, audio and performance · weight 15

Estimate: 60–110 h.

- [ ] Audio output through AAudio/Oboe (`AudioOut`), audio clock as playback master
- [ ] Vulkan compositor verified on Mali, Adreno and PowerVR; GLES fallback
- [ ] Hardware video decode through MediaCodec in `crates/platform` (ADR 0001)
- [ ] Phone defaults: playback resolution ½, paused Full, GPU effects on
- [ ] NEON-optimised hot loops checked (decode, scale, colour) and benchmarked on a mid-range phone
- [ ] Thermal and battery handling: lower resolution when throttled
- [ ] Lifecycle: surface loss and recreation, rotation, split screen without losing state
- [ ] Memory budget for 8 GB and 4 GB devices (frame cache, proxies)
- [ ] 1080p30 H.264 playback with 0 dropped frames on a reference mid-range phone

## M5 — Panels adapted to touch · weight 9

Estimate: 40–80 h.

- [ ] Project panel: poster grid, bin breadcrumb, multi-select by long press
- [ ] Effect Controls: touch sliders and steppers instead of hot-text drag, keyframe by tap
- [ ] Lumetri Color with large sliders, wheels sized for fingers
- [ ] Audio mixers: faders and pan with 48 dp grips
- [ ] Text, captions and Essential Graphics: on-screen text editing with the soft keyboard
- [ ] Effects panel: search first, tap to apply, long press to preview
- [ ] History, Markers, Info, Metadata as sheets
- [ ] Speech-to-text and Text to Speech models: download management on mobile data

## M6 — Export and share · weight 8

Estimate: 25–50 h.

- [ ] Quick Export as preset cards (Share, High quality, Audio only, Still frame)
- [ ] Export to Movies via MediaStore, plus the share sheet
- [ ] Foreground service and notification so exports survive the screen turning off
- [ ] Hardware encode (MediaCodec) with the software encoder as fallback
- [ ] Progress chip and cancel from the top bar and the notification
- [ ] Export queue by touch (reorder, retry)

## M7 — Release and quality · weight 5

Estimate: 25–45 h.

- [ ] Release signing, `.aab`, Play listing assets (original artwork only, AGENTS.md §1)
- [ ] Device matrix: 3 phones and 1 tablet, Android 9 to 15
- [ ] Crash and ANR reporting that respects privacy (opt-in), logs shareable from Settings
- [ ] TalkBack: GameActivity build with `accesskit`
- [ ] Localisation of the shell in every language the interface ships
- [ ] Battery and storage footprint documented; APK size budget
- [ ] Screenshots of the touch UI in the repo (FilmCraft only, original media)

## M8 — iOS readiness · weight 4

Estimate: 40–80 h, needs a Mac and an Apple developer account.

- [ ] Audit: no Android-only code outside `apps/filmcraft-android`; hooks cover every platform need
- [ ] `apps/filmcraft-ios` spike: winit iOS backend, wgpu/Metal, boots to the touch shell
- [ ] Document picker and Photos picker hooks
- [ ] VideoToolbox decode through the existing `crates/platform`
- [ ] AVAudioEngine output
- [ ] TestFlight build documented in `docs/mobile.md`

## Where the first APK stands

An APK from the current branch boots into the touch shell with the demo project, lets you switch
tabs, open the command menu, open sheets for Effects / Color / Audio / Titles, pick tools and edit
with the existing timeline gestures. It cannot import your own clips or play sound yet (M3, M4).
The author could not run it on a device from the cloud sandbox used to write it (the Android SDK
host is not reachable there), which is why "installs and starts on a real phone" is unchecked.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-18 | major | Created: phases M0–M8, weights, generated progress table, first slice of M0 and M1 |
