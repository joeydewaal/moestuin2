#!/usr/bin/env bash
# Nightly: SQLite backup, yesterday's webcam rollup, retention prune.
set -euo pipefail

STATE=${MOESTUIN_STATE:-/data}
DB="$STATE/moestuin.db"
BACKUPS="$STATE/backups"
WEBCAM="$STATE/webcam"
mkdir -p "$BACKUPS"
RETAIN_DAYS=${WEBCAM_RETAIN_DAYS:-30}

STAMP="$(date +%Y-%m-%dT%H%M%S)"
sqlite3 "$DB" ".backup '$BACKUPS/moestuin-$STAMP.db'"
find "$BACKUPS" -name 'moestuin-*.db' -mtime +14 -delete || true

YDAY="$(date -d 'yesterday' +%Y-%m-%d)"
if [ -d "$WEBCAM/$YDAY" ] && [ ! -f "$WEBCAM/$YDAY.mp4" ]; then
	cd "$WEBCAM/$YDAY"
	ls seg-*.ts 2>/dev/null | sort >segments.txt
	if [ -s segments.txt ]; then
		ffmpeg -nostdin -hide_banner -loglevel warning \
			-f concat -safe 0 -i <(awk '{print "file \x27"$0"\x27"}' segments.txt) \
			-c copy "../$YDAY.mp4"
		rm -f seg-*.ts live.m3u8 segments.txt
	fi
fi

find "$WEBCAM" -maxdepth 1 -name '*.mp4' -mtime +"$RETAIN_DAYS" -delete || true
find "$WEBCAM" -maxdepth 1 -type d -empty -mtime +1 -delete || true
