#!/usr/bin/env bash
# ffmpeg capture loop. Writes HLS segments into today's dated directory.
set -euo pipefail

ROOT=${WEBCAM_ROOT:-/data/webcam}
DEVICE=${MOESTUIN_VIDEO_DEVICE:-/dev/video0}
mkdir -p "$ROOT"

while true; do
	DAY="$(date +%Y-%m-%d)"
	mkdir -p "$ROOT/$DAY"
	cd "$ROOT/$DAY"

	ffmpeg -nostdin -hide_banner -loglevel warning \
		-f v4l2 -i "$DEVICE" \
		-r 15 -s 1280x720 \
		-c:v libx264 -preset veryfast -g 30 \
		-f hls -hls_time 5 -hls_list_size 12 \
		-hls_flags delete_segments+append_list \
		-strftime 1 -hls_segment_filename "seg-%Y%m%d-%H%M%S.ts" \
		live.m3u8 || true

	# Loop forever; each iteration either day-rolls or restarts after ffmpeg crashes.
	sleep 2
done
