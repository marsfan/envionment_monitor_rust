//! Build Script for integrating C code (ESP-IDF, drivers, etc) into the program

use std::env;
use std::path::PathBuf;

fn main() {
    embuild::espidf::sysenv::output();
}
