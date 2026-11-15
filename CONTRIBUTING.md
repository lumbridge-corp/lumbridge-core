# Contributing

Report bugs and request features as issues in this repository. Discuss bigger changes there before writing a PR. Report security bugs by email instead: see [SECURITY.md](SECURITY.md). Keep PRs small and add tests for behavior changes.

Core is an agent-neutral terminal runtime, not a multiplexer. We don't add per-agent integrations or hooks. Agent status comes only from OSC 7501; see [core's status docs](schema/README.md#status). Agent-specific status support belongs in the program or its integration, not core.

## Setup

Install Rust (1.97.1 is used), Zig 0.16.0, [just](https://just.systems/), Python 3 (one test uses it), and a C linker/toolchain for your OS. The README's quick start has the commands for Ubuntu.

```sh
rustup component add rustfmt clippy
cargo install cargo-deny --locked
git submodule update --init --depth 1
```

On Linux and macOS, also install the target used to type-check Windows code:

```sh
rustup target add x86_64-pc-windows-msvc
```

## Checks

```sh
cargo build --locked -p lumbridge-core
just check
just docs-check
```

`just check` runs formatting, cargo-deny, clippy, and the full test suite. `cargo test --locked` runs just the tests. On Windows, use `just test` to fetch the pinned ConPTY before testing. Tests use isolated sessions; don't point them at a running core.

Keep OS-specific code behind `platform/` and its shared interface, without `cfg` branches in shared code. See [AGENTS.md](AGENTS.md) for the project rules.

After changing dependencies, run `just notices` (Unix; needs `jq`) and review the generated licenses. Keep the [non-Cargo notices](distribution/notices/README.md) current too. `lbc --licenses` prints these texts.

Before a PR, run `scripts/scrub-check`. It fails on private network addresses, home folders, email addresses and token formats that are not listed in `.scrub-allow`. Use made-up examples: user `sam`, host `desk`, `example.com`, `192.0.2.x`.

PRs target `main`. Write what users will notice in CHANGELOG.md. By sending a PR you agree that your change is licensed under Apache-2.0, like the rest of the project.
