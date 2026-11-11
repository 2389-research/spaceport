#!/usr/bin/env python3
# ABOUTME: Build and run isolated, genuinely signed native Sparkle update cases.
# ABOUTME: Retain bounded evidence and wait for real native confirmation buttons.
import argparse
import base64
import hashlib
import http.server
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import re
import pwd
import shutil
import socket
import stat
import subprocess
import sys
import tempfile
import threading
import time
import uuid
import xml.etree.ElementTree as ET

CASES = ('valid-zip', 'valid-dmg', 'altered-feed', 'wrong-feed-key', 'altered-zip',
         'wrong-archive-key', 'wrong-name-id', 'same-name-wrong-id', 'downgrade',
         'payload-downgrade', 'aged-wrong-feed-key')
IDENTITY = 'FF599C9C778C879ECB7DC8AEF258890A1268F7E6'
NS = 'http://www.andymatuschak.org/xml-namespaces/sparkle'
ET.register_namespace('sparkle', NS)
LIMIT = 1024 * 1024


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def case_versions(case):
    if case not in CASES:
        raise ValueError(f'Unknown fixture case: {case}')
    return ('2', '1', '3' if case == 'payload-downgrade' else '1') if 'downgrade' in case else ('1', '2', '2')


def case_check_mode(case):
    case_versions(case)
    return 'information' if case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key', 'downgrade') else 'manual'


def owned_path(root, path):
    root, path = Path(root).resolve(), Path(path).resolve()
    if path == root or not path.is_relative_to(root):
        raise ValueError(f'Path leaves fixture root: {path}')
    return path


def validate_native_temp(actual, pinned, owner, expected_owner, mode):
    if Path(actual).resolve() != Path(pinned).resolve() or owner != expected_owner or mode & 0o777 != 0o700:
        raise ValueError('Native temp must match the exact pinned private owner/path')


def native_temp(root):
    output = subprocess.check_output(['/usr/bin/getconf', 'DARWIN_USER_TEMP_DIR'], timeout=5)
    path = Path(output.decode().strip()).resolve(strict=True)
    metadata = path.stat()
    validate_native_temp(path, path, metadata.st_uid, root.stat().st_uid, metadata.st_mode)
    return path, metadata.st_uid


def preferences_path(domain, home):
    if not re.fullmatch(r'ai\.2389\.spaceport\.fixture\.[0-9a-f]{32}', domain):
        raise ValueError('Preferences domain must be an exact unique fixture ID')
    return Path(home) / 'Library/Preferences' / (domain + '.plist')


def preferences_record(domain, home):
    path = preferences_path(domain, home)
    if path.exists() or path.is_symlink():
        raise ValueError(f'Fixture preferences already exist: {path}')
    return {'domain': domain, 'path': str(path), 'absent_before': True, 'owner': os.getuid()}


def preferences_cleanup_path(record, home, allowed):
    path = preferences_path(record['domain'], home)
    if (record['domain'] not in allowed or record.get('absent_before') is not True
            or record['path'] != str(path) or record['owner'] != os.getuid()):
        raise ValueError('Refusing preferences cleanup outside recorded fixture ownership')
    return path


def preferences_missing(returncode, error):
    return returncode == 1 and (b'does not exist' in error or (b'Domain ' in error and b'not found' in error))


def preferences_absent(domain):
    # Never print values if a domain unexpectedly exists; only the exit/error is evidence.
    result = subprocess.run(['/usr/bin/defaults', 'read', domain], stdout=subprocess.DEVNULL,
                            stderr=subprocess.PIPE, timeout=10, check=False)
    return preferences_missing(result.returncode, result.stderr)


def preferences_private(path):
    if path.is_symlink():
        raise ValueError('Fixture preferences must not be symlinked')
    if path.exists():
        metadata = path.stat()
        if metadata.st_uid != os.getuid() or metadata.st_mode & 0o777 != 0o600:
            raise ValueError('Fixture preferences require expected owner and mode0600')


