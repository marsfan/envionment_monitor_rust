//! Build Script for integrating C code (ESP-IDF, drivers, etc) into the program

use std::env;
use std::path::PathBuf;

fn main() {
    embuild::espidf::sysenv::output();

    // Compile the bme68x driver library
    cc::Build::new()
        .file("src/bme68x_sensor_api/bme68x.c")
        .compile("bme68x_sensor_api");

    // Generate BME68x bindings
    let bme68x_bindings = bindgen::Builder::default()
        .header("src/bme68x_bindings.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bme68x_bindings
        .write_to_file(out_path.join("bme68x_bindings.rs"))
        .expect("Couldn't write bindings!");
}
