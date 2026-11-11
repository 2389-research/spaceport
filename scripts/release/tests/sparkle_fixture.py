# ABOUTME: Build inert framework trees for pure Sparkle packaging contract tests.
# ABOUTME: These files cannot run and do not represent native signing evidence.
import importlib.util
from pathlib import Path
import plistlib


def helper():
    spec = importlib.util.spec_from_file_location('sparkle_framework', Path(__file__).resolve().parents[1] / 'sparkle_framework.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def framework_fixture(framework):
    base = framework / 'Versions/B'
    base.mkdir(parents=True)
    links = {'Versions/Current': 'B'}
    links.update({name: f'Versions/Current/{name}' for name in (
        'Autoupdate', 'Headers', 'Modules', 'PrivateHeaders', 'Resources',
        'Sparkle', 'Updater.app', 'XPCServices')})
    for name, target in links.items():
        (framework / name).symlink_to(target)
    for directory in ('Headers', 'Modules', 'PrivateHeaders'):
        (base / directory).mkdir()
    for name in helper().CODE:
        path = base / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b'inert signing fixture')
        path.chmod(0o755)
    bundles = {'Resources/Info.plist': ('Sparkle', 'org.sparkle-project.Sparkle'),
               'Updater.app/Contents/Info.plist': ('Updater', 'org.sparkle-project.Sparkle.Updater'),
               'XPCServices/Downloader.xpc/Contents/Info.plist': ('Downloader', 'org.sparkle-project.DownloaderService'),
               'XPCServices/Installer.xpc/Contents/Info.plist': ('Installer', 'org.sparkle-project.InstallerLauncher')}
    for name, (executable, identity) in bundles.items():
        path = base / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(plistlib.dumps(dict(CFBundleExecutable=executable,
                                            CFBundleIdentifier=identity,
                                            CFBundleShortVersionString='2.10.1')))


def sdk_fixture(scripts, sdk):
    """Use a fixture checksum in a private helper copy, never the production pin."""
    import hashlib
    import json
    import tarfile
    sdk.mkdir()
    framework_fixture(sdk / 'Sparkle.framework')
    (sdk / 'LICENSE').write_bytes(b'Sparkle fixture copyright and license text\n')
    archive = sdk / 'Sparkle-2.10.1.tar.xz'
    with tarfile.open(archive, 'w:xz') as target:
        target.add(sdk / 'Sparkle.framework', arcname='Sparkle.framework')
        target.add(sdk / 'LICENSE', arcname='LICENSE')
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    production = Path(__file__).resolve().parents[1]
    code = (production / 'sparkle_framework.py').read_text()
    (scripts / 'sparkle_framework.py').write_text(code.replace(helper().SHA256, digest))
    config = helper().read_config()
    config['sdk_sha256'] = digest
    (scripts / 'sparkle.json').write_text(json.dumps(config))
