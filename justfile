set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# Format, clippy here and for Windows, the whole suite: what CI runs on Linux.
check: lint test

# Refresh or check the help reference and command examples (no core is started).
[unix]
docs:
    sh scripts/docs.sh --write

[windows]
docs:
    $env:LBC_UPDATE_DOCS='1'; cargo test --locked -p lumbridge-core --test cli_docs

[unix]
docs-check:
    sh scripts/docs.sh --check

[windows]
docs-check:
    Remove-Item Env:LBC_UPDATE_DOCS -ErrorAction SilentlyContinue; cargo test --locked -p lumbridge-core --test cli_docs

[unix]
test:
    cargo test --locked

# Windows' own ConPTY swallows answers to programs: lbc's tests need the one it ships.
[windows]
test:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/fetch-conpty.ps1 -Dest target/debug
    cargo test --locked

# Format and clippy only (seconds once built). Windows is type-checked with zig stubbed.
[unix]
lint:
    cargo fmt --all -- --check
    cargo deny check
    cargo clippy --locked --all-targets -- -D warnings
    ZIG="$PWD/scripts/zig-stub.sh" CARGO_TARGET_DIR=target/lint-windows cargo clippy --locked --target x86_64-pc-windows-msvc --all-targets -- -D warnings

[windows]
lint:
    cargo fmt --all -- --check
    cargo deny check
    cargo clippy --locked --all-targets -- -D warnings

fmt:
    cargo fmt --all

# After a dependency change: the licenses lbc carries (`lbc --licenses`).
[unix]
notices:
    scripts/notices

# Build and install this checkout; running terminals are kept.
[unix]
install:
    sh scripts/install-local.sh
