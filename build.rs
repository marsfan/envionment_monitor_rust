//! Build Script for integrating C code (ESP-IDF, drivers, etc) into the program

fn main() {
    embuild::espidf::sysenv::output();
    // println!("cargo:rustc-link-search=src/bsec/bin/esp/esp32/");
    println!("cargo:rustc-link-search=/home/gabe/esp32_projects/environment-monitor-rust/src/bsec/bin/esp/esp32");
    println!("cargo:rustc-link-lib=static=algobsec");
}
