#!/usr/bin/env bash
set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR" || exit 1

BINARY="./target/release/raidlords-bot"
LOG_FILE="raidlords-bot.log"
SESSION_NAME="raidlords-bot"

build_release() {
    echo "Building release binary..."
    if ! cargo build --release; then
        echo "Build failed, aborting."
        exit 1
    fi
}

prepare() {
    if [ ! -f ".env" ]; then
        echo ".env not found, create it first (PRIVATEKEY=0x...)."
        exit 1
    fi
    if ! grep -qE '^PRIVATEKEY=0x[0-9a-fA-F]{64}$' .env; then
        echo ".env has no usable PRIVATEKEY, fill it with PRIVATEKEY=0x followed by 64 hex characters."
        exit 1
    fi
    if [ ! -x "$BINARY" ] || [ "Cargo.toml" -nt "$BINARY" ]; then
        build_release
    fi
}

mode_direct() {
    prepare
    exec "$BINARY" "${1:-run}"
}

mode_nohup() {
    prepare
    nohup "$BINARY" "${1:-run}" >> "$LOG_FILE" 2>&1 &
    echo "Started in background (PID $!)"
    echo "Logs: tail -f $LOG_FILE"
}

mode_screen() {
    if ! command -v screen >/dev/null 2>&1; then
        echo "screen not found, install with: apt install screen"
        exit 1
    fi
    prepare
    screen -dmS "$SESSION_NAME" "$BINARY" "${1:-run}"
    echo "Started in screen session '$SESSION_NAME'"
    echo "Attach : screen -r $SESSION_NAME"
    echo "Detach : Ctrl+A then D"
}

mode_stop() {
    if command -v screen >/dev/null 2>&1 && screen -ls | grep -q "$SESSION_NAME"; then
        screen -S "$SESSION_NAME" -X quit
        echo "Screen session stopped."
    fi
    if pkill -f "$BINARY" >/dev/null 2>&1; then
        echo "Process stopped."
    else
        echo "No running process found."
    fi
}

case "${1:-run}" in
    run|once|probe|selftest|verify|renew|entry)
        mode_direct "$1"
        ;;
    name)
        if [ -z "${2:-}" ]; then
            echo "Usage: ./run.sh name <your name>"
            exit 1
        fi
        prepare
        exec "$BINARY" "name" "$2"
        ;;
    paid)
        if [ -z "${2:-}" ]; then
            echo "Usage: ./run.sh paid <transaction hash>"
            exit 1
        fi
        prepare
        exec "$BINARY" "paid" "$2"
        ;;
    withdraw)
        if [ -z "${2:-}" ]; then
            echo "Usage: ./run.sh withdraw <amount>"
            exit 1
        fi
        prepare
        exec "$BINARY" "withdraw" "$2"
        ;;
    bg)
        mode_nohup "run"
        ;;
    screen)
        mode_screen "run"
        ;;
    stop)
        mode_stop
        ;;
    build)
        build_release
        ;;
    *)
        echo "Usage: ./run.sh [run|once|probe|entry|name <name>|paid <hash>|selftest|verify|renew|withdraw <amount>|bg|screen|stop|build]"
        exit 1
        ;;
esac
