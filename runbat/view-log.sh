#!/bin/bash

# Ensure we clean up background jobs on exit
trap 'kill $(jobs -p) 2>/dev/null; exit' SIGINT SIGTERM EXIT

LOG_DIR="logs"
CURRENT_LATEST=""

while true; do
    # Find the latest folder
    LATEST=$(ls -td "$LOG_DIR"/* 2>/dev/null | head -n 1)

    if [ -z "$LATEST" ]; then
        echo "No log folders found in $LOG_DIR. Waiting..."
        sleep 2
        continue
    fi

    # Update current latest
    CURRENT_LATEST="$LATEST"
    
    # Start a background monitor to check for newer folders
    (
        while true; do
            sleep 1
            CHECK_LATEST=$(ls -td "$LOG_DIR"/* 2>/dev/null | head -n 1)
            # If a new latest folder appears, kill the running multitail to trigger a reload
            if [ -n "$CHECK_LATEST" ] && [ "$CHECK_LATEST" != "$CURRENT_LATEST" ]; then
                # Kill multitail process that is a child of the main script ($$)
                pkill -P $$ -x multitail
                exit 0
            fi
        done
    ) &
    MONITOR_PID=$!

    # Run multitail in foreground
    # If it's the first iter or changed, we are here.
    multitail -s 2 "$LATEST"/*
    
    # Kill the monitor when multitail finishes (user quit or was killed)
    kill $MONITOR_PID 2>/dev/null
    wait $MONITOR_PID 2>/dev/null

    # Small pause before checking again
    sleep 1
done