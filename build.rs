//! Build Script for integrating C code (ESP-IDF, drivers, etc) into the program

fn main() {
    embuild::espidf::sysenv::output();
}
