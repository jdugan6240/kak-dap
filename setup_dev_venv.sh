#!/usr/bin/env sh

dir=~/.local/pyenv/

python -m venv $dir


$dir/bin/pip install -r requirements.txt
$dir/bin/pip install -r requirements_dev.txt

