#!/usr/bin/env bash
# ffmpeg capture loop. Writes HLS segments into today's dated directory.
# When MOESTUIN_MOCK_HW=1, swaps the v4l2 device for a synthetic lavfi source
# so dev boxes without /dev/video0 still exercise the full HLS pipeline.
set -euo pipefail

ROOT=${WEBCAM_ROOT:-/data/webcam}
DEVICE=${MOESTUIN_VIDEO_DEVICE:-/dev/video0}
MOCK=${MOESTUIN_MOCK_HW:-0}
mkdir -p "$ROOT"

if [ "$MOCK" = "1" ]; then
	echo "MOESTUIN_MOCK_HW=1 — using synthetic testsrc input" >&2
	INPUT=(-f lavfi -i "testsrc=size=1280x720:rate=15,format=yuv420p")
else
	INPUT=(-f v4l2 -i "$DEVICE")
fi

while true; do
	DAY="$(date +%Y-%m-%d)"
	mkdir -p "$ROOT/$DAY"
	cd "$ROOT/$DAY"

	ffmpeg -nostdin -hide_banner -loglevel warning \
		"${INPUT[@]}" \
		-r 15 -s 1280x720 \
		-c:v libx264 -preset veryfast -g 30 \
		-f hls -hls_time 5 -hls_list_size 12 \
		-hls_flags delete_segments+append_list \
		-strftime 1 -hls_segment_filename "seg-%Y%m%d-%H%M%S.ts" \
		live.m3u8 || true

	# Loop forever; each iteration either day-rolls or restarts after ffmpeg crashes.
	sleep 2
done
