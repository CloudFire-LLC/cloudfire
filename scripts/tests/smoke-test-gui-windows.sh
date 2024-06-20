#!/usr/bin/env bash
# Usage: This is made for CI, so it will change system-wide files without asking.
# Read it before running on a dev system.
# This script must run from an elevated shell so that Firezone won't try to elevate.

set -euox pipefail

# This prevents a `shellcheck` lint warning about using an unset CamelCase var
if [[ -z "$ProgramData" ]]; then
    echo "The env var \$ProgramData should be set to \`C:\ProgramData\` or similar"
    exit 1
fi

BUNDLE_ID="dev.firezone.client"
DEVICE_ID_PATH="$ProgramData/$BUNDLE_ID/config/firezone-id.json"
DUMP_PATH="$LOCALAPPDATA/$BUNDLE_ID/data/logs/last_crash.dmp"
GUI_BIN=firezone-gui-client
IPC_BIN=firezone-client-ipc

function smoke_test() {
    files=(
        "$LOCALAPPDATA/$BUNDLE_ID/config/advanced_settings.json"
        "$LOCALAPPDATA/$BUNDLE_ID/data/wintun.dll"
        "$DEVICE_ID_PATH"
    )

    # Make sure the files we want to check don't exist on the system yet
    for file in "${files[@]}"
    do
        rm -f "$file"
    done

    # Run the smoke test normally
    cargo run --bin "$IPC_BIN" -- smoke-test
    cargo run --bin "$GUI_BIN" -- smoke-test

    # Note the device ID
    DEVICE_ID_1=$(cat "$DEVICE_ID_PATH")

    # Run the test again and make sure the device ID is not changed
    cargo run --bin "$IPC_BIN" -- smoke-test
    cargo run --bin "$GUI_BIN" -- smoke-test
    DEVICE_ID_2=$(cat "$DEVICE_ID_PATH")

    if [ "$DEVICE_ID_1" != "$DEVICE_ID_2" ]
    then
        echo "The device ID should not change if the file is intact between runs"
        exit 1
    fi

    # Make sure the files were written in the right paths
    for file in "${files[@]}"
    do
        stat "$file"
    done
    stat "$LOCALAPPDATA/$BUNDLE_ID/data/logs/"connlib*log
}

function crash_test() {
    # Delete the crash file if present
    rm -f "$DUMP_PATH"

    # Fail if it returns success, this is supposed to crash
    cargo run --bin "$GUI_BIN" -- --crash && exit 1

    # Fail if the crash file wasn't written
    stat "$DUMP_PATH"
}

function get_stacktrace() {
    # Per `crash_handling.rs`
    SYMS_PATH="../target/debug/firezone-gui-client.syms"
    cargo install --quiet --locked dump_syms minidump-stackwalk
    dump_syms ../target/debug/firezone_gui_client.pdb ../target/debug/firezone-gui-client.exe --output "$SYMS_PATH"
    ls -lash ../target/debug
    minidump-stackwalk --symbols-path "$SYMS_PATH" "$DUMP_PATH"
}

smoke_test
smoke_test
crash_test
get_stacktrace

# Clean up
rm "$DUMP_PATH"
