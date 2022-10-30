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

    let cspice_dir = match env::var(CSPICE_DIR) {
        Ok(cspice_dir) => PathBuf::from(cspice_dir),
        Err(_) => {
            if let Some(path) = locate_cspice() {
                println!("Found CSPICE at {:?}/lib/libcspice.a", path);
                path
            } else {
                let downloaded = out_path.join("cspice");
                if !downloaded.exists() {
                    download_cspice(&out_path);
                }
                downloaded
            }
        }
    };
    if !cspice_dir.is_dir() {
        panic!(
            "Provided {CSPICE_DIR} ({}) is not a directory",
            cspice_dir.display()
        )
    }

    let include_dir = cspice_dir.join("include");

    assert!(
        include_dir.join("SpiceUsr.h").exists(),
        "CSPICE header files not found in {}",
        include_dir.display()
    );

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

// Fetch CSPICE source from NAIF servers and extract to `OUT_DIR/cspice`
fn download_cspice(out_dir: &Path) {
    let (platform, extension) = match env::consts::OS {
        "linux" => ("PC_Linux_GCC_64bit", "tar.Z"),
        "macos" => (
            if cfg!(target_arch = "arm") {
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

    let body = reqwest::blocking::get(url)
        .expect("Failed to download CSPICE")
        .bytes()
        .unwrap();
    std::fs::write(download_target, body).expect("Failed to write archive file");

    match (env::consts::OS, extension) {
        ("linux" | "macos", "tar.Z") => {
            Command::new("gzip")
                .current_dir(&out_dir)
                .args(["-d", "cspice.tar.Z"])
                .status()
                .expect("Failed to extract with gzip");
            Command::new("tar")
                .current_dir(&out_dir)
                .args(["xfv", "cspice.tar"])
                .status()
                .expect("Failed to extract with tar");

            std::fs::rename(
                out_dir.join("cspice/lib/cspice.a"),
                out_dir.join("cspice/lib/libcspice.a"),
            )
            .unwrap();
        }
        ("windows", "zip") => {
            Command::new("unzip")
                .current_dir(&out_dir)
                .args(["cspice.zip"])
                .status()
                .expect("Failed to extract with 'unzip'");
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
