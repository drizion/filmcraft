# ADR 0002: the exported `android_main` entry point in `apps/filmcraft-android`

- **Status:** proposed (2026-10-18, mobile fork; needs the project owner's approval because it
  widens [AGENTS.md](../../AGENTS.md) §0 item 3)
- **Related:** [ADR 0001](0001-platform-ffi.md), [docs/mobile.md](../mobile.md)

## Context

The workspace sets `unsafe_code = "forbid"`. A `NativeActivity` app starts when Android's loader
calls a symbol named `android_main` that the app's shared library exports. In Rust 2024 that is
`#[unsafe(no_mangle)] fn android_main(app: AndroidApp)`, and the `unsafe_code` lint reports
`no_mangle`, so the attribute cannot be written under `forbid`. The same applies to every Android
Rust app (the android-activity and eframe documentation show exactly this function).

## Decision

`apps/filmcraft-android` (an app crate, empty on every other target) does not use
`lints.workspace = true`. Its own `[lints]` table copies the workspace lints except
`unsafe_code = "deny"`, and exactly one item, `android_main`, carries `#[allow(unsafe_code)]`.

Rules:

1. The crate contains no `unsafe` block, no `unsafe fn` and no other `unsafe` attribute. Review and
   `rg 'unsafe' apps/filmcraft-android` must show only that one attribute pair.
2. `android_main` only sets up logging and the panic hook, builds the eframe options and calls
   `eframe::run_native`; all application code stays in safe crates.
3. Anything else that needs the Android runtime (the Storage Access Framework picker, audio,
   MediaCodec) uses safe wrappers from permissively licensed crates (`jni`, `ndk`, `oboe`, …) or,
   if raw FFI is unavoidable, goes in `crates/platform` under ADR 0001, never in the app crate.

## Consequences

The "never crash" guarantee is unchanged: the panic hook and the UI-pass `catch_unwind` run on
Android as on the desktop. The cost is one audited attribute. The alternative, a C `main` shim,
would add a non-Rust source file for no safety gain.
