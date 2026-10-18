# filmcraft-android

FilmCraft on Android: the engine and the touch UI (`filmcraft-ui-egui`, `LayoutMode::Touch`) in a
`NativeActivity`, started by eframe on winit's Android backend and wgpu (Vulkan).

Build, install and run with `cargo xtask android …`; toolchain, architecture and status:
[docs/mobile.md](../../docs/mobile.md), design: [docs/mobile-ux.md](../../docs/mobile-ux.md),
checklist and progress: [docs/mobile-roadmap.md](../../docs/mobile-roadmap.md).

The crate is empty on every target but Android. `res/` holds the launcher icons (rendered from
`assets/app-icon/`, each with an attribution sidecar).
