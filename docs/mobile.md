# FilmCraft on mobile

> **Last updated:** 2026-10-18 · Android first, iOS kept possible · progress and checklist:
> [mobile-roadmap.md](mobile-roadmap.md) · screens and touch UX: [mobile-ux.md](mobile-ux.md)

This fork runs the same engine and egui UI as the desktop app on a phone. Android comes first;
nothing in the design stops an iOS build later (see [iOS](#ios)).

## What exists

| Piece | Where | State |
|---|---|---|
| Touch shell (top bar, Media / Edit / Export tabs, contextual toolbar, sheets, command menu) | `crates/ui-egui/src/mobile/` | first slice, tested headless |
| Android app (NativeActivity, eframe + wgpu/Vulkan) | `apps/filmcraft-android/` | builds an APK; not yet run on a device by the author |
| Build, install, run, logs | `cargo xtask android …` | scripted |
| APK from CI | `.github/workflows/android.yml` | artifact `FilmCraft-apk` on every push to `main`, `claude/**`, `mobile/**` |
| Preview the phone UI on the desktop | `filmcraft --touch --window-size 393x852` | works today |

Everything else (file picker, audio, hardware decode, touch timeline gestures) is on the
checklist in [mobile-roadmap.md](mobile-roadmap.md).

## Get an APK to test

### Option A: from GitHub Actions (no Android tooling on your machine)

Push the branch. The **Android** workflow builds the APK on a GitHub runner (it ships the SDK and
NDK). Open the run, then **Artifacts ▸ FilmCraft-apk**, download and unzip, copy
`FilmCraft-<version>-arm64-v8a.apk` to the phone (or `adb install -r` it). Allow "Install unknown
apps" for the file manager you use. The APK is signed with the Android debug key, so installing a
newer build over an older one works as long as both came from this workflow *or* the same machine;
uninstall first when switching between a CI build and a local build.

### Option B: build locally

One-time setup (Linux, macOS, Windows):

1. Rust 1.95+ and the Android target: `rustup target add aarch64-linux-android`
   (add `x86_64-linux-android` for the emulator).
2. A JDK 17+ on `PATH` (`keytool` creates the debug key).
3. The Android SDK and NDK. Easiest: install Android Studio, then SDK Manager ▸ install *Android
   SDK Platform 34*, *Build-Tools 34*, *NDK (Side by side)* and *Platform-Tools*. Or with the
   command-line tools:
   `sdkmanager "platforms;android-34" "build-tools;34.0.0" "ndk;27.2.12479018" "platform-tools"`.
   Set `ANDROID_HOME` (Linux `~/Android/Sdk`, macOS `~/Library/Android/sdk`, Windows
   `%LOCALAPPDATA%\Android\Sdk`); the newest side-by-side NDK is found automatically, or set
   `ANDROID_NDK_ROOT`.
4. `cargo install cargo-apk --locked`

Then, from the repository root:

```sh
cargo xtask android doctor          # what is installed, what is missing and how to fix it
cargo xtask android check           # fast: compile for Android only (no SDK/NDK needed)
cargo xtask android build           # release APK → target/android/FilmCraft-<version>-arm64-v8a.apk
cargo xtask android install         # build, then adb install -r on the connected phone
cargo xtask android run             # build, install and start the app
cargo xtask android logs            # adb logcat for the app (tag FilmCraft, crashes)
```

Flags: `--debug` (unoptimised, much slower on the phone: use only to debug), `--emulator`
(x86_64 for the Android emulator). The first build compiles ~50 crates for Android and takes
several minutes; later builds are incremental.

Phone setup: Settings ▸ About phone ▸ tap *Build number* seven times ▸ Developer options ▸ enable
*USB debugging*, plug in, accept the key prompt, `adb devices` must list it. Wireless: *Wireless
debugging* ▸ `adb pair` / `adb connect`.

### The incremental loop

1. Change UI code in `crates/ui-egui/src/mobile/` (or any crate).
2. **Preview on the desktop first**: `cargo run -p filmcraft -- --touch --window-size 393x852`
   (add `--control 9876` and take `ui.screenshot` as in [agents.md](agents.md)). Rotate the preview
   with `--window-size 852x393`.
3. `cargo test -p filmcraft-ui-egui --test mobile_ui` for the headless touch-UI tests.
4. `cargo xtask android run` to try it on the device; `cargo xtask android logs` if it misbehaves.

## Architecture

```
        android_main (apps/filmcraft-android)         ← NativeActivity, logging, data dir
              │ eframe::run_native (winit android backend, wgpu → Vulkan)
              ▼
        FilmcraftApp (crates/ui-egui)  ui.layout = Touch
              │  mobile::show  → top bar / tabs / toolbar / sheets / command menu
              │  panels::show  → the same panel bodies as the desktop
              ▼
        Session (crates/engine) → project, edit, render, media, export …   unchanged
```

- **Layer rules hold.** Nothing below L5 knows about Android. The host (`filmcraft-android`) is
  an app crate like `filmcraft` and `filmcraft-web`; host services (file pickers, audio, clipboard,
  open URL, system theme) are the existing `HostHooks` / `AudioOut` traits, implemented per
  platform. iOS would add a sibling app crate and implement the same hooks.
- **`NativeActivity`, no Java or Kotlin.** `cargo-apk` packages the APK; no Gradle project is
  needed. The cost is that `accesskit` (screen-reader support) is unavailable with
  NativeActivity, so `filmcraft-ui-egui` enables it only off Android. TalkBack support means
  moving to GameActivity (a Gradle build) and is on the checklist.
- **Entry point.** The exported `android_main` is the only item that needs `#[allow(unsafe_code)]`
  ([ADR 0002](adr/0002-android-entry-point.md), awaiting the owner's approval). The never-crash
  rules ([AGENTS.md](../AGENTS.md) §0), the panic hook and the UI `catch_unwind` run on Android
  like on the desktop.
- **Data.** Projects, auto-save, recovery journal and logs live in the app's internal storage
  (`AndroidApp::internal_data_path`). Until the Storage Access Framework picker lands, the app
  opens the demo project (procedurally generated media).
- **Permissions** (`apps/filmcraft-android/Cargo.toml`): `INTERNET` (control channel, model
  downloads), `READ_MEDIA_VIDEO|AUDIO|IMAGES` (Android 13+) and `READ_EXTERNAL_STORAGE` (≤ 12) for
  media, `RECORD_AUDIO` for voice-over. Min SDK 28 (Android 9), target SDK 34.

## Known limits of the first APK

No audio output, no file import from the device (demo footage only), no hardware video decode,
timeline gestures are the desktop's (tap and drag work, pinch zoom and long-press pick-up are
planned), dialogs are floating windows rather than sheets, no soft-keyboard handling beyond
egui's. Each is an item in [mobile-roadmap.md](mobile-roadmap.md).

## iOS

Kept open on purpose: the shell is a layout mode, not Android code; platform services are hooks
behind traits; the app crate is thin. The path is a new `apps/filmcraft-ios` crate on winit's iOS
backend (egui-winit already reads the iOS safe area), wgpu on Metal, VideoToolbox through the
existing `crates/platform` decoder, a document picker for import, and an Xcode wrapper
(`cargo-xcode` or a small generated project). It needs a Mac to build and a developer account to
install; it is milestone M8 in the roadmap and no code depends on it yet.

## Control channel on the device

Agents drive the phone UI exactly like the desktop one ([control-protocol.md](control-protocol.md)),
including `ui.set {"layout": "touch", "mobile": {…}}`, `ui.click {"id": "m.nav.export"}` and
`ui.screenshot`. Exposing the TCP control server on the device (`adb forward tcp:9876 tcp:9876`)
is roadmap item M0 "control channel on device".
