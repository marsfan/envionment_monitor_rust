#!/bin/bash
# Use env var instead? https://doc.rust-lang.org/cargo/reference/config.html#environment-variables
cargo test --config "target.xtensa-esp32-espidf.runner = 'python3 ../test_wrapper.py'"