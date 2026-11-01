import importlib.util
from pathlib import Path
import stat
import struct
import tempfile
import unittest
import zipfile
import zlib

spec = importlib.util.spec_from_file_location('preflight', Path(__file__).with_name('installed-release-archive.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
EXE = 'Script Kit.app/Contents/MacOS/script-kit-gpui'


class ArchivePreflight(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.TemporaryDirectory()
        self.addCleanup(self.root.cleanup)
        self.path = Path(self.root.name) / 'fixture.zip'

    def make(self, *entries):
        with zipfile.ZipFile(self.path, 'w') as z:
            z.writestr(EXE, 'only a synthetic file, never executable')
            for entry in entries:
                z.writestr(entry, '')

    def test_good(self):
        self.make('__MACOSX/Script Kit.app/._Contents')
        module.inspect(self.path)

    def test_aliases_and_unsafe_names_including_metadata(self):
        for name in ['../escape', '/tmp/escape', 'Script Kit.app/Contents/../foo',
                     'Script Kit.app/Contents\\other', 'Script Kit.app/a:b', EXE.upper(),
                     'Script Kit.app/contents/not-the-same-dir', EXE+'/child',
                     '__MACOSX/../escape', '__MACOSX/Script Kit.app/not-appledouble']:
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.make(name); module.inspect(self.path)

    def test_links_and_path_extra(self):
        payload = struct.pack('<BI', 1, zlib.crc32(b'Script Kit.app/extra')) + b'../overridden'
        new_path = struct.pack('<HH', 0x7075, len(payload)) + payload
        for mode, extra in [(stat.S_IFLNK, b''), (stat.S_IFIFO, b''), (stat.S_IFREG, new_path)]:
            with self.subTest(mode=mode), self.assertRaises(ValueError):
                i = zipfile.ZipInfo('Script Kit.app/extra'); i.external_attr = mode << 16; i.extra = extra
                self.make(i); module.inspect(self.path)

    def test_spliced_local_bytes_or_disagreeing_local_path(self):
        self.make(); original = self.path.read_bytes()
        for data in [b'PK\x03\x04unindexed' + original, original.replace(EXE.encode(), b'X' * len(EXE), 1)]:
            with self.assertRaises(ValueError):
                self.path.write_bytes(data); module.inspect(self.path)


if __name__ == '__main__':
    unittest.main()
