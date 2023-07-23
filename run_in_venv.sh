#!/usr/bin/env sh

dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

. $dir/.env/bin/activate

python $dir/src/main.py $@
