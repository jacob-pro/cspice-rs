use std::env;
use std::path::PathBuf;

const CSPICE_DIR: &str = "CSPICE_DIR";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let cspice_dir = match env::var(CSPICE_DIR) {
        Ok(cspice_dir) => PathBuf::from(cspice_dir),
        Err(_) => panic!("Unable to read {CSPICE_DIR} environment variable. It must be set to the directory of your CSPICE installation.")
    };
    println!("cargo:rerun-if-env-changed={}", CSPICE_DIR);
    if !cspice_dir.is_dir() {
        panic!(
            "Provided {CSPICE_DIR} ({}) is not a directory",
            cspice_dir.display()
        )
    }

    let include_dir = cspice_dir.join("include");

    let bindings = bindgen::Builder::default()
        .header(include_dir.join("SpiceUsr.h").to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
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
