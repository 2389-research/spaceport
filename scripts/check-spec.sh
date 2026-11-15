#!/usr/bin/env bash
# Run the Spaceport specification gate: the tool tests and `spec.py check
# --skip-schema`. `just check-spec` and CI both call this script.
#
# Three tool tests import jsonschema. They run in a private virtual
# environment holding exactly the pinned docs/spaceport/tools/requirements.txt,
# so the result never depends on what the system Python happens to have.
# SPEC_VENV overrides where that environment lives (default target/spec-venv).
# The pinned jsonschema needs `python3` 3.10 or newer; macOS's /usr/bin/python3
# is 3.9, so pip finds no matching version there.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

requirements=docs/spaceport/tools/requirements.txt
venv=${SPEC_VENV:-target/spec-venv}
python=$venv/bin/python
stamp=$venv/requirements.txt

# Rebuild the environment when it is missing or its pins changed.
if [ ! -x "$python" ] || ! cmp -s "$requirements" "$stamp"; then
    rm -rf "$venv"
    python3 -m venv "$venv"
    "$python" -m pip install --quiet --disable-pip-version-check --requirement "$requirements"
    cp "$requirements" "$stamp"
fi

export PYTHONDONTWRITEBYTECODE=1
"$python" -m unittest discover -s docs/spaceport/tools -p 'test_*.py'
"$python" docs/spaceport/tools/spec.py check --skip-schema
