extern crate core;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

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
        Err(_) => panic!("Unable to read {CSPICE_DIR} environment variable. It must be set to the directory of your CSPICE installation.")
    };
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

// For docs.rs only we will download and extract the CSPICE library automatically
// It is not a good idea to do this in general though, it should be specific to the user / platform
// https://kornel.ski/rust-sys-crate
fn docs_rs(out_dir: &Path) {
    let outfile = out_dir.join("cspice.tar.Z");
    let curl_status = Command::new("curl")
        .arg("https://naif.jpl.nasa.gov/pub/naif/toolkit//C/PC_Linux_GCC_64bit/packages/cspice.tar.Z")
        .arg("-o")
        .arg(outfile.display().to_string())
        .status()
        .expect("Unable to call curl");
    assert!(curl_status.success());
    let tar_status = Command::new("tar")
        .arg("-zxf")
        .arg("cspice.tar.Z")
        .current_dir(out_dir)
        .status()
        .expect("Unable to call tar");
    assert!(tar_status.success());
    env::set_var("CSPICE_DIR", out_dir.join("cspice").as_os_str());
}
