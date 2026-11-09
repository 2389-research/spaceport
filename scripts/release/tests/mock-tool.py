#!/usr/bin/env python3
"""Apple-tool stand-ins: never access a keychain, network, or executable."""
import base64
import json
import os
from pathlib import Path
import shutil
import signal
import sys

tool = Path(sys.argv[0]).name
args = sys.argv[1:]
if tool == "uname":
    assert args == ["-s"], args
    print(os.environ.get("MOCK_OS", "Darwin"))
    sys.exit(0)
with open(os.environ["MOCK_LOG"], "a") as log:
    log.write(json.dumps([tool, *args]) + "\n")
if os.environ.get("MOCK_FAIL") == tool:
    sys.exit(1)
# The first MOCK_XCRUN_FAILURES xcrun calls fail, like a dropped connection.
if tool == "xcrun" and os.environ.get("MOCK_XCRUN_FAILURES"):
    counter = Path(os.environ["HOME"]) / "xcrun-failures"
    failed = int(counter.read_text()) if counter.exists() else 0
    if failed < int(os.environ["MOCK_XCRUN_FAILURES"]):
        counter.write_text(str(failed + 1))
        sys.exit(1)
if tool == "security":
    if args == ["list-keychains", "-d", "user"]:
        print('    "/mock/login keychain-db"\n    "/mock/system.keychain"')
    elif args[0] == "find-identity":
        print('1) FF599C9C778C879ECB7DC8AEF258890A1268F7E6 "Developer ID Application: 2389 Research, Inc (HD9NM9NSMK)"')
        print(f"  {os.environ.get('MOCK_IDENTITY_COUNT', '1')} valid identities found")
    elif args[0] == "create-keychain":
        Path(args[-1]).write_bytes(b"fixture zip")
    elif args[0] == "delete-keychain":
        Path(args[-1]).unlink()
elif tool == "openssl":
    print("dummy-temporary-password")
elif tool == "base64":
    assert args == ["-D"], args
    sys.stdout.buffer.write(base64.b64decode(sys.stdin.buffer.read(), validate=True))
elif tool == "plutil" and args[0] == "-extract":
    print({"LSMinimumSystemVersion": "14.2", "CFBundleExecutable": "spaceport",
           "CFBundleIdentifier": "so.pen.herdr-gpui"}.get(args[1], "20260920.3"))
elif tool == "lipo":
    if args[0] == "-archs":
        # A fixture is named for its architecture, or sits in a folder that is.
        path = Path(args[1])
        print(os.environ.get("MOCK_ARCHS", path.name if path.name in ("arm64", "x86_64") else path.parent.name))
    elif args[0] == "-create":
        Path(args[-1]).write_text("never execute this fixture")
    else:
        assert len(args) == 3 and args[1] == "-verify_arch" and args[2] in ("arm64", "x86_64"), args
        assert Path(args[0]).is_file(), args
elif tool == "ditto":
    if args[0] == "-c":
        Path(args[-1]).write_bytes(b"fixture zip")
    else:
        shutil.copytree(args[0], args[1])
elif tool == "xcrun" and args[:2] == ["notarytool", "submit"]:
    if os.environ.get("MOCK_SUBMIT_CANCEL"):
        os.kill(os.getppid(), signal.SIGTERM)
        sys.exit(1)
    if os.environ.get("MOCK_SUBMIT_UNKNOWN"):
        print("controlled connection loss", file=sys.stderr)
        sys.exit(1)
    print(os.environ.get("MOCK_NOTARY_JSON", '{"id":"12345678-1234-1234-1234-123456789abc"}'))
elif tool == "xcrun" and args[:2] == ["notarytool", "wait"]:
    print(json.dumps({"id": args[2], "status": os.environ.get("MOCK_NOTARY_STATUS", "Accepted")}))
elif tool == "xcrun" and args[:2] == ["notarytool", "log"]:
    Path(args[-1]).write_text('{"issues": ["controlled rejection"]}')
elif tool == "codesign" and args[:2] == ["-d", "--verbose=4"]:
    print(f"TeamIdentifier={os.environ.get('MOCK_SIGNED_TEAM', 'HD9NM9NSMK')}", file=sys.stderr)
    print(f"Identifier={os.environ.get('MOCK_SIGNED_ID', 'ai.2389.spaceport')}", file=sys.stderr)
elif tool == "xcrun" and args[:2] == ["stapler", "staple"] and Path(args[-1]).is_dir():
    (Path(args[-1]) / "Contents/CodeResources").write_bytes(b"mock stapled ticket")
elif tool == "codesign" and args[0] == "--force" and Path(args[-1]).is_dir():
    signature = Path(args[-1]) / "Contents/_CodeSignature"
    signature.mkdir()
    (signature / "CodeResources").write_bytes(b"mock code signature")
elif tool == "hdiutil":
    image = Path(args[args.index("-srcfolder") + 1])
    assert (image / "Applications").is_symlink()
    assert os.readlink(image / "Applications") == "/Applications"
    assert (image / "Spaceport.app").is_dir()
    Path(args[-1]).write_text("mock dmg")
