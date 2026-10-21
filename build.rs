use std::{env, error::Error, fs, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icons/keyscribe.ico");
    println!("cargo:rerun-if-changed=packaging/windows/keyscribe.rc");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }
    if !env::var("HOST")?.contains("-windows-") {
        println!("cargo:warning=Windows icon resource compilation is skipped on this host; build the final EXE on Windows with MSVC and Windows SDK.");
        return Ok(());
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        return Err("Windows packaging requires the x86_64-pc-windows-msvc target".into());
    }
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("Missing manifest path")?);
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("Missing build output path")?);
    // Keep the resource filename independent of spaces and non-ASCII characters
    // in the checkout path. RC resolves this filename in its working directory.
    fs::copy(
        root.join("assets/icons/keyscribe.ico"),
        out.join("keyscribe.ico"),
    )?;
    let status = Command::new("rc.exe")
        .current_dir(&out)
        .args(["/nologo", "/fo", "keyscribe.res"])
        .arg(root.join("packaging/windows/keyscribe.rc"))
        .status()
        .map_err(|error| format!("Cannot run Windows SDK rc.exe: {error}. Build from Developer PowerShell for Visual Studio with Windows SDK installed."))?;
    if !status.success() {
        return Err(format!("Windows icon resource compiler failed: {status}").into());
    }
    println!(
        "cargo:rustc-link-arg-bin=keyscribe={}",
        out.join("keyscribe.res").display()
    );
    Ok(())
}
