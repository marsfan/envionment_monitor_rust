#!/usr/bin/env python3
# -*- coding: UTF-8 -*-
"""Wrapper to run rust unit test binaries in QEMU.

This expects a single argument passed in via the command line, which is
the path to the binary to run. It will then use espflash to merge the
binary into one that can be run in QEMU, and then run it in QEMU.

"""
import subprocess
import sys
import tempfile
from argparse import ArgumentParser
from os import environ
from pathlib import Path

import pexpect

# TODO: Better way to config this. CLI arg?

QEMU_PATH = f"{environ['HOME']}/qemu-xtensa/bin/qemu-system-xtensa"


# TODO: Support specifying the arch?
def invoker() -> None:
    """Function for the runner to act as the master that invokes cargo test."""
    subprocess.run(
        [
            "cargo",
            "test",
            "--config",
            "target.xtensa-esp32-espidf.runner = 'python3 ../test_wrapper.py'"
        ],
        check=True
    )


def test_instance(binary: str) -> bool:
    """Function to run an individually specified test binary and process it.

    Arguments:
        binary: The path to the binary to run.

    Returns:
        Boolean indicating if all tests passed or failed.

    """
    target_file = Path("test.qemu")

    with tempfile.TemporaryDirectory() as temp_dir:
        target_file = Path(temp_dir) / "test.qemu"

        # Generate the fully binary to run
        subprocess.run(
            [
                "espflash",
                "save-image",
                "--merge",
                "--chip",
                "esp32",
                "--partition-table",
                "../partition_table.csv",
                binary,
                target_file
            ],
            check=True
        )

        # Run the binary in QEMU
        child = pexpect.spawn(
            QEMU_PATH,
            [
                "-nographic",
                "-machine",
                "esp32",
                "-drive",
                f"file={target_file},if=mtd,format=raw"
            ],
            encoding="utf-8"
        )
        # Also print out to console the results
        child.logfile_read = sys.stdout

        # TODO: On abort message, previous line will be the failed test.
        # Might want to log that.
        # TODO: Only print out test result lines?
        child.expect(
            r"(Returned from app_main\(\))|(abort\(\) was called at PC)"
        )
        passed = "Returned from app_main()" in child.after
        child.send("\x01\x11cq")

    return passed


def main() -> None:
    """Run when function is called from CLI."""

    parser = ArgumentParser(
        description="""Wrapper tool to run ESP32 unit test binaries in QEMU.
        The behavior of the tool depends on the number of positional arguments supplied.
        If none are supplied, it will act as the master runner, executing `cargo test` with the
        necessary setting changes to invoke QEMU. If one argument is supplied, it
        should be the path to the test binary that will be run."""
    )
    parser.add_argument(
        "test_binary",
        nargs="?",
        help="Path to a binary to run. Leave unspecified to act as main runner"
    )
    args = parser.parse_args()
    if args.test_binary is None:
        invoker()
    else:
        result = test_instance(args.test_binary)
        if not result:
            sys.exit(1)


if __name__ == "__main__":
    main()
