# ABOUTME: Check the Google Play upload helper's service-account token request and track release body.
# ABOUTME: Signs with a throwaway RSA key and verifies the signature with openssl, so no network is used.
import base64
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('play_upload', Path(__file__).resolve().parents[1] / 'play-upload.py')
play = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(play)


def unpad(part):
    return base64.urlsafe_b64decode(part + '=' * (-len(part) % 4))


class PlayUploadTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-play-')
        self.addCleanup(self.temp.cleanup)
        self.dir = Path(self.temp.name)
        key = self.dir / 'key.pem'
        subprocess.run(['openssl', 'genpkey', '-algorithm', 'RSA', '-pkeyopt', 'rsa_keygen_bits:2048', '-out', str(key)],
                       check=True, capture_output=True)
        subprocess.run(['openssl', 'pkey', '-in', str(key), '-pubout', '-out', str(self.dir / 'pub.pem')],
                       check=True, capture_output=True)
        self.account = dict(type='service_account', client_email='upload@example.iam.gserviceaccount.com',
                            private_key=key.read_text(), token_uri='https://oauth2.googleapis.com/token')

    def test_assertion_is_an_rs256_jwt_signed_by_the_account_key(self):
        header, claims, signature = play.assertion(self.account, now=1_800_000_000).split('.')
        self.assertEqual(json.loads(unpad(header)), dict(alg='RS256', typ='JWT'))
        self.assertEqual(json.loads(unpad(claims)), dict(
            iss='upload@example.iam.gserviceaccount.com', scope='https://www.googleapis.com/auth/androidpublisher',
            aud='https://oauth2.googleapis.com/token', iat=1_800_000_000, exp=1_800_003_600))
        (self.dir / 'signed').write_bytes(f'{header}.{claims}'.encode())
        (self.dir / 'sig').write_bytes(unpad(signature))
        check = subprocess.run(['openssl', 'dgst', '-sha256', '-verify', str(self.dir / 'pub.pem'),
                                '-signature', str(self.dir / 'sig'), str(self.dir / 'signed')], capture_output=True, text=True)
        self.assertEqual(check.returncode, 0, check.stdout + check.stderr)

    def test_signing_leaves_no_key_file_behind(self):
        before = set(Path(tempfile.gettempdir()).glob('spaceport-play-key-*'))
        play.assertion(self.account, now=1_800_000_000)
        self.assertEqual(set(Path(tempfile.gettempdir()).glob('spaceport-play-key-*')), before)

    def test_account_must_be_a_service_account_with_a_key(self):
        for broken in (dict(self.account, type='authorized_user'), {k: v for k, v in self.account.items() if k != 'private_key'}):
            with self.subTest(broken=sorted(broken)), self.assertRaisesRegex(ValueError, 'service account'):
                play.check_account(broken)

    def test_track_release_names_the_build_and_rolls_it_out(self):
        self.assertEqual(play.track_body('internal', 2026112602, '20261126.2', 'completed'), dict(
            track='internal', releases=[dict(name='20261126.2', versionCodes=['2026112602'], status='completed')]))
        with self.assertRaisesRegex(ValueError, 'track'):
            play.track_body('production', 1, '1', 'completed')
        with self.assertRaisesRegex(ValueError, 'status'):
            play.track_body('internal', 1, '1', 'halted')


if __name__ == '__main__':
    unittest.main()
