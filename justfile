set quiet

default: clippy

fmt:
    rustfmt +nightly **/*.rs --edition 2024

clippy: fmt
    cargo clippy

clean:
    cargo clean

build: clean
    cargo build

bump-deps: clean
    #!/usr/bin/env bash
    set -euxo pipefail
    cargo +nightly -Zunstable-options update --breaking
    cargo update

bump-version: clean
    #!/usr/bin/env bash
    set -euxo pipefail
    VERSION="v$(date +%y.%-m.%-d)"
    CV="${VERSION#v}"
    sed -i 's/^version = ".*"/version = "'"$CV"'"/' Cargo.toml
    cargo check
    git add Cargo.toml Cargo.lock
    git commit -m "chore: bump version"
    git tag "$VERSION"
    git push
    git push origin "$VERSION"
