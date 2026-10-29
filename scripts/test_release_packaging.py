import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
import zipfile

SPEC = importlib.util.spec_from_file_location('release_packaging', Path(__file__).with_name('prepare-release.py'))
PACKAGING = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGING)


class ReleasePackagingTests(unittest.TestCase):
    def test_old_license_timestamps_and_private_work_files(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'packaging/licenses/example').mkdir(parents=True)
            license_file = root / 'packaging/licenses/example/LICENSE'
            license_file.write_text('upstream license', encoding='utf-8')
            os.utime(license_file, (0, 0))
            (root / 'scripts/__pycache__').mkdir(parents=True)
            (root / 'scripts/__pycache__/private.pyc').write_bytes(b'local paths')
            (root / 'packaging/private.key').write_bytes(b'private fixture')
            (root / 'install.sh').write_text('installer fixture', encoding='utf-8')
            previous = PACKAGING.ROOT
            PACKAGING.ROOT = root
            try:
                archive = PACKAGING.source_archive(root, '0.1.1')
            finally:
                PACKAGING.ROOT = previous
            with zipfile.ZipFile(archive) as package:
                names = package.namelist()
                self.assertIsNone(package.testzip())
                self.assertIn('cinnamoroll-desktop/install.sh', names)
                self.assertFalse(any('__pycache__' in name or name.endswith('.key') for name in names))
                entry = package.getinfo('cinnamoroll-desktop/packaging/licenses/example/LICENSE')
                self.assertEqual(entry.date_time[0], 1980)
                self.assertEqual(package.read(entry), b'upstream license')


if __name__ == '__main__':
    unittest.main()
