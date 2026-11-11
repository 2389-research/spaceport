#!/usr/bin/env python3
"""ABOUTME: Run one notarization upload with a deadline and durable receipts.
ABOUTME: Reap only the owned upload child; uncertain outcomes never trigger retries.
"""
import argparse
from pathlib import Path
import signal
import subprocess
import sys


def upload(command, receipt, timeout):
    with (receipt / 'submission.json').open('xb') as stdout, (receipt / 'submit.stderr').open('xb') as stderr:
        child = subprocess.Popen(command, stdout=stdout, stderr=stderr)
        try:
            return child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            print('Notarization upload deadline exceeded; outcome unknown; do not resubmit. '
                  f'Receipt: {receipt}', file=sys.stderr)
            return 124
        finally:
            if child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait()


def interrupted(signum, _frame):
    raise SystemExit(128 + signum)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout-seconds', type=float, default=300,
                        help='Upload deadline in seconds (default: 300); never retries')
    parser.add_argument('receipt', type=Path, help='Existing reserved receipt directory')
    parser.add_argument('command', nargs=argparse.REMAINDER, help='Upload command after --')
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command or not 0 < args.timeout_seconds <= 300:
        parser.error('A command and a positive deadline of at most 300 seconds are required')
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, interrupted)
    try:
        status = upload(command, args.receipt, args.timeout_seconds)
    except OSError:
        print(f'Notarization upload failed; outcome unknown; do not resubmit. Receipt: {args.receipt}',
              file=sys.stderr)
        return 1
    return status if status >= 0 else 128 - status


if __name__ == '__main__':
    sys.exit(main())
