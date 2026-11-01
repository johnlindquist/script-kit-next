"""Tiny non-executable Mach-O headers inspected by the real macOS otool."""
from pathlib import Path
import plistlib
import struct
import tempfile
import unittest

from verify_macos_deployment import verify


SCK = b"/System/Library/Frameworks/ScreenCaptureKit.framework/Versions/A/ScreenCaptureKit\0"


def image(minimum=0x0b0000, link=None, platform=1):
    commands = [struct.pack("<6I", 0x32, 24, platform, minimum, 0x0f0000, 0)]
    if link:
        # Legacy required / weak dylib commands and the modern weak-option encoding.
        offset = 28 if link == "weak-option" else 24
        size = (offset + len(SCK) + 7) & ~7
        command = 0x80000018 if link == "weak" else 0xc
        marker = 0x1a741800 if offset == 28 else 0
        header = struct.pack("<6I", command, size, offset, marker, 0, 0)
        if offset == 28:
            header += struct.pack("<I", 1)
        commands.append((header + SCK).ljust(size, b"\0"))
    body = b"".join(commands)
    return struct.pack("<8I", 0xfeedfacf, 0x0100000c, 0, 2, len(commands), len(body), 0, 0) + body


class Deployment(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix="kit-deployment-test-")
        self.addCleanup(directory.cleanup)
        self.app = Path(directory.name) / "Script Kit.app"
        self.contents = self.app / "Contents"
        self.bin = self.contents / "MacOS"
        self.bin.mkdir(parents=True)
        for name in ("script-kit-gpui", "goose"):
            (self.bin / name).write_bytes(image())
        self.declare("11.0")

    def declare(self, minimum, **more):
        (self.contents / "Info.plist").write_bytes(plistlib.dumps({"LSMinimumSystemVersion": minimum, **more}))

    def test_detects_the_observed_0_1_21_load_command_mismatch(self):
        (self.bin / "script-kit-gpui").write_bytes(image(link="required"))
        with self.assertRaisesRegex(ValueError, r"script-kit-gpui: ScreenCaptureKit requires macOS 12\.3"):
            verify(self.app)
        self.declare("12.3")
        self.assertTrue(verify(self.app)["images"]["script-kit-gpui"]["requiresScreenCaptureKitAtLaunch"])

    def test_an_optional_framework_does_not_raise_the_startup_floor(self):
        for link in (None, "weak", "weak-option"):
            with self.subTest(link=link):
                (self.bin / "script-kit-gpui").write_bytes(image(link=link))
                self.assertFalse(verify(self.app)["images"]["script-kit-gpui"]["requiresScreenCaptureKitAtLaunch"])

    def test_checks_the_sidecars_deployment_floor_numerically(self):
        (self.bin / "goose").write_bytes(image(minimum=0x0c0a00))
        self.declare("12.3")
        with self.assertRaisesRegex(ValueError, r"goose: Mach-O requires 12\.10"):
            verify(self.app)
        self.declare("12.3", LSMinimumSystemVersionByArchitecture={"arm64": "12.10.0"})
        self.assertEqual(verify(self.app)["status"], "pass")

    def test_refuses_missing_or_wrong_platform_evidence(self):
        self.declare("unknown")
        with self.assertRaisesRegex(ValueError, "invalid macOS minimum"):
            verify(self.app)
        self.declare("11.0")
        (self.bin / "goose").write_bytes(image(platform=2))
        with self.assertRaisesRegex(ValueError, "not a macOS image"):
            verify(self.app)
        # otool can return zero and print no commands for a thin x86_64 image.
        wrong_arch = bytearray(image())
        struct.pack_into("<II", wrong_arch, 4, 0x01000007, 3)
        (self.bin / "goose").write_bytes(wrong_arch)
        with self.assertRaisesRegex(ValueError, "missing or ambiguous arm64"):
            verify(self.app)


if __name__ == "__main__":
    unittest.main()
