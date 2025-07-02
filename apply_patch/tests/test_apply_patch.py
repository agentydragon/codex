import sys
import io
import os
import tempfile
import unittest

import apply_patch


class ApplyPatchTests(unittest.TestCase):
    def setUp(self):
        self.tempdir = tempfile.TemporaryDirectory()
        self.dir = self.tempdir.name

    def tearDown(self):
        self.tempdir.cleanup()

    def run_patch(self, patch_text):
        old_stdin = sys.stdin
        try:
            sys.stdin = io.StringIO(patch_text)
            apply_patch.main()
        finally:
            sys.stdin = old_stdin

    def test_add_file(self):
        target = os.path.join(self.dir, 'foo.txt')
        patch = f"""*** Begin Patch
*** Add File: {target}
+hello
+world
*** End Patch
"""
        self.run_patch(patch)
        with open(target) as f:
            self.assertEqual(f.read(), 'hello\nworld\n')

    def test_delete_file(self):
        target = os.path.join(self.dir, 'foo.txt')
        with open(target, 'w') as f:
            f.write('data')
        patch = f"""*** Begin Patch
*** Delete File: {target}
*** End Patch
"""
        self.run_patch(patch)
        self.assertFalse(os.path.exists(target))

    def test_update_file(self):
        target = os.path.join(self.dir, 'file.txt')
        with open(target, 'w') as f:
            f.write('line1\nline2\nline3\n')
        patch = f"""*** Begin Patch
*** Update File: {target}
@@
 line1
-line2
+new2
 line3
*** End Patch
"""
        self.run_patch(patch)
        with open(target) as f:
            self.assertEqual(f.read(), 'line1\nnew2\nline3\n')

    def test_update_and_move_file(self):
        src = os.path.join(self.dir, 'a.txt')
        with open(src, 'w') as f:
            f.write('x\n')
        dst = os.path.join(self.dir, 'b.txt')
        patch = f"""*** Begin Patch
*** Update File: {src}
*** Move to: {dst}
@@
 x
*** End Patch
"""
        self.run_patch(patch)
        self.assertFalse(os.path.exists(src))
        with open(dst) as f:
            self.assertEqual(f.read(), 'x\n')

if __name__ == '__main__':
    unittest.main()
