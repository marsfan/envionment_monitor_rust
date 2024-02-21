//! Build Script for integrating C code (ESP-IDF, drivers, etc) into the program

fn main() {
    // Linking for the ESP-IDF
    embuild::espidf::sysenv::output();
}
