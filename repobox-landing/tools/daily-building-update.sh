#!/bin/bash

# Daily building status data regeneration
# Run this via cron to keep the /building page current

cd "$(dirname "$0")/.." || exit 1

echo "$(date): Starting daily building status update..."

# Regenerate building status data
./tools/generate-building-data.sh

echo "$(date): Daily building status update completed."