extern crate core;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

const CSPICE_DIR: &str = "CSPICE_DIR";
const CSPICE_CLANG_TARGET: &str = "CSPICE_CLANG_TARGET";
const CSPICE_CLANG_ROOT: &str = "CSPICE_CLANG_ROOT";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    if std::env::var("DOCS_RS").is_ok() {
        docs_rs(&out_path);
    }

    println!("cargo:rerun-if-env-changed={}", CSPICE_DIR);
    println!("cargo:rerun-if-env-changed={}", CSPICE_CLANG_TARGET);
    println!("cargo:rerun-if-env-changed={}", CSPICE_CLANG_ROOT);

    let cspice_dir = env::var(CSPICE_DIR)
        .ok()
        .map(PathBuf::from)
        .or_else(locate_cspice);

    #[cfg(feature = "downloadcspice")]
    let cspice_dir = cspice_dir.or_else(|| {
        let downloaded = out_path.join("cspice");
        if !downloaded.exists() {
            download_cspice(&out_path);
        }
        Some(downloaded)
    });

    let cspice_dir =
		cspice_dir.expect("Cannot build: CSPICE_DIR environment variable was not provided, no CSPICE install was found, and feature \"downloadcspice\" is disabled.");

    if !cspice_dir.is_dir() {
        panic!(
            "Provided {CSPICE_DIR} ({}) is not a directory",
            cspice_dir.display()
        )
    }

    let include_dir = cspice_dir.join("include");

    let mut clang_args = vec![];
    if let Ok(target) = env::var(CSPICE_CLANG_TARGET) {
        if !target.is_empty() {
            clang_args.push(format!("--target={}", target));
        }
    }
    if let Ok(sysroot) = env::var(CSPICE_CLANG_ROOT) {
        if !sysroot.is_empty() {
            clang_args.push(format!("--sysroot={}", sysroot));
        }
    }

    let bindings = bindgen::Builder::default()
        .header(include_dir.join("SpiceUsr.h").to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .clang_args(clang_args)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindgen.rs"))
        .expect("Couldn't write bindings!");

    println!(
        "cargo:rustc-link-search=native={}",
        cspice_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=cspice");
}

// Check for CSPICE installation in system library folders
fn locate_cspice() -> Option<PathBuf> {
    match env::consts::OS {
        "linux" | "macos" if Path::new("/usr/lib/libcspice.a").exists() => {
            Some(PathBuf::from("/usr"))
        }
        _ => None,
    }
}

// Fetch CSPICE source from NAIF servers and extract to `<out_dir>/cspice`
#[cfg(feature = "downloadcspice")]
fn download_cspice(out_dir: &Path) {
    // Pick appropriate package to download
    let (platform, extension) = match env::consts::OS {
        "linux" => ("PC_Linux_GCC_64bit", "tar.Z"),
        "macos" => (
            if cfg!(target_arch = "aarch64") {
                "MacM1_OSX_clang_64bit"
            } else {
                "MacIntel_OSX_AppleC_64bit"
            },
            "tar.Z",
        ),
        "windows" => ("PC_Windows_VisualC_64bit", "zip"),
        _ => {
            unimplemented!("Cannot fetch CSPICE source for this platform, please download manually")
        }
    };

    let url = format!(
        "https://naif.jpl.nasa.gov/pub/naif/toolkit//C/{}/packages/cspice.{}",
        platform, extension
    );

    let download_target = out_dir.join(format!("cspice.{}", extension));

    println!("Downloading from: {}", url);

    // Download CSPICE using ureq
    let downloaded_bytes = download_cspice_sync(&url, &download_target);

    println!("Download complete: {} bytes", downloaded_bytes);

    // Extract package based on platform
    match (env::consts::OS, extension) {
        ("linux" | "macos", "tar.Z") => {
            Command::new("gzip")
                .current_dir(out_dir)
                .args(["-d", "cspice.tar.Z"])
                .status()
                .expect("Failed to extract with gzip");
            Command::new("tar")
                .current_dir(out_dir)
                .args(["xf", "cspice.tar"])
                .status()
                .expect("Failed to extract with tar");

            fs::rename(
                out_dir.join("cspice/lib/cspice.a"),
                out_dir.join("cspice/lib/libcspice.a"),
            )
            .unwrap();
        }
        ("windows", "zip") => {
            Command::new("tar")
                .current_dir(out_dir)
                .args(["xf", "cspice.zip"])
                .status()
                .expect("Failed to extract with tar");
        }
        _ => unreachable!(),
    }
}

// For docs.rs only we will bundle the headers
// It is not a good idea to do this in general though, it should be specific to the user / platform
// https://kornel.ski/rust-sys-crate
fn docs_rs(out_dir: &Path) {
    let headers_dir = out_dir.join("docs-rs-headers");
    fs::create_dir_all(&headers_dir).expect("Unable to create CSPICE headers directory");
    let headers_tar = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs-rs-headers.tar")
        .canonicalize()
        .unwrap();
    let tar_status = Command::new("tar")
        .arg("-xf")
        .arg(&headers_tar)
        .arg("-C")
        .arg(&headers_dir)
        .status()
        .expect("Unable to call tar");
    assert!(tar_status.success());
    env::set_var("CSPICE_DIR", headers_dir.as_os_str());
}

// Synchronous download implementation using ureq
#[cfg(feature = "downloadcspice")]
fn download_cspice_sync(url: &str, download_target: &PathBuf) -> u64 {
    use std::io::{Read, Write};

    // Send request with timeout configuration
    let response = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout_read(std::time::Duration::from_secs(60))
        .build()
        .get(url)
        .call()
        .expect("Failed to start CSPICE download");

    // Check status code
    if response.status() < 200 || response.status() >= 300 {
        panic!(
            "Failed to download CSPICE: HTTP {} from {}",
            response.status(),
            url
        );
    }

    // Get content size
    let total_size = response
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok());
    
    if let Some(size) = total_size {
        println!(
            "Download size: {} bytes ({} MB)",
            size,
            size / (1024 * 1024)
        );
    }

    // Create output file
    let mut file = std::fs::File::create(download_target).expect("Failed to create download file");

    let mut downloaded = 0u64;
    let mut reader = response.into_reader();
    let mut buffer = vec![0u8; 8192]; // 8KB buffer

    // Show initial progress
    println!("Starting download from {}", url);
    if let Some(total) = total_size {
        println!("Progress: 0 MB / {} MB (0%)", total / (1024 * 1024));
    }

    // Download with progress reporting
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                file.write_all(&buffer[..n])
                    .expect("Failed to write to download file");
                
                downloaded += n as u64;

                // Progress display (every 1MB)
                if downloaded % (1024 * 1024) < n as u64 {
                    if let Some(total) = total_size {
                        let percent = (downloaded as f64 / total as f64 * 100.0) as u32;
                        println!(
                            "Progress: {} MB / {} MB ({}%)",
                            downloaded / (1024 * 1024),
                            total / (1024 * 1024),
                            percent
                        );
                    } else {
                        println!("Downloaded {} MB", downloaded / (1024 * 1024));
                    }
                }
            }
            Err(e) => panic!("Failed to read download chunk: {}", e),
        }
    }

    file.flush().expect("Failed to flush download file");

    // Verify download completion
    if let Some(total) = total_size {
        if downloaded != total {
            panic!(
                "Download incomplete: got {} bytes, expected {} bytes",
                downloaded, total
            );
        }
    }

    downloaded
}
