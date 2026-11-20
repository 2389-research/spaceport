#!/bin/sh
# ABOUTME: Captures a bounded native key sequence inside the owned daemon pane.
# ABOUTME: Restores the real terminal discipline and emits correlated completion markers.
LC_ALL=C
export LC_ALL
if [ "$#" -ne 3 ]; then
    printf '%s\n' 'Usage: capture.sh NONCE CASE BYTE_COUNT' >&2
    exit 64
fi
saved_tty=$(stty -g) || exit 1
trap 'stty "$saved_tty"' 0
trap 'exit 1' HUP INT TERM
stty raw -echo || exit 1
printf '\r\nPANE_%s_%s_READY\r\n' "$1" "$2" || exit 1
od -A n -v -t x1 -N "$3"
capture_status=$?
stty "$saved_tty" || exit 1
trap - 0 HUP INT TERM
[ "$capture_status" -eq 0 ] || exit "$capture_status"
printf '\r\nPANE_%s_%s_DONE\r\n' "$1" "$2"
