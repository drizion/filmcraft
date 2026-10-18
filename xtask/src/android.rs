//! `cargo xtask android <doctor|check|build|install|run|logs>`: the Android APK (docs/mobile.md).
//!
//! The APK is built by [cargo-apk](https://github.com/rust-mobile/cargo-apk) from the
//! `[package.metadata.android]` table of `apps/filmcraft-android/Cargo.toml`. This file only checks
//! the toolchain, picks the target and release/debug profile, copies the result to a stable path
//! (`<target>/android/FilmCraft-<version>-<abi>.apk`) and drives `adb`.

use std::path::{Path, PathBuf};
use std::process::Command;

const PACKAGE: &str = "filmcraft-android";
const APP_ID: &str = "ai.storyteller.filmcraft";
const APK_NAME: &str = "FilmCraft";
const PLATFORM: u32 = 34;

/// Rust target and Android ABI name for a phone (arm64) or the x86_64 emulator.
fn target(emulator: bool) -> (&'static str, &'static str) {
    if emulator { ("x86_64-linux-android", "x86_64") } else { ("aarch64-linux-android", "arm64-v8a") }
}

fn find_in_path(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|p| std::env::split_paths(&p).map(|d| d.join(bin)).find(|f| f.is_file()))
}

fn env_dir(names: &[&str]) -> Option<PathBuf> {
    names.iter().filter_map(std::env::var_os).map(PathBuf::from).find(|p| p.is_dir())
}

fn sdk() -> Option<PathBuf> {
    env_dir(&["ANDROID_HOME", "ANDROID_SDK_ROOT"])
}

fn ndk() -> Option<PathBuf> {
    if let Some(d) = env_dir(&["ANDROID_NDK_ROOT", "ANDROID_NDK_HOME", "NDK_HOME"]) {
        return Some(d);
    }
    // the newest side-by-side NDK in the SDK
    let mut versions: Vec<PathBuf> = std::fs::read_dir(sdk()?.join("ndk")).ok()?.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_dir()).collect();
    versions.sort();
    versions.pop()
}

fn adb() -> Option<PathBuf> {
    find_in_path("adb").or_else(|| sdk().map(|s| s.join("platform-tools").join("adb")).filter(|p| p.is_file()))
}

fn installed_rust_targets() -> Vec<String> {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_string()).collect())
        .unwrap_or_default()
}

/// One line of the doctor report: a check, what was found and how to fix it.
struct Check {
    name: &'static str,
    found: Option<String>,
    fix: String,
}

fn checks(emulator: bool) -> Vec<Check> {
    let (rust_target, _) = target(emulator);
    let targets = installed_rust_targets();
    let platform = sdk().map(|s| s.join("platforms").join(format!("android-{PLATFORM}"))).filter(|p| p.is_dir());
    let build_tools = sdk().and_then(|s| std::fs::read_dir(s.join("build-tools")).ok()).and_then(|mut d| d.next()).and_then(|e| e.ok()).map(|e| e.path());
    vec![
        Check {
            name: "Rust target",
            found: targets.iter().any(|t| t == rust_target).then(|| rust_target.to_string()),
            fix: format!("rustup target add {rust_target}"),
        },
        Check { name: "cargo-apk", found: find_in_path("cargo-apk").map(|p| p.display().to_string()), fix: "cargo install cargo-apk --locked".into() },
        Check {
            name: "JDK (keytool)",
            found: find_in_path("keytool").map(|p| p.display().to_string()),
            fix: "install a JDK 17+ (for example Temurin) and put it on PATH".into(),
        },
        Check {
            name: "Android SDK",
            found: sdk().map(|p| p.display().to_string()),
            fix: "install Android Studio or the command-line tools and set ANDROID_HOME (Linux: ~/Android/Sdk)".into(),
        },
        Check {
            name: "SDK platform",
            found: platform.map(|p| p.display().to_string()),
            fix: format!("sdkmanager \"platforms;android-{PLATFORM}\" \"build-tools;34.0.0\""),
        },
        Check { name: "SDK build-tools", found: build_tools.map(|p| p.display().to_string()), fix: "sdkmanager \"build-tools;34.0.0\"".into() },
        Check { name: "Android NDK", found: ndk().map(|p| p.display().to_string()), fix: "sdkmanager \"ndk;27.2.12479018\" (or set ANDROID_NDK_ROOT)".into() },
        Check { name: "adb (to install)", found: adb().map(|p| p.display().to_string()), fix: "sdkmanager \"platform-tools\" and put it on PATH".into() },
    ]
}

/// Print the toolchain report; fails when something needed to *build* is missing (adb is only
/// needed to install, so it is reported but does not fail the build).
pub fn doctor(emulator: bool) -> Result<(), String> {
    let mut missing = 0;
    for c in checks(emulator) {
        match &c.found {
            Some(f) => println!("  ok       {:<18} {f}", c.name),
            None if c.name.starts_with("adb") => println!("  optional {:<18} not found: {}", c.name, c.fix),
            None => {
                missing += 1;
                println!("  MISSING  {:<18} {}", c.name, c.fix)
            }
        }
    }
    if missing == 0 { Ok(()) } else { Err(format!("{missing} build tool(s) missing; see docs/mobile.md")) }
}

