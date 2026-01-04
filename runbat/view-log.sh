#!/bin/bash

# Script to find the latest log folder
# Usage: ./find_latest_log.sh [log_directory]

LOG_DIR="${1:-logs}"

if [ ! -d "$LOG_DIR" ]; then
    echo "Error: Directory '$LOG_DIR' does not exist" >&2
    exit 1
fi

# Find the latest folder by modification time
LATEST_FOLDER=$(ls -t "$LOG_DIR" | grep -E '^[0-9]{8}-[0-9]{6}$' | head -n 1)

if [ -z "$LATEST_FOLDER" ]; then
    echo "Error: No log folders found in '$LOG_DIR'" >&2
    exit 1
fi

multitail -s 2 $LOG_DIR/$LATEST_FOLDER/*

