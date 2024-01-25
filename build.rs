//! Build Script for integrating C code (ESP-IDF, drivers, etc) into the program
use std::env;
use std::path::PathBuf;

fn main() {
    // Linking for the ESP-IDF
    embuild::espidf::sysenv::output();

    // Linking for the BSEC library
    // FIXME: Use path module to use local path so it is not limited to my computer
    println!("cargo:rustc-link-search=/home/gabe/esp32_projects/environment-monitor-rust/src/bsec/bin/esp/esp32");
    println!("cargo:rustc-link-lib=static=algobsec");

    // Use bindgen to generate the bindings for BSEC library.
    let bindings = bindgen::Builder::default()
        .header("src/bsec/inc/bsec_datatypes.h")
        .header("src/bsec/inc/bsec_interface.h")
        .prepend_enum_name(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the build directory
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bsec_bindings.rs"))
        .expect("Failed to write bindings.");
}
