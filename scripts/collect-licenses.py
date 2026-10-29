import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent
cargo = [str(ROOT / 'scripts/cargo.sh')] if os.name != 'nt' else ['cargo']
result = subprocess.run(cargo + ['metadata', '--locked', '--format-version', '1', '--features', 'updater,bundled-sdl'], cwd=ROOT, check=True, capture_output=True, text=True)
packages = json.loads(result.stdout)['packages']
destination = ROOT / 'packaging/licenses'
destination.mkdir(parents=True, exist_ok=True)
index = []
for package in sorted(packages, key=lambda item: (item['name'], item['version'])):
    if package['name'] == 'cinnamoroll-desktop':
        continue
    folder = Path(package['manifest_path']).parent
    files = [path for path in folder.iterdir() if path.is_file() and path.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE', 'UNLICENSE'))]
    if package.get('license_file'):
        path = folder / package['license_file']
        if path.is_file() and path.resolve().is_relative_to(folder.resolve()):
            files.append(path)
    copied = []
    for path in sorted(set(files)):
        target = destination / f"{package['name']}-{package['version']}" / path.name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)
        copied.append(path.name)
    index.append({'name': package['name'], 'version': package['version'], 'license': package['license'], 'files': copied})
(destination / 'index.json').write_text(json.dumps(index, indent=2) + '\n', encoding='utf-8')
print(f'Collected license files and SPDX metadata for {len(index)} dependencies.')
