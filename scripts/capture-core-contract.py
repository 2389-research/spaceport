# ABOUTME: Captures the methods and schema advertised by an isolated Herdr Core.
# ABOUTME: Starts only the supplied binary with private state and stops that child.

import argparse
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def call(stream, request_id, method, params):
    request = {"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}
    stream.write(json.dumps(request).encode() + b"\n")
    stream.flush()
    reply = stream.readline(16 * 1024 * 1024 + 1)
    if not reply:
        raise RuntimeError(f"Core closed the connection during {method}")
    if len(reply) > 16 * 1024 * 1024 or not reply.endswith(b"\n"):
        raise RuntimeError(f"Core sent an oversized or incomplete {method} reply")
    answer = json.loads(reply)
    if answer.get("id") != request_id or "error" in answer:
        raise RuntimeError(f"Core rejected {method}: {answer}")
    return answer["result"]


def capture(binary, log_path):
    with tempfile.TemporaryDirectory(prefix="spcore-", dir="/tmp") as private:
        root = Path(private)
        endpoint = root / "core.sock"
        environment = {
            "HOME": private,
            "PATH": "/usr/bin:/bin",
            # The private serve child must stay ours to stop and reap; macOS
            # otherwise moves it into launchd and outlives this sandbox.
            "HC_LAUNCHD_DIRECT": str(os.getpid()),
            "HC_SESSION": "spaceport-t00",
            "HC_SOCKET": str(endpoint),
            "TMPDIR": private,
            "XDG_CACHE_HOME": str(root / "cache"),
            "XDG_CONFIG_HOME": str(root / "config"),
            "XDG_DATA_HOME": str(root / "data"),
            "XDG_RUNTIME_DIR": private,
            "XDG_STATE_HOME": str(root / "state"),
        }
        with log_path.open("xb") as log:
            child = subprocess.Popen(
                [str(binary), "serve"],
                env=environment,
                cwd=private,
                stdin=subprocess.DEVNULL,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            try:
                deadline = time.monotonic() + 10
                while not endpoint.exists():
                    if child.poll() is not None:
                        raise RuntimeError(
                            f"Core exited with status {child.returncode}"
                        )
                    if time.monotonic() >= deadline:
                        raise TimeoutError("Core did not create its private socket")
                    time.sleep(0.05)
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
                    connection.settimeout(10)
                    connection.connect(str(endpoint))
                    with connection.makefile("rwb") as stream:
                        hello = call(
                            stream, 1, "hello", {"generation": 1, "framed": False}
                        )
                        schema = call(stream, 2, "rpc.discover", {})
                        advertised = set(hello["methods"])
                        described = {method["name"] for method in schema["methods"]}
                        if (
                            len(advertised) != len(hello["methods"])
                            or advertised != described
                        ):
                            raise RuntimeError(
                                "hello.methods differs from rpc.discover methods"
                            )
                        if child.poll() is not None:
                            raise RuntimeError(
                                "Core detached from its private test child"
                            )
                        return {
                            "version": hello["version"],
                            "generation": hello["generation"],
                            "features": hello["features"],
                            "methods": hello["methods"],
                            "schema": schema,
                        }
            finally:
                stop = None
                stop_error = None
                if endpoint.exists():
                    try:
                        stop = subprocess.run(
                            [str(binary), "server", "stop"],
                            env=environment,
                            cwd=private,
                            stdin=subprocess.DEVNULL,
                            stdout=log,
                            stderr=subprocess.STDOUT,
                            timeout=10,
                            check=False,
                        )
                    except (OSError, subprocess.TimeoutExpired) as error:
                        stop_error = error
                if child.poll() is None and (stop is None or stop.returncode != 0):
                    child.terminate()
                child_timed_out = False
                try:
                    child.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    child_timed_out = True
                    child.kill()
                    child.wait()
                if child_timed_out:
                    raise RuntimeError("Core private child did not stop within 10 seconds")
                if stop_error is not None:
                    raise RuntimeError("Core private server stop failed") from stop_error
                if stop is not None and stop.returncode != 0:
                    raise RuntimeError(
                        f"Core private server stop failed with status {stop.returncode}"
                    )


def main():
    parser = argparse.ArgumentParser(
        description="Capture hello and rpc.discover from a private Core fixture."
    )
    parser.add_argument(
        "--binary", type=Path, required=True, help="Absolute Core binary"
    )
    parser.add_argument("--output", type=Path, required=True, help="JSON evidence path")
    args = parser.parse_args()
    if os.name != "posix":
        parser.error(
            "this fixture uses a Unix socket; use a native Windows fixture there"
        )
    if not args.binary.is_absolute() or not args.binary.is_file():
        parser.error("--binary must name an absolute executable file")
    if not os.access(args.binary, os.X_OK):
        parser.error("--binary is not executable")
    if not args.output.parent.is_dir():
        parser.error("--output parent directory does not exist")

    log_path = args.output.with_suffix(".log")
    try:
        result = capture(args.binary, log_path)
        with tempfile.NamedTemporaryFile(
            mode="w", dir=args.output.parent, delete=False
        ) as temp:
            json.dump(result, temp, indent=2)
            temp.write("\n")
            staged = Path(temp.name)
        try:
            os.link(staged, args.output)
        finally:
            staged.unlink(missing_ok=True)
    except (OSError, ValueError, KeyError, RuntimeError, TimeoutError) as error:
        print(f"capture failed: {error}; Core log: {log_path}", file=sys.stderr)
        return 1
    print(
        f"captured {len(result['methods'])} methods from Core {result['version']}: {args.output}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
