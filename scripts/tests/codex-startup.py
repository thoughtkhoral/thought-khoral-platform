#!/usr/bin/env python3
"""Check the host UID guard; real image checks live in codex-package.py."""
import os
import pathlib
import subprocess
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]

class Startup(unittest.TestCase):
    def test_unprepared_host_cannot_assert_packaging_readiness(self):
        result = subprocess.run(['bash', str(ROOT/'scripts/codex-start.sh'), '--check'], capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn(b'packaging verified', result.stdout)

if __name__ == '__main__': unittest.main()
