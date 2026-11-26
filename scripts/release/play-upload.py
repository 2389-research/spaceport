#!/usr/bin/env python3
# ABOUTME: Uploads a signed Android App Bundle to a Google Play testing track, or lists what each track holds.
# ABOUTME: Signs in as the Play service account with only the standard library and the openssl CLI.
"""Use from CI or by hand, with the service account JSON in SPACEPORT_PLAY_SERVICE_ACCOUNT_JSON or --credentials.

  play-upload.py tracks
  play-upload.py upload BUNDLE --name 20261126.2 [--track internal] [--status completed]

Production is deliberately absent: promoting a build to everyone stays a person's decision in the Play Console.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

PACKAGE = 'ai.research2389.spaceport.mobile'
SCOPE = 'https://www.googleapis.com/auth/androidpublisher'
API = f'https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{PACKAGE}'
UPLOAD = f'https://androidpublisher.googleapis.com/upload/androidpublisher/v3/applications/{PACKAGE}'
TRACKS = ('internal', 'alpha', 'beta')
# 'draft' is the only status Play accepts until the app's first release has been rolled out.
STATUSES = ('completed', 'draft')


class PlayError(RuntimeError):
    """Google answered with an error; the message carries its explanation."""


def encode(data):
    return base64.urlsafe_b64encode(data).rstrip(b'=').decode()


def check_account(account):
    """Refuse anything but a service account key, such as a user's OAuth client file."""
    if account.get('type') != 'service_account' or not account.get('private_key') or not account.get('client_email'):
        raise ValueError('Credentials must be a Google service account key with client_email and private_key')
    return account


def assertion(account, now):
    """An RS256-signed JWT that trades for an Android Publisher access token."""
    header = encode(json.dumps(dict(alg='RS256', typ='JWT'), separators=(',', ':')).encode())
    claims = encode(json.dumps(dict(iss=account['client_email'], scope=SCOPE, aud=account['token_uri'],
                                    iat=now, exp=now + 3600), separators=(',', ':')).encode())
    signed = f'{header}.{claims}'
    # openssl reads the key from a private temporary file that is removed before this returns.
    with tempfile.TemporaryDirectory(prefix='spaceport-play-key-') as directory:
        key = Path(directory) / 'key.pem'
        key.touch(mode=0o600)
        key.write_text(account['private_key'])
        signature = subprocess.run(['openssl', 'dgst', '-sha256', '-sign', str(key)], input=signed.encode(),
                                   capture_output=True, check=True, timeout=60).stdout
    return f'{signed}.{encode(signature)}'


def track_body(track, version_code, name, status):
    if track not in TRACKS:
        raise ValueError(f'Only testing tracks are allowed: {", ".join(TRACKS)}')
    if status not in STATUSES:
        raise ValueError(f'Release status must be one of: {", ".join(STATUSES)}')
    return dict(track=track, releases=[dict(name=name, versionCodes=[str(version_code)], status=status)])


class Play:
    def __init__(self, account):
        body = urllib.parse.urlencode(dict(grant_type='urn:ietf:params:oauth:grant-type:jwt-bearer',
                                           assertion=assertion(account, int(time.time())))).encode()
        self.token = self.call('POST', account['token_uri'], body, 'application/x-www-form-urlencoded',
                               auth=False)['access_token']

    def call(self, method, url, body=None, content_type='application/json', auth=True):
        request = urllib.request.Request(url, data=body, method=method)
        if body is not None:
            request.add_header('Content-Type', content_type)
        if auth:
            request.add_header('Authorization', f'Bearer {self.token}')
        try:
            with urllib.request.urlopen(request, timeout=600) as response:
                text = response.read()
        except urllib.error.HTTPError as error:
            detail = error.read().decode(errors='replace')
            try:
                detail = json.loads(detail)['error']['message']
            except (ValueError, KeyError, TypeError):
                pass
            raise PlayError(f'{method} {url.split("?")[0]} answered {error.code}: {detail}') from None
        return json.loads(text) if text else {}

    def json(self, method, url, value=None):
        return self.call(method, url, None if value is None else json.dumps(value).encode())

    def edit(self, work):
        """Run work(edit_id) inside a Play edit; commit=True from work commits it, anything else discards it."""
        edit = self.json('POST', f'{API}/edits', {})['id']
        try:
            result, commit = work(edit)
            if commit:
                self.json('POST', f'{API}/edits/{edit}:commit')
                edit = None
            return result
        finally:
            if edit is not None:
                self.call('DELETE', f'{API}/edits/{edit}')


def tracks(play):
    def read(edit):
        return play.json('GET', f'{API}/edits/{edit}/tracks').get('tracks', []), False
    return play.edit(read)


def upload(play, bundle, track, name, status):
    def send(edit):
        uploaded = play.call('POST', f'{UPLOAD}/edits/{edit}/bundles?uploadType=media', Path(bundle).read_bytes(),
                             'application/octet-stream')
        code = uploaded['versionCode']
        play.json('PUT', f'{API}/edits/{edit}/tracks/{track}', track_body(track, code, name, status))
        return code, True
    return play.edit(send)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--credentials', type=Path, help='service account JSON file (default: $SPACEPORT_PLAY_SERVICE_ACCOUNT_JSON)')
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('tracks', help='list each track and its releases')
    send = commands.add_parser('upload', help='upload a bundle to a testing track')
    send.add_argument('bundle', type=Path)
    send.add_argument('--name', required=True, help='release name shown in the Play Console, such as the tag version')
    send.add_argument('--track', default='internal', choices=TRACKS)
    send.add_argument('--status', default='completed', choices=STATUSES)
    args = parser.parse_args()
    try:
        text = args.credentials.read_text() if args.credentials else os.environ.get('SPACEPORT_PLAY_SERVICE_ACCOUNT_JSON', '')
        if not text:
            raise ValueError('Pass --credentials or set SPACEPORT_PLAY_SERVICE_ACCOUNT_JSON')
        play = Play(check_account(json.loads(text)))
        if args.command == 'tracks':
            for entry in tracks(play):
                for release in entry.get('releases', []) or [{}]:
                    print(f"{entry['track']:<10} {release.get('status', '-'):<10} {release.get('name', '-'):<14} "
                          f"{','.join(release.get('versionCodes', [])) or '-'}")
        else:
            code = upload(play, args.bundle, args.track, args.name, args.status)
            print(f'Uploaded version code {code} to the {args.track} track as {args.status}')
    except (ValueError, OSError, PlayError, subprocess.SubprocessError, KeyError) as error:
        sys.exit(f'Play upload refused: {error}')


if __name__ == '__main__':
    main()