def claim_preferences(root, manifest):
    home = Path(pwd.getpwuid(os.getuid()).pw_dir)
    records = []
    for domain in sorted({manifest['bundle_id'], manifest['payload_id']}):
        record = preferences_record(domain, home)
        if not preferences_absent(domain):
            raise ValueError('Fixture preferences domain already exists or absence cannot be verified')
        records.append(record)
    manifest['preferences'] = records
    manifest['preferences_home'] = str(home)
    (root / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


def preferences_values_absent(data):
    values = plistlib.loads(data)
    if not isinstance(values, dict):
        raise ValueError('Fixture preferences must be a dictionary')
    return not values


def preferences_clean_state(domain, path):
    public_absent = preferences_absent(domain)
    if not public_absent:
        result = subprocess.run(['/usr/bin/defaults', 'export', domain, '-'],
                                capture_output=True, timeout=10, check=True)
        if len(result.stdout) > LIMIT or not preferences_values_absent(result.stdout):
            raise ValueError('Fixture preference values remain after cleanup')
    if path.exists():
        preferences_private(path)
        if path.stat().st_size > LIMIT or not preferences_values_absent(path.read_bytes()):
            raise ValueError('Fixture preference backing file retains values')
    return {'domain': domain, 'values_absent_after': True,
            'domain_absent_after': public_absent, 'empty_backing_file': path.exists()}


def cleanup_preferences(root, manifest):
    allowed = {manifest['bundle_id'], manifest['payload_id']}
    records = manifest.get('preferences', [])
    home = Path(pwd.getpwuid(os.getuid()).pw_dir)
    evidence = []
    for record in records:
        path = preferences_cleanup_path(record, home, allowed)
        preferences_private(path)
        if not preferences_absent(record['domain']):
            command(['/usr/bin/defaults', 'delete', record['domain']], root, timeout=10)
        evidence.append(preferences_clean_state(record['domain'], path))
    (root / 'preferences-cleanup.json').write_text(json.dumps(evidence, indent=2) + '\n')


def cache_path(domain, home):
    # Reuse the exact random fixture-ID boundary used for preferences.
    preferences_path(domain, home)
    return Path(home) / 'Library/Caches' / domain


def cache_parent(home):
    home = Path(home)
    for path in (home, home / 'Library', home / 'Library/Caches'):
        metadata = path.lstat()
        if (not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.getuid()
                or metadata.st_mode & 0o022):
            raise ValueError('Cache ancestors must be nonsymlink owned directories without shared write access')
    return home / 'Library/Caches'


def cache_record(domain, home):
    parent = cache_parent(home)
    path = cache_path(domain, home)
    path.mkdir(mode=0o700)  # Atomic refusal of existing directories and symlinks.
    metadata, ancestor = path.lstat(), parent.stat()
    if (not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.getuid()
            or metadata.st_mode & 0o777 != 0o700):
        raise ValueError('New fixture cache claim has unexpected owner or mode')
    return dict(domain=domain, home=str(home), path=str(path), absent_before=True,
                owner=metadata.st_uid, device=metadata.st_dev, inode=metadata.st_ino,
                parent_device=ancestor.st_dev, parent_inode=ancestor.st_ino)


def cache_cleanup_path(record, homes, allowed):
    home = Path(record['home'])
    path = cache_path(record['domain'], home)
    if (home not in homes or record['domain'] not in allowed
            or record.get('absent_before') is not True or record['path'] != str(path)
            or record['owner'] != os.getuid()):
        raise ValueError('Refusing cache cleanup outside recorded fixture ownership')
    parent = cache_parent(home).stat()
    if (parent.st_dev, parent.st_ino) != (record['parent_device'], record['parent_inode']):
        raise ValueError('Fixture cache parent was replaced')
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return path
    if (not stat.S_ISDIR(metadata.st_mode) or metadata.st_mode & 0o777 != 0o700
            or (metadata.st_uid, metadata.st_dev, metadata.st_ino)
            != (record['owner'], record['device'], record['inode'])):
        raise ValueError('Fixture cache claim was replaced or changed ownership/mode')
    return path


def rollback_cache_records(records, homes, allowed):
    # No host/helper has launched. Only remove exact same-inode EMPTY claims.
    for record in reversed(records):
        path = cache_cleanup_path(record, homes, allowed)
        if path.exists():
            path.rmdir()


def cache_homes(root):
    return {Path(pwd.getpwuid(os.getuid()).pw_dir), Path(root) / 'home'}


def claim_caches(root, manifest):
    homes, allowed = cache_homes(root), {manifest['bundle_id'], manifest['payload_id']}
    records = []
    try:
        # Check the whole set before creating anything; mkdir still enforces absence atomically.
        for home in homes:
            cache_parent(home)
            for domain in allowed:
                path = cache_path(domain, home)
                if path.exists() or path.is_symlink():
                    raise ValueError('Fixture cache domain already exists; it must remain untouched')
        for home in sorted(homes):
            for domain in sorted(allowed):
                records.append(cache_record(domain, home))
        manifest['caches'] = records
        (root / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    except (ValueError, OSError):
        rollback_cache_records(records, homes, allowed)
        raise


def fixture_processes(root, paths):
    # Read executable paths only, never unrelated process arguments. These
    # observations authorize refusal/retention; they never authorize signals.
    result = subprocess.run(['/bin/ps', '-axo', 'pid=,uid=,comm='],
                            capture_output=True, text=True, timeout=5, check=True)
    if len(result.stdout) > LIMIT or result.stderr:
        raise ValueError('Executable-path census failed or exceeded its bound')
    prefixes = [str(Path(root).resolve()) + '/'] + [str(path) + '/' for path in paths]
    matches = []
    for line in result.stdout.splitlines():
        fields = line.strip().split(None, 2)
        if len(fields) != 3 or not fields[0].isdigit() or not fields[1].isdigit():
            raise ValueError('Executable-path census returned an unexpected row')
        executable = str(Path(fields[2]).resolve()) if fields[2].startswith('/') else ''
        if int(fields[0]) != os.getpid() and any(executable.startswith(prefix) for prefix in prefixes):
            matches.append(int(fields[0]))
    return matches


def cache_open_processes(path):
    result = subprocess.run(['/usr/sbin/lsof', '-nP', '-F', 'p', '+D', str(path)],
                            capture_output=True, text=True, timeout=10, check=False)
    # lsof returns 1 with empty stdout/stderr for a successfully searched empty tree.
    if result.returncode == 1 and not result.stdout and not result.stderr:
        return []
    lines = result.stdout.splitlines()
    if (result.returncode != 0 or result.stderr or len(result.stdout) > LIMIT
            or not lines or any(not (re.fullmatch(r'p[0-9]+', line) or line.startswith('f')) for line in lines)):
        raise ValueError('Scoped cache lsof check failed; cache retained')
    pids = [int(line[1:]) for line in lines if line.startswith('p')]
    if not pids:
        raise ValueError('Scoped cache lsof check returned no process identity; retained')
    return pids


def cache_tree_owned(path):
    for base, directories, files in os.walk(path, followlinks=False):
        for name in directories + files:
            metadata = (Path(base) / name).lstat()
            if metadata.st_uid != os.getuid() or not (stat.S_ISDIR(metadata.st_mode)
                    or stat.S_ISREG(metadata.st_mode) or stat.S_ISLNK(metadata.st_mode)):
                raise ValueError('Fixture cache contains unowned or special files; retained')


def cleanup_cache_records(records, homes, allowed, root):
    if not records or not shutil.rmtree.avoids_symlink_attacks:
        raise ValueError('Cache cleanup requires prior claims and fd-safe tree deletion')
    paths = [cache_cleanup_path(record, homes, allowed) for record in records]
    if fixture_processes(root, paths):
        raise ValueError('Active fixture/helper process; cache retained')
    for path in paths:
        if path.exists():
            cache_tree_owned(path)
            if cache_open_processes(path):
                raise ValueError('Cache has open files or active processes; retained')
    if fixture_processes(root, paths):
        raise ValueError('Active fixture/helper process after cache inspection; retained')
    for record, path in zip(records, paths):
        cache_cleanup_path(record, homes, allowed)
        if not path.exists():
            continue
        parent_fd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            parent = os.fstat(parent_fd)
            current = os.stat(path.name, dir_fd=parent_fd, follow_symlinks=False)
            if ((parent.st_dev, parent.st_ino) != (record['parent_device'], record['parent_inode'])
                    or (current.st_dev, current.st_ino, current.st_uid)
                    != (record['device'], record['inode'], record['owner'])
                    or not stat.S_ISDIR(current.st_mode)):
                raise ValueError('Cache identity changed before fd-safe removal; retained')
            shutil.rmtree(path.name, dir_fd=parent_fd)
        finally:
            os.close(parent_fd)
    return [dict(domain=record['domain'], path=record['path'], absent_after=True) for record in records]


def cleanup_caches(root, manifest):
    evidence = cleanup_cache_records(manifest.get('caches', []), cache_homes(root),
                                    {manifest['bundle_id'], manifest['payload_id']}, root)
    (root / 'cache-cleanup.json').write_text(json.dumps(evidence, indent=2) + '\n')


def fixture_environment(root):
    root = Path(root).resolve()
    env = {'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'LANG': 'en_US.UTF-8'}
    for key, name in (('HOME', 'home'), ('CFFIXED_USER_HOME', 'home'), ('TMPDIR', 'tmp'),
                      ('XDG_CONFIG_HOME', 'config'), ('XDG_DATA_HOME', 'data'),
                      ('XDG_STATE_HOME', 'state'), ('XDG_CACHE_HOME', 'cache'),
                      ('XDG_RUNTIME_DIR', 'runtime')):
        path = owned_path(root, root / name)
        path.mkdir(mode=0o700, exist_ok=True)
        if path.stat().st_mode & 0o777 != 0o700:
            raise ValueError(f'Non-private fixture directory: {path}')
        env[key] = str(path)
    for name in ('Library/Caches', 'Library/Preferences', 'Library/Application Support'):
        (root / 'home' / name).mkdir(mode=0o700, parents=True, exist_ok=True)
    return env


def sparkle_error(payload, code):
    return payload.get('domain') == 'SUSparkleErrorDomain' and payload.get('code') == code


def expected_abort(case, payload):
    # Sparkle 2.10.1 SUErrors.h, SUUpdateValidator, AppInstaller and
    # SPUInstallerDriver define these chains; code 10 is SPUInstallerError.
    underlying = payload.get('underlying', [])
    codes = [item.get('code') for item in underlying]
    if any(item.get('domain') != 'SUSparkleErrorDomain' for item in underlying):
        return False
    if case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key'):
        outer, chain = 1000, [3002]
    elif case in ('altered-zip', 'wrong-archive-key'):
        outer, chain = 4005, [4005, 3002]
    elif case in ('wrong-name-id', 'same-name-wrong-id'):
        outer, chain = 4005, [10, 3002]
    else:
        outer, chain = 4005, [10, 4006]
    if not sparkle_error(payload, outer) or codes != chain:
        return False
    if case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key', 'altered-zip', 'wrong-archive-key'):
        if case in ('altered-zip', 'wrong-archive-key') and not underlying[0].get('message', '').startswith(
                '(Ed)DSA signature validation before unarchiving failed for update '):
            return False
        return underlying[-1].get('message', '').startswith('EdDSA signature does not match.')
    if case in ('wrong-name-id', 'same-name-wrong-id'):
        return underlying[-1].get('message') == 'No suitable install is found in the update. The update will be rejected.'
    return True


def assess(case, recorded, unchanged, launches, old_exited, archive_requests):
    names = [item['event'].removeprefix('sparkle://') for item in recorded]
    aborts = [item.get('payload', {}) for item in recorded
              if item['event'] == 'sparkle://did-abort-with-error']
    if case in ('valid-zip', 'valid-dmg', 'same-name-wrong-id'):
        if aborts and case != 'same-name-wrong-id':
            raise ValueError('Native updater aborted a valid case')
        if old_exited and launches == 2 and not unchanged and not aborts:
            return 'observed-identity-rotation' if case == 'same-name-wrong-id' else 'passed'
        if case != 'same-name-wrong-id' or not aborts:
            return None
    if not unchanged or launches > 1 or old_exited:
        raise ValueError('Negative case changed or relaunched the installed app')
    forbidden = ('will-install-update', 'will-relaunch-application', 'will-install-update-on-quit')
    install_choice = any(item['event'] == 'sparkle://user-did-make-choice'
                         and item.get('payload', {}).get('choice') == 'install'
                         and item.get('payload', {}).get('stage') in ('downloaded', 'installing')
                         for item in recorded)
    if any(name in names for name in forbidden) or install_choice:
        raise ValueError('Rejected payload advanced to installation or relaunch')
    if case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key', 'downgrade'):
        if archive_requests or 'will-download-update' in names:
            raise ValueError('Rejected feed/selection reached archive download')
    if case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key') and 'did-finish-loading-appcast' in names:
        raise ValueError('Invalid signed feed reached appcast selection')
    if case == 'downgrade':
        outcomes = [item.get('payload', {}) for item in recorded
                    if item['event'] == 'sparkle://did-not-find-update']
        reason = lambda item: item.get('reason') == 'onNewerThanLatestVersion' and item.get('reasonCode') == 2
        if any(not sparkle_error(item, 1001) or not reason(item.get('noUpdate', {})) for item in aborts):
            raise ValueError('Unexpected downgrade abort')
        if outcomes:
            if not all(reason(item) for item in outcomes) or 'did-finish-loading-appcast' not in names:
                raise ValueError('Unexpected no-update selection reason/stage')
            return 'passed'
        return None
    if not aborts:
        return None
    if not all(expected_abort(case, item) for item in aborts):
        raise ValueError('Unexpected native abort domain/code/cause')
    if case not in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key'):
        required = ('did-finish-loading-appcast', 'did-find-valid-update',
                    'will-download-update', 'did-download-update', 'will-extract-update',
                    'did-abort-with-error')
        positions = [names.index(name) if name in names else -1 for name in required]
        if not archive_requests or -1 in positions or positions != sorted(positions):
            raise ValueError('Archive refusal lacks ordered download/validation evidence')
        # 2.10.1 SPUInstallerDriver completes launcher setup before installer messages;
        # did-extract-update is that early callback, not proof of actual unarchiving.
        # Signature cases prove pre-unarchive refusal through their exact cause above;
        # validly signed identity/version cases may unarchive to inspect metadata.
    return 'observed-rejection' if case == 'same-name-wrong-id' else 'passed'


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def installed_state(app):
    return {'executable': digest(Path(app) / 'Contents/MacOS/sparkle_fixture'),
            'plist': digest(Path(app) / 'Contents/Info.plist')}


def command(args, root, *, data=None, timeout=180):
    # Output goes to a regular evidence file, with a hard size bound after completion.
    log = root / 'tools.log'
    with log.open('ab') as output:
        result = subprocess.run([str(arg) for arg in args], input=data, stdout=output,
                                stderr=output, timeout=timeout, check=False)
    if log.stat().st_size > LIMIT:
        raise ValueError(f'Tool output exceeds bound; inspect {log}')
    if result.returncode:
        raise ValueError(f'{args[0]} exited {result.returncode}; inspect {log}')


def disposable_key(root, name, openssl):
    private, public = root / f'{name}.der', root / f'{name}.public.der'
    command([openssl, 'genpkey', '-algorithm', 'ED25519', '-outform', 'DER', '-out', private], root)
    private.chmod(0o600)
    command([openssl, 'pkey', '-inform', 'DER', '-in', private, '-pubout', '-outform', 'DER', '-out', public], root)
    encoded = private.read_bytes()
    if len(encoded) != 48 or not encoded.startswith(bytes.fromhex('302e020100300506032b657004220420')):
        raise ValueError('Unexpected OpenSSL Ed25519 seed encoding')
    raw_public = public.read_bytes()
    if len(raw_public) != 44 or not raw_public.startswith(bytes.fromhex('302a300506032b6570032100')):
        raise ValueError('Unexpected OpenSSL Ed25519 public encoding')
    return base64.b64encode(encoded[-32:]), base64.b64encode(raw_public[-32:]).decode()


def signed_feed_content(data):
    match = re.fullmatch(rb'(.+)<!-- sparkle-signatures:\nedSignature: [A-Za-z0-9+/=]+\nlength: ([0-9]+)\n-->\s*', data, re.DOTALL)
    if not match or len(match[1]) != int(match[2]):
        raise ValueError('Invalid official signed feed length')
    return match[1]


def signature_result(output, path):
    result = output.decode().strip()
    if not result and path.suffix == '.xml':
        match = re.search(rb'<!-- sparkle-signatures:\nedSignature: ([A-Za-z0-9+/=]+)\n', path.read_bytes())
        if match:
            result = match[1].decode()
    if len(base64.b64decode(result, validate=True)) != 64:
        raise ValueError('Invalid SDK signature result')
    return result


def signature(tool, path, key):
    output = subprocess.run([str(tool), '--ed-key-file', '-', '-p', str(path)],
                            input=key, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            timeout=120, check=True)
    if len(output.stdout) > 4096 or len(output.stderr) > LIMIT:
        raise ValueError('Signature tool output exceeds bound')
    return signature_result(output.stdout, path)


def bundle(root, destination, host, sdk, identity, version, bundle_id, feed, public_key, environment, check_mode, feed_failure_age_days=0):
    destination.mkdir(parents=True)
    contents = destination / 'Contents'
    (contents / 'MacOS').mkdir(parents=True)
    (contents / 'Frameworks').mkdir()
    shutil.copy2(host, contents / 'MacOS/sparkle_fixture')
    framework = contents / 'Frameworks/Sparkle.framework'
    shutil.copytree(sdk / 'Sparkle.framework', framework, symlinks=True)
    temp, owner = native_temp(root)
    (contents / 'Resources').mkdir()
    shutil.copy2(sdk / 'LICENSE', contents / 'Resources/Sparkle-LICENSE.txt')
    helper = module('fixture_framework', 'sparkle_framework.py')
    info = dict(CFBundleExecutable='sparkle_fixture', CFBundleIdentifier=bundle_id,
                CFBundleName=destination.stem, CFBundlePackageType='APPL',
                CFBundleVersion=version, CFBundleShortVersionString=version,
                LSMinimumSystemVersion='14.2', NSHighResolutionCapable=True,
                SUFeedURL=feed, SUPublicEDKey=public_key,
                SUEnableAutomaticChecks=False, SUEnableInstallerLauncherService=False,
                SUEnableDownloaderService=False, SpaceportFixtureRoot=str(root),
                SpaceportFixtureID=bundle_id, SpaceportFixtureNativeTemp=str(temp),
                SpaceportFixtureOwner=str(owner), SpaceportFixtureCheckMode=check_mode,
                SpaceportFixtureFailedFeedAgeDays=str(feed_failure_age_days), LSEnvironment=environment,
                NSAppTransportSecurity={'NSAllowsLocalNetworking': True})
    info.update(helper.POLICY)
    (contents / 'Info.plist').write_bytes(plistlib.dumps(info))
    helper.sign_framework(framework, ['--force', '--sign', identity])
    command(['codesign', '--force', '--sign', identity, '--options', 'runtime',
             '--timestamp', destination], root)
    command(['codesign', '--verify', '--deep', '--strict', destination], root)


def prepare(args):
    if sys.platform != 'darwin':
        raise ValueError('Native signed fixtures require macOS')
    sdk, host = args.sdk.resolve(), args.host.resolve()
    if not host.is_file() or host.is_symlink():
        raise ValueError('Build the explicit sparkle_fixture example first')
    module('fixture_framework', 'sparkle_framework.py').validate_sdk(sdk)
    parent = args.evidence.resolve()
    if not parent.is_relative_to(Path(__file__).resolve().parents[2] / 'target'):
        raise ValueError('Evidence must remain outside Git under this worktree target/')
    parent.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix=f'{args.case}-', dir=parent)).resolve()
    root.chmod(0o700)
    print(json.dumps({'status': 'preparing', 'root': str(root), 'case': args.case}), flush=True)
    env = fixture_environment(root)
    (root / 'launches').mkdir(mode=0o700)
    (root / 'served').mkdir(mode=0o700)
    with socket.socket() as probe:
        probe.bind(('127.0.0.1', 0))
        port = probe.getsockname()[1]
    feed = f'http://127.0.0.1:{port}/appcast.xml'
    bundle_id = 'ai.2389.spaceport.fixture.' + uuid.uuid4().hex
    private, public = disposable_key(root, 'trusted', args.openssl)
    other_private, other_public = disposable_key(root, 'other', args.openssl)
    old_version, payload_version, offered_version = case_versions(args.case)
    installed = root / 'installed/Spaceport Fixture.app'
    payload_id = 'ai.2389.spaceport.fixture.' + uuid.uuid4().hex if args.case in ('wrong-name-id', 'same-name-wrong-id') else bundle_id
    payload_name = 'Other Fixture.app' if args.case == 'wrong-name-id' else installed.name
    bundle(root, installed, host, sdk, args.identity, old_version, bundle_id, feed, public, env, case_check_mode(args.case), 21 if args.case == 'aged-wrong-feed-key' else 0)
    payload = root / 'payload' / payload_name
    bundle(root, payload, host, sdk, args.identity, payload_version, payload_id, feed, public, env, case_check_mode(args.case), 21 if args.case == 'aged-wrong-feed-key' else 0)
    archive = root / 'served' / ('update.dmg' if args.case == 'valid-dmg' else 'update.zip')
    if args.case == 'valid-dmg':
        command(['hdiutil', 'create', '-quiet', '-format', 'UDZO', '-srcfolder', payload.parent, archive], root)
        command(['codesign', '--sign', args.identity, '--timestamp', archive], root)
    else:
        command(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', payload, archive], root)
    tool = sdk / 'bin/sign_update'
    archive_key = other_private if args.case == 'wrong-archive-key' else private
    archive_signature = signature(tool, archive, archive_key)
    verifier = module('fixture_feed', 'sparkle-feed.py')
    verifier.OPENSSL = args.openssl
    verifier.verify_signature(archive, archive_signature, other_public if args.case == 'wrong-archive-key' else public)
    if args.case == 'altered-zip':
        with archive.open('ab') as stream:
            stream.write(b'fixture-tamper')
    rss = ET.Element('rss', {'version': '2.0'})
    channel = ET.SubElement(rss, 'channel')
    ET.SubElement(channel, 'title').text = 'Isolated Sparkle fixture'
    item = ET.SubElement(channel, 'item')
    ET.SubElement(item, 'title').text = 'Native fixture update'
    ET.SubElement(item, 'enclosure', {
        'url': f'http://127.0.0.1:{port}/{archive.name}', 'length': str(archive.stat().st_size),
        'type': 'application/octet-stream', f'{{{NS}}}version': offered_version,
        f'{{{NS}}}shortVersionString': offered_version, f'{{{NS}}}edSignature': archive_signature})
    xml = root / 'served/appcast.xml'
    unsigned = ET.tostring(rss, encoding='utf-8', xml_declaration=True) + b'\n'
    xml.write_bytes(unsigned)
    feed_key = other_private if args.case in ('wrong-feed-key', 'aged-wrong-feed-key') else private
    feed_signature = signature(tool, xml, feed_key)
    verifier.verify_signature(signed_feed_content(xml.read_bytes()), feed_signature, other_public if args.case in ('wrong-feed-key', 'aged-wrong-feed-key') else public)
    if b'sparkle-signatures:' not in xml.read_bytes():
        raise ValueError('Official SDK did not append signed feed metadata')
    if args.case == 'altered-feed':
        xml.write_bytes(xml.read_bytes().replace(b'Native fixture update', b'Native fixture tamper', 1))
    executable = installed / 'Contents/MacOS/sparkle_fixture'
    manifest = dict(root=str(root), case=args.case, app=str(installed), executable=str(executable),
                    bundle_id=bundle_id, payload_id=payload_id, old_version=old_version,
                    payload_version=payload_version, port=port, archive=archive.name,
                    before=installed_state(installed), identity=args.identity, environment=env,
                    archive_sha256=digest(archive), check_mode=case_check_mode(args.case))
    (root / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    return root


class FixtureServer(http.server.ThreadingHTTPServer):
    daemon_threads = True


def serve(root, manifest):
    requests = []
    allowlist = {'/appcast.xml': root / 'served/appcast.xml',
                 '/' + manifest['archive']: root / 'served' / manifest['archive']}

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            path = allowlist.get(self.path)
            if len(requests) >= 1024:
                self.send_error(429)
                return
            requests.append(self.path)
            if path is None or path.is_symlink():
                self.send_error(404)
                return
            self.send_response(200)
            self.send_header('Content-Length', str(path.stat().st_size))
            self.send_header('Content-Type', 'application/xml' if path.suffix == '.xml' else 'application/octet-stream')
            self.end_headers()
            try:
                with path.open('rb') as source:
                    shutil.copyfileobj(source, self.wfile, 65536)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def log_message(self, *_):
            pass

    server = FixtureServer(('127.0.0.1', manifest['port']), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, requests


def relaunch_ready(recorded, pid):
    return any(event.get('pid') == pid and event.get('event') == 'fixture://ready' for event in recorded)


def read_launches(root):
    paths = list((root / 'launches').glob('*.json'))
    if len(paths) > 8:
        raise ValueError('Unexpected fixture launch count')
    return [json.loads(path.read_text()) for path in paths]


def events(root):
    path = root / 'events.jsonl'
    if not path.exists():
        return []
    if path.stat().st_size > LIMIT:
        raise ValueError('Event evidence exceeds bound')
    lines = path.read_text().splitlines()
    # A writer may be appending its last record while the driver reads.
    if lines and not path.read_bytes().endswith(b'\n'):
        lines.pop()
    return [json.loads(line) for line in lines]


def process_path(pid):
    result = subprocess.run(['/bin/ps', '-p', str(pid), '-o', 'comm='],
                            capture_output=True, text=True, timeout=5, check=False)
    return result.stdout.strip() if result.returncode == 0 else None


def request_quit(root):
    raw = Path(root) / 'quit'
    if raw.is_symlink():
        raise ValueError('Quit sentinel must not be symlinked')
    path = owned_path(root, raw)
    with path.open('x') as output:
        path.chmod(0o600)
        output.write('quit\n')


def cleanup(root, manifest, child):
    request_quit(root)
    expected = str(Path(manifest['executable']).resolve())
    stop = time.monotonic() + 5
    while time.monotonic() < stop:
        markers_alive = any(process_path(launch['pid']) == expected for launch in read_launches(root))
        if not markers_alive and (child is None or child.poll() is not None):
            return {'quit_sentinel': True, 'graceful_exit': True}
        time.sleep(0.1)
    # Only the unreaped original Popen child may receive a signal. Relaunched
    # processes receive the private sentinel; PID/path observations never authorize signals.
    if child is not None and child.poll() is None:
        # Popen is the exact created process, including before a launch marker exists.
        child.terminate()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=5)
    if any(process_path(launch['pid']) == expected for launch in read_launches(root)):
        raise ValueError('Relaunched fixture did not quit; preferences and evidence retained')
    return {'quit_sentinel': True, 'graceful_exit': False, 'owned_child_fallback': True}


def final_result(result, cleanup_error=None):
    return dict(result, status='failed', cleanup_error=cleanup_error) if cleanup_error else result


def run(root, deadline):
    root = root.resolve()
    manifest = json.loads((root / 'manifest.json').read_text())
    if root != Path(manifest['root']) or root.stat().st_mode & 0o777 != 0o700:
        raise ValueError('Invalid private prepared root')
    executable = owned_path(root, manifest['executable'])
    if read_launches(root):
        raise ValueError('Prepared case has already launched; create a fresh case')
    claim_caches(root, manifest)
    try:
        claim_preferences(root, manifest)
        server, requests = serve(root, manifest)
    except (ValueError, OSError, subprocess.SubprocessError):
        rollback_cache_records(manifest['caches'], cache_homes(root),
                               {manifest['bundle_id'], manifest['payload_id']})
        raise
    child = None
    result = {'status': 'failed', 'root': str(root), 'case': manifest['case'],
              'check_mode': manifest.get('check_mode', 'manual')}
    try:
        with (root / 'host.log').open('wb') as output:
            child = subprocess.Popen([str(executable)], env=manifest['environment'],
                                     stdout=output, stderr=output)
        state = dict(result, status='checking-native-information' if manifest.get('check_mode') == 'information' else 'waiting-for-native-ui', pid=child.pid,
                     app=manifest['app'], deadline_seconds=deadline)
        print(json.dumps(state), flush=True)
        (root / 'waiting.json').write_text(json.dumps(state, indent=2) + '\n')
        stop = time.monotonic() + deadline
        while time.monotonic() < stop:
            launches = read_launches(root)
            recorded = events(root)
            for record in manifest['preferences']:
                preferences_private(Path(record['path']))
            try:
                unchanged = installed_state(manifest['app']) == manifest['before']
            except FileNotFoundError:
                # Replacement can briefly remove the old bundle before renaming the new one.
                time.sleep(0.25)
                continue
            status = assess(manifest['case'], recorded, unchanged, len(launches),
                            child.poll() is not None, requests.count('/' + manifest['archive']))
            if status:
                if manifest['case'] == 'aged-wrong-feed-key' and (len(launches) != 1 or not launches[0].get('aged_feed_failure_seeded')):
                    raise ValueError('Aged feed case lacks the actual native NSDate seed proof')
                if len(launches) == 2:
                    expected_id = manifest['payload_id']
                    latest = next(launch for launch in launches if launch['pid'] != child.pid)
                    if (latest['version'] != manifest['payload_version'] or latest['bundle_id'] != expected_id
                            or latest['executable'] != str(executable)):
                        raise ValueError('Relaunch identity/version/path mismatch')
                    if not relaunch_ready(recorded, latest['pid']):
                        time.sleep(0.25)
                        continue
                    command(['codesign', '--verify', '--deep', '--strict', manifest['app']], root)
                result['status'] = status
                break
            if child.poll() is not None and not launches:
                raise ValueError('Host exited before isolation/startup marker; inspect host.log')
            if (root / 'host.log').stat().st_size > LIMIT:
                raise ValueError('Host output exceeds bound')
            time.sleep(0.25)
        else:
            raise ValueError('Case deadline expired without required native evidence')
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        result['error'] = str(error)
        raise
    finally:
        diagnostic_errors = []
        try:
            # Diagnostic failures must never skip exact-child and preference cleanup.
            try:
                server.shutdown()
                server.server_close()
                (root / 'requests.json').write_text(json.dumps(requests, indent=2) + '\n')
            except (ValueError, OSError, subprocess.SubprocessError) as error:
                diagnostic_errors.append(f'requests/server: {error}')
            try:
                result['installed_after'] = installed_state(manifest['app'])
            except (ValueError, OSError) as error:
                diagnostic_errors.append(f'installed_after: {error}')
            result['archive_requests'] = requests.count('/' + manifest['archive'])
            result['old_pid_exited_before_cleanup'] = child is not None and child.poll() is not None
        finally:
            cleanup_error = None
            try:
                result.update(cleanup(root, manifest, child))
                cleanup_caches(root, manifest)
                cleanup_preferences(root, manifest)
            except (ValueError, OSError, subprocess.SubprocessError) as error:
                cleanup_error = str(error)
                raise
            finally:
                if diagnostic_errors:
                    result.update(status='failed', diagnostic_errors=diagnostic_errors)
                result = final_result(result, cleanup_error)
                (root / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result), flush=True)
    if result['status'] == 'failed':
        raise ValueError('Native evidence collection failed; inspect result.json')


def main():
    parser = argparse.ArgumentParser(description='Real signed Sparkle fixtures. Feed/key/downgrade cases use read-only native information checks; install cases require native buttons. Evidence retained on every outcome.')
    parser.add_argument('action', choices=('prepare', 'run'))
    parser.add_argument('--case', choices=CASES, default='valid-zip')
    parser.add_argument('--prepared', type=Path, help='Run an existing never-launched prepared case')
    parser.add_argument('--sdk', type=Path, help='Pinned Sparkle 2.10.1 SDK directory')
    parser.add_argument('--host', type=Path, help='Compiled GPUI sparkle_fixture example executable')
    parser.add_argument('--identity', default=IDENTITY, help='Existing Developer ID SHA1; no Keychain changes')
    parser.add_argument('--openssl', default='/opt/homebrew/opt/openssl@3/bin/openssl', help='OpenSSL 3 executable')
    parser.add_argument('--evidence', type=Path, default=Path(__file__).resolve().parents[2] / 'target/sparkle-native')
    parser.add_argument('--deadline', type=int, default=240, help='Bounded native interaction deadline, 10–900 seconds')
    args = parser.parse_args()
    if not 10 <= args.deadline <= 900:
        parser.error('--deadline must be 10–900 seconds')
    if args.prepared and args.action != 'run':
        parser.error('--prepared requires run')
    if not args.prepared and (not args.sdk or not args.host):
        parser.error('--sdk and --host are required for a new case')
    root = args.prepared.resolve() if args.prepared else prepare(args)
    print(json.dumps({'status': 'prepared', 'root': str(root), 'case': json.loads((root / 'manifest.json').read_text())['case']}), flush=True)
    if args.action == 'run':
        run(root, args.deadline)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        sys.exit(f'Sparkle native fixture: {error}. Retained evidence: use the printed fixture root.')
