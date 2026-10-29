import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parent.parent
TARGETS = {'aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-pc-windows'}


def source_archive(output: Path, version: str) -> Path:
    archive = output / f'Cinnamoroll-{version}-source.zip'
    roots = ['Cargo.toml', 'Cargo.lock', 'build.rs', '.gitignore', 'README.md', 'LICENSE', 'THIRD_PARTY_NOTICES.md', 'src', 'assets', 'scripts', 'packaging', 'docs', '.github']
    with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=6) as package:
        for name in roots:
            root = ROOT / name
            files = [root] if root.is_file() else sorted(root.rglob('*'))
            for path in files:
                if path.is_file() and path.name != '.DS_Store' and path.suffix not in {'.key', '.pem', '.pyc', '.pyo'} and '__pycache__' not in path.parts:
                    package.write(path, Path('cinnamoroll-desktop') / path.relative_to(ROOT))
    return archive


def main() -> None:
    parser = argparse.ArgumentParser(description='Package and verify a signed release without publishing it.')
    parser.add_argument('--target', choices=sorted(TARGETS))
    parser.add_argument('--key', type=Path)
    parser.add_argument('--output', type=Path, default=ROOT / 'dist/releases')
    parser.add_argument('--source-only', action='store_true')
    parser.add_argument('--skip-launch-check', action='store_true', help='Cross-build preparation only; native validation is still required.')
    args = parser.parse_args()
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
    args.output.mkdir(parents=True, exist_ok=True)
    if args.source_only:
        print(source_archive(args.output, version))
        return
    if not args.target or not args.key:
        parser.error('--target and --key are required for a signed app package')
    key = args.key.resolve(strict=True)
    if key.is_relative_to(ROOT):
        parser.error('Keep the private signing key outside the repository')
    if os.name != 'nt' and key.stat().st_mode & 0o077:
        parser.error('The private signing key must be readable only by its owner')
    channel = json.loads((ROOT / 'packaging/update-channel.json').read_text(encoding='utf-8'))
    public = bytes.fromhex(channel['public_key'])
    if len(public) != 32 or key.read_bytes()[32:] != public:
        parser.error('The signing key does not match the configured public key')
    mac = args.target.endswith('apple-darwin')
    root = ROOT / 'dist' / ('Cinnamoroll.app' if mac else 'Cinnamoroll-Windows')
    executable = root / ('Contents/MacOS/Cinnamoroll' if mac else 'Cinnamoroll.exe')
    if not executable.is_file():
        parser.error('Build the portable app before packaging it')
    if not args.skip_launch_check:
        host_target = ('aarch64' if platform.machine().lower() in {'arm64', 'aarch64'} else 'x86_64') + '-apple-darwin' if platform.system() == 'Darwin' else 'x86_64-pc-windows'
        if host_target != args.target:
            parser.error('Native launch verification needs a matching computer')
        result = subprocess.run([str(executable), '--update-config'], check=True, capture_output=True, text=True, timeout=15)
        actual = json.loads(result.stdout)
        if not actual['enabled'] or any(actual.get(field) != value for field, value in {'version': version, **channel}.items()):
            parser.error('The app version or embedded update channel does not match this checkout')
    archive = args.output / f'Cinnamoroll-{version}-{args.target}.zip'
    if archive.exists():
        parser.error(f'Refusing to overwrite {archive}')
    if mac:
        subprocess.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', str(root)], check=True)
        subprocess.run(['/usr/bin/ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', str(root), str(archive)], check=True)
    else:
        with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=6) as package:
            for path in sorted(root.rglob('*')):
                if path.is_file():
                    package.write(path, Path(root.name) / path.relative_to(root))
    zipsign = os.environ.get('ZIPSIGN') or shutil.which('zipsign')
    if not zipsign:
        local = ROOT / '.tools/zipsign/bin/zipsign'
        if local.is_file():
            zipsign = str(local)
        else:
            parser.error('Install zipsign 0.2.1 or set ZIPSIGN to its executable')
    subprocess.run([zipsign, 'sign', 'zip', str(archive), str(key)], check=True)
    with tempfile.TemporaryDirectory() as folder:
        public_path = Path(folder) / 'public.key'
        public_path.write_bytes(public)
        subprocess.run([zipsign, 'verify', 'zip', str(archive), str(public_path)], check=True)
    with zipfile.ZipFile(archive) as package:
        if package.testzip() is not None:
            parser.error('The signed archive failed its integrity check')
    if not args.skip_launch_check:
        subprocess.run([str(executable), '--verify-update', str(archive)], check=True, timeout=60)
    with archive.open('rb') as file:
        checksum = hashlib.file_digest(file, 'sha256').hexdigest()
    archive.with_suffix('.zip.sha256').write_text(f'{checksum}  {archive.name}\n', encoding='utf-8')
    print(f'Prepared signed release: {archive.name}')


if __name__ == '__main__':
    main()