fn run(cmd: &mut Command) -> Result<(), String> {
    eprintln!("$ {cmd:?}");
    let st = cmd.status().map_err(|e| format!("{cmd:?}: {e}"))?;
    if st.success() { Ok(()) } else { Err(format!("failed: {cmd:?}")) }
}

/// `cargo check` for Android: no SDK or NDK needed for the pure-Rust crates, so it works anywhere
/// the Rust target is installed. Catches Android-only compile errors early.
pub fn check(emulator: bool) -> Result<(), String> {
    let (rust_target, _) = target(emulator);
    run(Command::new(env!("CARGO")).args(["check", "--target", rust_target, "-p", PACKAGE, "-p", "filmcraft-ui-egui"]))
}

/// The Android debug keystore cargo-apk signs local builds with (created by Android Studio too).
fn ensure_debug_keystore() -> Result<(), String> {
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or("HOME is not set")?;
    let store = home.join(".android").join("debug.keystore");
    if store.is_file() {
        return Ok(());
    }
    std::fs::create_dir_all(store.parent().ok_or("no keystore directory")?).map_err(|e| e.to_string())?;
    run(Command::new("keytool")
        .args(["-genkeypair", "-v", "-keystore"])
        .arg(&store)
        .args(["-alias", "androiddebugkey", "-storepass", "android", "-keypass", "android", "-keyalg", "RSA", "-keysize", "2048", "-validity", "10000"])
        .args(["-dname", "CN=Android Debug,O=Android,C=US"]))
}

fn target_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("target"))
}

/// Build the APK and copy it to `<target>/android/FilmCraft-<version>-<abi>.apk`; returns the path.
pub fn build(root: &Path, debug: bool, emulator: bool) -> Result<PathBuf, String> {
    doctor(emulator)?;
    ensure_debug_keystore()?;
    let (rust_target, abi) = target(emulator);
    let mut cmd = Command::new(env!("CARGO"));
    cmd.current_dir(root).args(["apk", "build", "-p", PACKAGE, "--target", rust_target]);
    if !debug {
        cmd.arg("--release");
    }
    run(&mut cmd)?;
    let profile = if debug { "debug" } else { "release" };
    let built = root.join(target_dir()).join(profile).join("apk").join(format!("{APK_NAME}.apk"));
    if !built.is_file() {
        return Err(format!("cargo apk finished but {} does not exist", built.display()));
    }
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).map_err(|e| format!("Cargo.toml: {e}"))?;
    let version = crate::version::read(&manifest)?;
    let out_dir = root.join(target_dir()).join("android");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let out = out_dir.join(format!("{APK_NAME}-{version}-{abi}{}.apk", if debug { "-debug" } else { "" }));
    std::fs::copy(&built, &out).map_err(|e| format!("copying the APK: {e}"))?;
    println!("\nAPK: {}", out.display());
    Ok(out)
}

pub fn install(apk: &Path) -> Result<(), String> {
    let adb = adb().ok_or("adb not found: sdkmanager \"platform-tools\" and put it on PATH (docs/mobile.md)")?;
    run(Command::new(&adb).args(["install", "-r"]).arg(apk))
}

pub fn launch() -> Result<(), String> {
    let adb = adb().ok_or("adb not found")?;
    run(Command::new(&adb).args(["shell", "am", "start", "-n", &format!("{APP_ID}/android.app.NativeActivity")]))
}

/// `adb logcat` filtered to the app (`FilmCraft` is the log tag; crashes show as `RustStdoutStderr`).
pub fn logs() -> Result<(), String> {
    let adb = adb().ok_or("adb not found")?;
    run(Command::new(&adb).args(["logcat", "-s", "FilmCraft:V", "RustStdoutStderr:V", "AndroidRuntime:E", "DEBUG:V"]))
}

pub fn main(root: &Path, args: &[String]) -> Result<(), String> {
    let flag = |f: &str| args.iter().any(|a| a == f);
    let emulator = flag("--emulator");
    match args.first().map(String::as_str) {
        Some("doctor") => doctor(emulator),
        Some("check") => check(emulator),
        Some("build") => build(root, flag("--debug"), emulator).map(|_| ()),
        Some("install") => {
            let apk = build(root, flag("--debug"), emulator)?;
            install(&apk)
        }
        Some("run") => {
            let apk = build(root, flag("--debug"), emulator)?;
            install(&apk)?;
            launch()
        }
        Some("logs") => logs(),
        _ => Err("usage: cargo xtask android <doctor|check|build|install|run|logs> [--debug] [--emulator]".into()),
    }
}
