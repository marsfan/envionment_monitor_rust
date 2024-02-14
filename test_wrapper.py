#!/usr/bin/env python3
# -*- coding: UTF-8 -*-
"""Wrapper to run rust unit test binaries in QEMU.

This expects a single argument passed in via the command line, which is
the path to the binary to run. It will then use espflash to merge the
binary into one that can be run in QEMU, and then run it in QEMU.

Currently it does not support quitting QEMU at the end of the binary
or if an error ocurrs. That functionality will be added later.

"""
import subprocess
import sys
from pathlib import Path
from os import environ

# TODO: Better way to config this
QEMU_PATH = f"{environ['HOME']}/qemu-xtensa/bin/qemu-system-xtensa"


def main() -> None:
    """Run when function is called from CLI."""
    test_binary = Path(sys.argv[1])

    # TODO: use tempfile mopdule for a temp dir
    target_file = Path("test.qemu")

    # Generate the fully binary to run
    subprocess.run(
        [
            "espflash",
            "save-image",
            "--merge",
            "--chip",
            "esp32",
            "--partition-table",
            "../environment-monitor/partition_table.csv",
            test_binary,
            target_file
        ],
        check=True
    )

    # Run the binary in QEMU
    # TODO: Realtime processing of output to detect finish/crash
    # Probably requires "expect" package
    subprocess.run(
        [
            QEMU_PATH,
            "-nographic",
            "-machine",
            "esp32",
            "-drive",
            f"file={target_file},if=mtd,format=raw"

        ],
        check=True
    )

    if target_file.exists():
        target_file.unlink()


if __name__ == "__main__":
    main()
