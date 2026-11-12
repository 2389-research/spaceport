#!/usr/bin/env python3
# ABOUTME: Validate the pinned Sparkle SDK and preview bundle update policy.
# ABOUTME: Sign only the SDK's known nested code before its enclosing framework.
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tarfile
import tempfile

VERSION = '2.10.1'
SHA256 = '1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27'
FEED = 'https://github.com/2389-research/spaceport/releases/download/macos-preview-feed/appcast.xml'
CONFIG = Path(__file__).with_name('sparkle.json')
CODE = (
    'Autoupdate',
    'Updater.app/Contents/MacOS/Updater',
    'XPCServices/Downloader.xpc/Contents/MacOS/Downloader',
    'XPCServices/Installer.xpc/Contents/MacOS/Installer',
    'Sparkle',
)
SIGN_ORDER = (
    'Autoupdate',
    'Updater.app/Contents/MacOS/Updater', 'Updater.app',
    'XPCServices/Downloader.xpc/Contents/MacOS/Downloader', 'XPCServices/Downloader.xpc',
    'XPCServices/Installer.xpc/Contents/MacOS/Installer', 'XPCServices/Installer.xpc',
    'Sparkle',
)
POLICY = dict(SURequireSignedFeed=True, SUSignedFeedFailureExpirationInterval=0,
              SUVerifyUpdateBeforeExtraction=True,
              SUAutomaticallyUpdate=False)


def read_config(path=CONFIG):
    data = json.loads(Path(path).read_text())
    if data.get('sdk_version') != VERSION or data.get('sdk_sha256') != SHA256:
        raise ValueError('Sparkle SDK version/checksum configuration mismatch')
    if data.get('feed_url') != FEED:
        raise ValueError('Sparkle preview feed URL mismatch')
    try:
        key = base64.b64decode(data.get('public_key', ''), validate=True)
    except (ValueError, TypeError) as error:
        raise ValueError('Invalid Sparkle public key') from error
    if len(key) != 32:
        raise ValueError('Sparkle public key must contain 32 bytes')
    return data


def verify_archive(archive, config=CONFIG):
    read_config(config)
    if not archive.is_file() or archive.is_symlink():
        raise ValueError('Pinned Sparkle SDK archive required beside framework')
    digest = hashlib.sha256()
    with archive.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            digest.update(block)
    if digest.hexdigest() != SHA256:
        raise ValueError('Sparkle SDK archive checksum mismatch')


def acquire_sdk(output, archive=None, config=CONFIG):
    read_config(config)
    output = Path(output)
    if output.exists() or output.is_symlink():
        raise ValueError(f'Sparkle SDK output already exists: {output}')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.sparkle-sdk-', dir=output.parent) as temporary:
        stage = Path(temporary) / 'sdk'
        stage.mkdir()
        pinned = stage / f'Sparkle-{VERSION}.tar.xz'
        if archive is None:
            subprocess.run(['curl', '--fail', '--location', '--proto', '=https',
                            '--tlsv1.2', '--output', str(pinned),
                            f'https://github.com/sparkle-project/Sparkle/releases/download/{VERSION}/{pinned.name}'], check=True)
        else:
            shutil.copyfile(archive, pinned)
        # No archive member is extracted until its complete bytes match the pin.
        verify_archive(pinned, config)
        subprocess.run(['tar', '-xf', str(pinned), '-C', str(stage)], check=True)
        validate_sdk(stage, config)
        validate_framework(stage / 'Sparkle.framework', 'universal')
        if output.exists() or output.is_symlink():
            raise ValueError(f'Sparkle SDK output already exists: {output}')
        stage.rename(output)
    print(output.resolve())


def validate_sdk(directory, config=CONFIG):
    directory = Path(directory)
    archive = directory / f'Sparkle-{VERSION}.tar.xz'
    verify_archive(archive, config)
    validate_license(directory / 'LICENSE', archive)
    framework = directory / 'Sparkle.framework'
    validate_framework_bytes(framework, archive)
    return framework


