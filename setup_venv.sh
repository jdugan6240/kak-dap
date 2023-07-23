#!/usr/bin/env sh

dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

python -m venv $dir/.env

$dir/.env/bin/pip install -r requirements.txt
