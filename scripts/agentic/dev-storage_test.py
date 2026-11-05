#!/usr/bin/env python3
"""Exercise real storage policy with fake mount observations, never host disks."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shlex
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("dev_storage", Path(__file__).with_name("dev-storage.py"))
storage = importlib.util.module_from_spec(spec)
spec.loader.exec_module(storage)


class StorageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name) / "repo"
        self.volume = Path(self.temp.name) / "SSD with ' quotes"
        self.repo.mkdir()
        self.volume.mkdir()
        self.env = {"SCRIPT_KIT_DEV_VOLUME": str(self.volume)}
        self.mount = {"path": str(self.volume), "uuid": "volume-a", "filesystem": "APFS",
                      "external": True, "writable": True, "available_bytes": 30 * 1024**3}
        self.observer = patch.object(storage, "observe_volume", return_value=self.mount)
        self.mock = self.observer.start()
        self.addCleanup(self.observer.stop)
        self.root = storage.resolve_layout(str(self.repo), self.env)[2]

    def invoke(self, command="inspect", shell=False):
        out, err = io.StringIO(), io.StringIO()
        with patch.dict(os.environ, self.env, clear=True), contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = storage.main(["--repo", str(self.repo), command] + (["--shell"] if shell else []))
        return code, out.getvalue(), err.getvalue()

    def refused(self, kind, command="prepare"):
        code, out, err = self.invoke(command, shell=True)
        self.assertEqual(code, 78)
        self.assertEqual(out, "")
        self.assertEqual(json.loads(err)["error"]["type"], kind)

    def test_without_a_mounted_default_ssd_builds_inside_the_checkout(self):
        self.env = {}
        with patch.object(storage.os.path, "ismount", return_value=False):
            code, out, err = self.invoke("prepare", shell=True)
            self.assertEqual(code, 0, err)
            exports = dict(line.removeprefix("export ").split("=", 1) for line in out.splitlines())
            local = self.repo.resolve() / "target" / "dev"
            self.assertEqual(shlex.split(exports["CARGO_TARGET_DIR"])[0], str(local / "target"))
            self.assertEqual(shlex.split(exports["SCRIPT_KIT_DEV_VOLUME"])[0], "local")
            self.assertTrue((local / "target").is_dir())
            self.assertFalse((local / storage.MARKER).exists())
            # dev-cycle.sh re-runs prepare with the exported variables.
            self.env = {k: shlex.split(v)[0] for k, v in exports.items()}
            code, _, err = self.invoke("prepare", shell=True)
            self.assertEqual(code, 0, err)
        self.mock.assert_not_called()

    def test_explicit_local_ignores_a_mounted_default_ssd(self):
        self.env = {"SCRIPT_KIT_DEV_VOLUME": "local"}
        with patch.object(storage.os.path, "ismount", return_value=True):
            code, out, err = self.invoke("inspect")
        self.assertEqual(code, 0, err)
        self.assertEqual(json.loads(out)["volume"], "local")
        self.mock.assert_not_called()

    def test_inspect_is_read_only_including_shell_under_noninteractive(self):
        self.env["SCRIPT_KIT_NONINTERACTIVE"] = "1"
        code, out, err = self.invoke()
        self.assertEqual((code, err), (0, ""))
        self.assertFalse(json.loads(out)["prepared"])
        self.assertEqual(self.invoke(shell=True)[0], 0)
        self.assertFalse(self.root.parent.exists())
        self.refused("noninteractive")
        self.assertFalse(self.root.parent.exists())

    def test_missing_mount_never_creates_fallback_directories(self):
        self.volume.rmdir()
        self.mock.side_effect = storage.Refused("missing_mount", "not mounted")
        self.refused("missing_mount")
        self.assertFalse(self.volume.exists())

    def test_idempotent_preparation_and_shell_quoting(self):
        code, out, err = self.invoke("prepare", shell=True)
        self.assertEqual((code, err), (0, ""))
        exports = dict(shlex.split(line)[1].split("=", 1) for line in out.splitlines())
        marker = self.root / storage.MARKER
        before = marker.stat().st_mtime_ns
        self.env.update(exports)
        self.assertEqual(self.invoke("prepare", shell=True), (0, out, ""))
        self.assertEqual(marker.stat().st_mtime_ns, before)
        for name in ("CARGO_TARGET_DIR", "GOOSE_SIDECAR_TARGET_DIR", "CLANG_MODULE_CACHE_PATH", "SCCACHE_DIR", "TMPDIR"):
            self.assertTrue(Path(exports[name]).is_dir())
            self.assertTrue(Path(exports[name]).is_relative_to(self.volume))
        self.assertNotIn("SCRIPT_KIT_GOOSE_BINARY", exports)
        self.assertFalse((self.repo / "target").exists())

    def test_swapped_uuid_and_foreign_marker_are_protected(self):
        self.assertEqual(self.invoke("prepare")[0], 0)
        marker = self.root / storage.MARKER
        original = marker.read_text()
        self.mount["uuid"] = "volume-b"
        self.refused("swapped_mount")
        self.assertEqual(marker.read_text(), original)
        self.mount["uuid"] = "volume-a"
        marker.write_text(json.dumps({"repo": "someone-else", "volume_uuid": "volume-a"}))
        self.refused("foreign_marker")

    def test_inherited_watcher_uuid_detects_replacement_before_claim(self):
        self.env["SCRIPT_KIT_DEV_VOLUME_UUID"] = "other-volume"
        self.refused("swapped_mount")
        self.assertFalse(self.root.exists())

    def test_volume_change_during_prepare_creates_nothing(self):
        self.mock.side_effect = [self.mount, {**self.mount, "uuid": "replacement"}]
        self.refused("swapped_mount")
        self.assertFalse(self.root.parent.exists())

    def test_symlink_target_cannot_redirect_build_to_internal_storage(self):
        self.assertEqual(self.invoke("prepare")[0], 0)
        target = self.root / "target"
        target.rmdir()
        target.symlink_to(self.repo, target_is_directory=True)
        self.refused("symlink_target")
        self.assertEqual(list(self.repo.iterdir()), [])

    def test_symlink_mount_is_rejected_before_observation(self):
        self.volume.rmdir()
        self.volume.symlink_to(self.repo, target_is_directory=True)
        self.refused("symlink_target")
        self.mock.assert_not_called()

    def test_nonempty_unowned_directory_is_never_adopted(self):
        self.root.mkdir(parents=True)
        protected = self.root / "other-work"
        protected.write_text("keep")
        self.refused("unowned_root")
        self.assertEqual(protected.read_text(), "keep")
        self.assertFalse((self.root / storage.MARKER).exists())

    def test_target_cache_and_build_dir_conflicts_precede_mutation(self):
        for key in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "CARGO_BUILD_BUILD_DIR",
                    "CARGO_RESOLVER_LOCKFILE_PATH", "GOOSE_SIDECAR_TARGET_DIR", "SCCACHE_DIR"):
            with self.subTest(key=key):
                self.env[key] = str(self.repo / "wrong")
                self.refused("conflicting_target")
                self.assertFalse(self.root.parent.exists())
                del self.env[key]

    def test_low_disk_refuses_but_exact_floor_passes(self):
        self.mount["available_bytes"] = storage.MIN_FREE_BYTES - 1
        self.refused("insufficient_space")
        self.assertFalse(self.root.parent.exists())
        self.mount["available_bytes"] = storage.MIN_FREE_BYTES
        self.assertEqual(self.invoke("prepare")[0], 0)


if __name__ == "__main__":
    unittest.main()