def validate_license(path, archive):
    # Sparkle's root SDK license also carries licenses for its bundled code.
    # It is outside Sparkle.framework, so preserve and verify it separately.
    with tarfile.open(archive) as source:
        member = next((entry for entry in source
                       if entry.name.removeprefix('./') == 'LICENSE'), None)
        if (member is None or not member.isfile() or not path.is_file()
                or path.is_symlink() or path.stat().st_size != member.size
                or path.read_bytes() != source.extractfile(member).read()):
            raise ValueError(f'Tampered or missing Sparkle LICENSE: {path}')


def validate_framework_bytes(framework, archive):
    expected = set()
    # Compare the complete source tree with the checksum-pinned archive. An
    # adjacent archive alone is insufficient proof of the extracted SDK bytes.
    with tarfile.open(archive) as source:
        for member in source:
            name = member.name.removeprefix('./')
            if not name.startswith('Sparkle.framework/'):
                continue
            relative = name.removeprefix('Sparkle.framework/').rstrip('/')
            if not relative:
                continue
            expected.add(relative)
            expected.update(str(parent) for parent in Path(relative).parents if str(parent) != '.')
            path = framework / relative
            if member.issym():
                valid = path.is_symlink() and os.readlink(path) == member.linkname
            elif member.isdir():
                valid = path.is_dir() and not path.is_symlink()
            elif member.isfile():
                valid = (path.is_file() and not path.is_symlink()
                         and path.stat().st_size == member.size
                         and path.read_bytes() == source.extractfile(member).read()
                         and bool(path.stat().st_mode & 0o111) == bool(member.mode & 0o111))
            else:
                valid = False
            if not valid:
                raise ValueError(f'Tampered Sparkle SDK framework member: {relative}')
    actual = {str(path.relative_to(framework)) for path in framework.rglob('*')}
    if not expected or actual != expected:
        raise ValueError('Unexpected or missing Sparkle SDK framework members')


def validate_framework(framework, architecture):
    framework = Path(framework)
    base = framework / 'Versions/B'
    if framework.is_symlink() or not base.is_dir() or base.is_symlink():
        raise ValueError('Malformed Sparkle framework version directory')
    links = {'Versions/Current': 'B'}
    links.update({name: f'Versions/Current/{name}' for name in (
        'Autoupdate', 'Headers', 'Modules', 'PrivateHeaders', 'Resources',
        'Sparkle', 'Updater.app', 'XPCServices')})
    for name, target in links.items():
        path = framework / name
        if not path.is_symlink() or os.readlink(path) != target:
            raise ValueError(f'Malformed Sparkle framework link: {name}')
    for path in framework.rglob('*'):
        if path.is_symlink() and not path.resolve().is_relative_to(framework.resolve()):
            raise ValueError(f'Escaping Sparkle framework link: {path}')
        if path.is_file() and not path.is_symlink():
            relative = str(path.relative_to(base)) if path.is_relative_to(base) else ''
            with path.open('rb') as source:
                magic = source.read(4)
            macho = magic in (b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\xca\xfe\xba\xbe', b'\xbe\xba\xfe\xca',
                              b'\xce\xfa\xed\xfe', b'\xfe\xed\xfa\xce', b'\xca\xfe\xba\xbf', b'\xbf\xba\xfe\xca')
            if (macho or path.stat().st_mode & 0o111) and relative not in CODE:
                raise ValueError(f'Unexpected Sparkle framework executable: {path}')
    required = {'arm64'} if architecture == 'arm64' else {'arm64', 'x86_64'}
    for name in CODE:
        path = base / name
        if not path.is_file() or path.is_symlink() or not path.stat().st_mode & 0o111:
            raise ValueError(f'Missing Sparkle framework executable: {name}')
        archs = set(subprocess.check_output(['lipo', '-archs', str(path)], text=True).split())
        if not required <= archs or not archs <= {'arm64', 'x86_64'}:
            raise ValueError(f'Sparkle framework architecture mismatch: {name}')
    bundles = {'Resources/Info.plist': ('Sparkle', 'org.sparkle-project.Sparkle'),
               'Updater.app/Contents/Info.plist': ('Updater', 'org.sparkle-project.Sparkle.Updater'),
               'XPCServices/Downloader.xpc/Contents/Info.plist': ('Downloader', 'org.sparkle-project.DownloaderService'),
               'XPCServices/Installer.xpc/Contents/Info.plist': ('Installer', 'org.sparkle-project.InstallerLauncher')}
    for name, (executable, identity) in bundles.items():
        metadata = plistlib.loads((base / name).read_bytes())
        if (metadata.get('CFBundleExecutable') != executable
                or metadata.get('CFBundleIdentifier') != identity
                or metadata.get('CFBundleShortVersionString') != VERSION):
            raise ValueError(f'Sparkle framework metadata mismatch: {name}')


