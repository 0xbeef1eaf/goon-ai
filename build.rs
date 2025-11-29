use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    match target_os.as_str() {
        "windows" => handle_windows(&out_dir),
        "linux" => handle_linux(),
        "macos" => handle_macos(),
        _ => println!(
            "cargo:warning=Unsupported OS for automatic mpv setup: {}",
            target_os
        ),
    }
}

fn handle_windows(out_dir: &Path) {
    let mpv_url =
        "https://nightly.link/mpv-player/mpv/workflows/build/master/mpv-dev-x86_64-v3.zip";
    let archive_path = out_dir.join("mpv-dev.zip");

    // Check if already downloaded/extracted
    if out_dir.join("mpv-2.dll").exists() {
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        return;
    }

    println!("cargo:warning=Downloading libmpv for Windows...");

    // Download
    // We use a match to handle potential errors gracefully-ish, though panic in build.rs stops build.
    let response = match reqwest::blocking::get(mpv_url) {
        Ok(r) => r,
        Err(e) => {
            println!("cargo:warning=Failed to download libmpv: {}", e);
            return;
        }
    };

    let mut file = fs::File::create(&archive_path).expect("Failed to create archive file");
    let bytes = response.bytes().expect("Failed to get bytes");
    let mut content = Cursor::new(bytes);
    std::io::copy(&mut content, &mut file).expect("Failed to write archive");

    println!("cargo:warning=Extracting libmpv...");

    // Extract
    let file = fs::File::open(&archive_path).expect("Failed to open archive");
    let mut archive = zip::ZipArchive::new(file).expect("Failed to open zip archive");
    archive
        .extract(out_dir)
        .expect("Failed to extract zip archive");

    // Link
    println!("cargo:rustc-link-search=native={}", out_dir.display());
}

fn handle_linux() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // Use master branch to ensure compatibility with latest FFmpeg (7.1+)
    let _mpv_version = "master";
    let mpv_url = "https://github.com/mpv-player/mpv/archive/refs/heads/master.tar.gz";
    let archive_path = out_dir.join("mpv-master.tar.gz");
    let source_dir = out_dir.join("mpv-master");

    // Check if library already exists
    if out_dir.join("libmpv.so").exists() {
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=mpv");
        return;
    }

    println!("cargo:warning=Downloading mpv source...");
    let response = match reqwest::blocking::get(mpv_url) {
        Ok(r) => r,
        Err(e) => {
            println!("cargo:warning=Failed to download mpv source: {}", e);
            return;
        }
    };

    let mut file = fs::File::create(&archive_path).expect("Failed to create archive file");
    let mut content = Cursor::new(response.bytes().expect("Failed to get bytes"));
    std::io::copy(&mut content, &mut file).expect("Failed to write archive");

    println!("cargo:warning=Extracting mpv source...");
    let tar_gz = fs::File::open(&archive_path).expect("Failed to open archive");
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(&out_dir).expect("Failed to unpack archive");

    println!("cargo:warning=Building mpv (this may take a while)...");

    // Configure with Meson
    let output = std::process::Command::new("meson")
        .arg("setup")
        .arg("build")
        .arg("-Dlibmpv=true")
        .arg("-Dcplayer=false")
        .arg("-Dbuild-date=false")
        .current_dir(&source_dir)
        .output()
        .expect("Failed to run meson setup");

    if !output.status.success() {
        println!("cargo:warning=meson setup failed:");
        println!("cargo:warning={}", String::from_utf8_lossy(&output.stdout));
        println!("cargo:warning={}", String::from_utf8_lossy(&output.stderr));
        return;
    }

    // Build with Ninja
    let output = std::process::Command::new("meson")
        .arg("compile")
        .arg("-C")
        .arg("build")
        .current_dir(&source_dir)
        .output()
        .expect("Failed to run meson compile");

    if !output.status.success() {
        println!("cargo:warning=meson compile failed:");
        println!("cargo:warning={}", String::from_utf8_lossy(&output.stdout));
        println!("cargo:warning={}", String::from_utf8_lossy(&output.stderr));
        return;
    }

    // Copy library
    // The library is usually in build/libmpv.so
    let built_lib = source_dir.join("build/libmpv.so");
    if built_lib.exists() {
        fs::copy(&built_lib, out_dir.join("libmpv.so")).expect("Failed to copy libmpv.so");
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=mpv");
    } else {
        println!("cargo:warning=Could not find built libmpv.so");
    }
}

fn handle_macos() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mpv_url = "https://nightly.link/mpv-player/mpv/workflows/build/master/libmpv-macos.zip";
    let archive_path = out_dir.join("libmpv-macos.zip");

    // Check if library already exists
    if out_dir.join("libmpv.dylib").exists() {
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=mpv");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
        return;
    }

    println!("cargo:warning=Downloading libmpv for macOS...");
    let response = match reqwest::blocking::get(mpv_url) {
        Ok(r) => r,
        Err(e) => {
            println!("cargo:warning=Failed to download libmpv: {}", e);
            return;
        }
    };

    let mut file = fs::File::create(&archive_path).expect("Failed to create archive file");
    let mut content = Cursor::new(response.bytes().expect("Failed to get bytes"));
    std::io::copy(&mut content, &mut file).expect("Failed to write archive");

    println!("cargo:warning=Extracting libmpv...");
    let file = fs::File::open(&archive_path).expect("Failed to open archive");
    let mut archive = zip::ZipArchive::new(file).expect("Failed to open zip archive");

    // Extract to a subdirectory first to avoid clutter or just extract directly
    // The zip likely contains libmpv.dylib directly or in a folder.
    // Let's extract to out_dir.
    archive
        .extract(&out_dir)
        .expect("Failed to extract zip archive");

    // Check if we need to move files or if it's in a subdir
    // Assuming the zip contains libmpv.dylib at root or we find it.
    // If it's in a folder, we might need to find it.
    // But for now let's assume it extracts to out_dir/libmpv.dylib or similar.

    // If the dylib is not in out_dir, try to find it in subdirs
    let dylib_path = out_dir.join("libmpv.dylib");
    if !dylib_path.exists() {
        // Try to find it
        for entry in walkdir::WalkDir::new(&out_dir) {
            let entry = entry.expect("Failed to read directory entry");
            if entry.file_name() == "libmpv.dylib" {
                std::fs::copy(entry.path(), &dylib_path).expect("Failed to move libmpv.dylib");
                break;
            }
        }
    }

    if dylib_path.exists() {
        // Fix install name to be relative
        let _ = std::process::Command::new("install_name_tool")
            .arg("-id")
            .arg("@rpath/libmpv.dylib")
            .arg(&dylib_path)
            .status();

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=mpv");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    } else {
        println!("cargo:warning=Could not find extracted libmpv.dylib");
    }
}
