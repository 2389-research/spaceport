"""Signature replacement for the pinned, already-signed Windows companion."""
import struct
import unittest
from test_windows_signing import (
    OPTIONAL, SECURITY, SIGNING, executable, sign,
)


class ResigningTests(unittest.TestCase):
    def setUp(self):
        self.original = sign(executable(), certificate=b"upstream certificate")
        self.offset = SIGNING.certificate_table(self.original)[2]
        self.resigned = sign(self.original[:self.offset], certificate=b"new certificate")

    def test_replaces_a_trailing_certificate_with_a_different_size(self):
        SIGNING.without_signature(self.resigned, self.original)
        larger = sign(self.original[:self.offset], certificate=b"new certificate" * 10)
        SIGNING.without_signature(larger, self.original)

    def test_rejects_changes_outside_the_certificate(self):
        for position in (0x150, OPTIONAL + 68, self.offset - 1):
            with self.subTest(position=position):
                changed = bytearray(self.resigned)
                changed[position] ^= 1
                with self.assertRaises(ValueError):
                    SIGNING.without_signature(changed, self.original)

    def test_rejects_missing_moved_or_non_trailing_new_table(self):
        for changed in (
            executable(), self.resigned + b"extra",
            sign(self.original[:self.offset - 8]),
            sign(self.original[:self.offset] + bytes(8)),
            sign(self.original),
        ):
            with self.subTest(length=len(changed)), self.assertRaises(ValueError):
                SIGNING.without_signature(changed, self.original)

    def test_rejects_malformed_original_certificate_ranges(self):
        for offset, size in (
            (0, 8), (self.offset, 0), (self.offset + 1, 19),
            (self.offset, 19), (self.offset, 21),
            (SECURITY, len(self.original) - SECURITY),
            (0xfffffff8, 8),
        ):
            with self.subTest(offset=offset, size=size):
                original = bytearray(self.original)
                struct.pack_into("<II", original, SECURITY, offset, size)
                with self.assertRaises(ValueError):
                    SIGNING.without_signature(self.resigned, original)


if __name__ == "__main__":
    unittest.main()
