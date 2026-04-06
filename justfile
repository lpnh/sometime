set quiet

default: clippy

fmt:
    rustfmt +nightly src/**/*.rs --edition 2024

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
    jj describe -m "chore: bump version"
    jj tag set "$VERSION"
    git push origin "$VERSION"
