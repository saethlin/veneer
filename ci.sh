#!/bin/bash

set -e
exec 2>&1
export TERM=xterm-256color

function group {
    echo "::group::$@"
    $@
    echo "::endgroup"
}

if [[ "$1" == "style" ]]
then
    group cargo fmt --check
    group cargo clippy -- -Dclippy::all
else
    group cargo test
    group cargo test --features=rt
fi