def bundle_policy(config):
    return dict(POLICY, SUFeedURL=config['feed_url'], SUPublicEDKey=config['public_key'])


def configure_bundle(app, config=CONFIG):
    policy = bundle_policy(read_config(config))
    info = Path(app) / 'Contents/Info.plist'
    metadata = plistlib.loads(info.read_bytes())
    if metadata.get('CFBundleIdentifier') != 'ai.2389.spaceport':
        raise ValueError('Sparkle preview bundle identity mismatch')
    metadata.update(policy)
    # Absence permits Sparkle's native consent prompt.
    metadata.pop('SUEnableAutomaticChecks', None)
    info.write_bytes(plistlib.dumps(metadata))


def validate_policy(app, config=CONFIG):
    metadata = plistlib.loads((Path(app) / 'Contents/Info.plist').read_bytes())
    if metadata.get('CFBundleIdentifier') != 'ai.2389.spaceport':
        raise ValueError('Sparkle preview bundle identity mismatch')
    for name, value in bundle_policy(read_config(config)).items():
        if type(metadata.get(name)) is not type(value) or metadata[name] != value:
            raise ValueError(f'Sparkle bundle configuration mismatch: {name}')
    if 'SUEnableAutomaticChecks' in metadata:
        raise ValueError('Sparkle automatic checks require native consent')


def sign_framework(framework, options):
    for name in SIGN_ORDER:
        subprocess.run(['codesign', *options, '--options', 'runtime', '--timestamp',
                        str(framework / 'Versions/B' / name)], check=True)
    subprocess.run(['codesign', *options, '--options', 'runtime', '--timestamp', str(framework)], check=True)
    subprocess.run(['codesign', '--verify', '--deep', '--strict', '--verbose=2', str(framework)], check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__ or 'Validate and package pinned Sparkle framework')
    parser.add_argument('action', choices=('acquire', 'validate-sdk', 'configure', 'validate-bundle', 'validate-unsigned-bundle', 'sign'))
    parser.add_argument('path', type=Path, help='SDK directory or Spaceport.app')
    parser.add_argument('--architecture', choices=('arm64', 'universal'), default='universal')
    parser.add_argument('--config', type=Path, default=CONFIG)
    parser.add_argument('--archive', type=Path, help='Local SDK archive; otherwise download the pinned official release')
    args, options = parser.parse_known_args()
    if args.action != 'sign' and options:
        parser.error('Unexpected signing options')
    if args.action == 'acquire':
        acquire_sdk(args.path, args.archive, args.config)
    elif args.action == 'validate-sdk':
        validate_sdk(args.path, args.config)
        validate_framework(args.path / 'Sparkle.framework', args.architecture)
    elif args.action == 'configure':
        configure_bundle(args.path, args.config)
    else:
        validate_policy(args.path, args.config)
        directory = args.path / 'Contents/Frameworks'
        if (not directory.is_dir() or directory.is_symlink()
                or {path.name for path in directory.iterdir()} != {'Sparkle.framework'}):
            raise ValueError('Unexpected bundled frameworks; Sparkle.framework required')
        framework = directory / 'Sparkle.framework'
        validate_framework(framework, args.architecture)
        if args.action == 'validate-unsigned-bundle':
            sdk = os.environ.get('SPARKLE_FRAMEWORK_PATH')
            if not sdk:
                raise ValueError('SPARKLE_FRAMEWORK_PATH must name the pinned Sparkle SDK directory')
            archive = Path(sdk) / f'Sparkle-{VERSION}.tar.xz'
            verify_archive(archive, args.config)
            validate_license(args.path / 'Contents/Resources/Sparkle-LICENSE.txt', archive)
            validate_framework_bytes(framework, archive)
        if args.action == 'sign':
            sign_framework(framework, options[1:] if options[:1] == ['--'] else options)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError, tarfile.TarError) as error:
        sys.exit(f'Sparkle: {error}')
