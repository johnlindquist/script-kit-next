"""Read-only preflight for the ditto ZIP accepted by the installed-release host.

Do not use zipinfo's line-oriented filenames as an extraction boundary. Inspect
both record sets (including AppleDouble), reject links, and ensure there are no
unindexed local records that a streaming extractor could interpret instead.
"""
import os
import stat
import struct
import sys
import unicodedata
import zipfile


def require(ok):
    if not ok:
        raise ValueError('unsupported or unsafe release ZIP')


def extras(data):
    # ditto / ordinary ZIP timestamps and Unix uid/gid; no pathname overrides.
    while data:
        require(len(data) >= 4)
        kind, size = struct.unpack_from('<HH', data)
        require(kind in (0x5455, 0x5855, 0x7855, 0x7875) and size <= len(data) - 4)
        data = data[4 + size:]


def inspect(path):
    size = os.stat(path).st_size
    require(22 <= size <= 768 * 1024**2)
    # The publisher uses ditto: no split, ZIP64, suffixes or comments. Bound the
    # central directory before Python allocates or parses it.
    with open(path, 'rb') as fp:
        fp.seek(-22, os.SEEK_END)
        sig, disk, central_disk, here, total, central_size, start, comment = struct.unpack('<I4H2IH', fp.read(22))
        require(sig == 0x06054b50 and disk == central_disk == comment == 0 and 0 < here == total <= 20000 and
                central_size <= 16 * 1024**2 and start + central_size == size - 22)
    with zipfile.ZipFile(path) as archive:
        items = sorted(archive.infolist(), key=lambda i: i.header_offset)
        require(0 < len(items) <= 20000 and sum(i.file_size for i in items) <= 2 * 1024**3)
        seen, components, apple_double, pos = set(), {}, [], 0
        for i in items:
            require(i.create_system in (3, 19) and i.header_offset == pos and not i.flag_bits & ~0x80e and
                    i.compress_type in (0, 8) and i.file_size <= 512 * 1024**2)
            extras(i.extra)
            name = i.orig_filename
            require(name == i.filename and len(name) <= 1024 and not any(ord(c) < 32 or c in '\\:\x7f' for c in name))
            directory = name.endswith('/')
            parts = (name[:-1] if directory else name).split('/')
            require(parts[0] in ('Script Kit.app', '__MACOSX') and all(p not in ('', '.', '..') for p in parts))
            if parts[0] == '__MACOSX':
                require((len(parts) == 1 and directory) or parts[1] in ('Script Kit.app', '._Script Kit.app'))
                if not directory:
                    require(parts[-1].startswith('._') and parts[-1][2:] not in ('', '.', '..'))
                    apple_double.append('/'.join(parts[1:-1] + [parts[-1][2:]]))
            require(stat.S_IFMT(i.external_attr >> 16) in (0, stat.S_IFDIR if directory else stat.S_IFREG))
            require((not directory or i.file_size == 0) and not ((i.external_attr >> 16) & 0o7000))
            for j in range(1, len(parts) + 1):
                prefix = '/'.join(parts[:j]); key = unicodedata.normalize('NFD', prefix).casefold()
                node = (prefix, directory or j < len(parts))
                require(components.setdefault(key, node) == node)
            require(key not in seen); seen.add(key)
            fp = archive.fp; fp.seek(pos); local = fp.read(30)
            require(len(local) == 30 and local[:4] == b'PK\x03\x04')
            flags, method = struct.unpack_from('<HH', local, 6)
            n, x = struct.unpack_from('<HH', local, 26)
            require(flags == i.flag_bits and method == i.compress_type and fp.read(n) == name.encode('utf8' if flags & 2048 else 'ascii'))
            extras(fp.read(x)); pos = fp.tell() + i.compress_size
            if flags & 8:
                fp.seek(pos); desc = fp.read(4)
                if desc == b'PK\x07\x08':
                    desc = fp.read(4)
                require(desc + fp.read(8) == struct.pack('<III', i.CRC, i.compress_size, i.file_size)); pos = fp.tell()
            else:
                require(struct.unpack_from('<III', local, 14) == (i.CRC, i.compress_size, i.file_size))
        require(pos == start == archive.start_dir and 'script kit.app/contents/macos/script-kit-gpui' in seen)
        for target in apple_double:
            require(components.get(unicodedata.normalize('NFD', target).casefold(), (None,))[0] == target)
        require(archive.testzip() is None)


if __name__ == '__main__':
    inspect(sys.argv[1])
