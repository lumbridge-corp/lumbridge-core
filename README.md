# Lumbridge Core (`lbc`)

Lumbridge Core runs programs and coding agents in terminals that stay alive after you leave.
Agents can start other agents, send them work, and hear back. You can attach to any terminal and take over.

Core draws nothing. Use it from your shell, an agent, or your own app.
It is the runtime of [Lumbridge](https://lumbridgecorp.com) ([source](https://github.com/lumbridge-corp)), the open engine for agentic organizations.

- Guide: <https://lumbridgecorp.com/guide/core>
- About this part: <https://lumbridgecorp.com/core>
- Other parts: [Lumbridge App](https://github.com/lumbridge-corp/lumbridge-app) (the window), [Lumbridge TUI](https://github.com/lumbridge-corp/lumbridge-tui) (the older terminal multiplexer), and the rest at <https://github.com/lumbridge-corp>

## State

- Version 0.1. You build it from source. There is no installer and no download.
- Linux and macOS, on x86_64 and aarch64. The Windows code is kept, but not tested.
- Core never checks for updates and never fetches one.
- `lbc --version` prints 0.1.19, the crate version. Release v0.1.0 is this tree.

## Quick start

You need git, a C toolchain, Rust 1.97.1 or newer, and Zig 0.16.0 (exactly that release).

On Ubuntu 24.04:

```sh
sudo apt-get update
sudo apt-get install -y git curl ca-certificates build-essential xz-utils

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.97.1
. "$HOME/.cargo/env"

# Zig 0.16.0 (on aarch64, use zig-aarch64-linux-0.16.0)
curl -fsSLO https://ziglang.org/download/0.16.0/zig-x86_64-linux-0.16.0.tar.xz
mkdir -p "$HOME/.local/bin"
tar -xf zig-x86_64-linux-0.16.0.tar.xz -C "$HOME/.local"
ln -sf "$HOME/.local/zig-x86_64-linux-0.16.0/zig" "$HOME/.local/bin/zig"
export PATH="$HOME/.local/bin:$PATH"
```

On macOS, install the Xcode Command Line Tools (`xcode-select --install`), Rust from <https://rustup.rs>,
and Zig 0.16.0 from <https://ziglang.org/download/>. Put `zig` and `~/.local/bin` on your PATH.

Get the source and build it. The first build takes a few minutes and downloads the Rust crates and Zig packages it needs:

```sh
git clone --recurse-submodules --shallow-submodules https://github.com/lumbridge-corp/lumbridge-core.git
cd lumbridge-core
sh scripts/install-local.sh
```

That builds a release binary, copies it to `~/.local/bin/lumbridge-core`, links `lbc` to it, and starts the core.
Set `LBC_INSTALL_DIR` to install somewhere else.

Try it:

```sh
lbc --version
lbc server status
lbc run echo hello                          # run a program, stream its output, wait for it
lbc run --bg --name demo sh -c 'echo started; sleep 600'
lbc ls                                      # the live terminals
lbc read demo                               # what demo printed, without attaching
lbc kill demo
```

`lbc server stop` stops the core. Any `lbc` command starts it again when it needs one.

To switch a running core to another build you already have, run
`lbc update --from /path/to/lumbridge-core`. Running terminals keep running.
Updates and `lbc server restart` keep the environment the core started with;
run `lbc server restart --fresh` from a terminal outside Lumbridge to use that shell's
environment without ending programs. Stopping and starting also takes a fresh one.

If saving fails, the restart still goes ahead with the launcher's environment.
Saved settings expire after three minutes and cannot carry over to another boot
on Linux or macOS. On Windows, a restart cut short by a fast reboot can reuse them
once. A plain SIGTERM keeps terminals and screens, but does not save settings.
The new launcher still chooses the session's location and, on macOS, whether it
starts from SSH or launchd.
On Windows, templates and quoted shell commands need `sh` on PATH, such as Git for Windows provides.

See [security](docs/security.md) for the trust details.

## Run it and come back

```sh
lbc run cargo test                  # stream output and wait for it
lbc run --bg --name web bun dev     # start it and return
lbc ls                             # see live terminals
lbc attach web                     # take over; Ctrl+\ detaches
lbc read web --lines 100            # read without attaching
```

Closing your viewer does not end the program.
A waiting run checks in after 180 seconds by default, leaving the program running.
It prints the terminal's address so you can read it or wait again.

For a terminal you want to keep even if its program is lost, use `lbc run --keep`.
Recovery starts a fresh shell, never reruns the task.
A normal exit or an explicit kill still ends it.

## Default shell

With no program, `lbc run --bg` starts the machine's shell: `$SHELL` (or
`/bin/sh`) on Unix, PowerShell 7 on Windows if available, else Windows PowerShell.
Override it in `~/.config/lumbridge-core/config.toml` (Windows:
`%APPDATA%\lumbridge-core\config.toml`, or `$XDG_CONFIG_HOME/lumbridge-core/config.toml`
when set):

```toml
shell = ["nu"]
# Windows Git Bash (literal strings keep backslashes):
# shell = ['C:\Program Files\Git\bin\bash.exe', '--login', '-i']
```

The list is an executable followed by separate arguments, without extra path
quoting. It must contain a nonempty executable. Changes apply to the next new
shell or `keep` recovery, including recovery after a restart; no restart is
needed. Running terminals, explicit programs, and quoted command lines keep
their behavior. A bad setting reports a config error; a shell that cannot start
reports `spawn_failed`. Recovery stays pending and retries, with errors in
`core.log` (config) or `holder.log` (spawn); it never silently falls back to another shell.

## Draft wait

Sends and parent notes wait for you to send or clear an unfinished draft,
up to 60 seconds after your last key. In the same config.toml:

```toml
notes_quiet = 3 # seconds of quiet before a parent note
draft_wait = 60 # seconds a draft holds sends and notes; 0 keeps only the quiet pause
```

Both settings apply to the next delivery without restarting the core. Values must
be finite, non-negative seconds that fit in a duration. If the config is invalid,
delivery uses defaults and logs the config error once in `core.log`.
Reverse sends allow at least the default 60-second draft hold.
A parent configured above 60 seconds can still outwait a child configured below
that parent's limit and cause its reverse send to time out.

## Agents run agents

Give an agent `lbc --skill`. It prints the agent docs in one read, including how to delegate, wait, report back, and clean up.

[Templates](docs/templates.md) give an agent a command and a role.
After saving the reviewer example from that page:

```sh
lbc templates
lbc run --bg --name review reviewer "Review src/x.rs. Read-only."
lbc send review "Focus on the error path."
lbc read review --lines 100
```

Agents started inside an lbc terminal become its children.
They can report back with `lbc send` and wait for their children with `lbc wait --children all`.
The parent gets notes when a child needs attention or ends.
Children end with their parent unless started with `--outlive-parent`.
On Unix, ending a terminal hangs up its program and foreground job, rather than
hunting down detached programs. Programs deliberately detached with `nohup` or
`setsid` can keep running, including background servers (their ports stay busy).
An unexpected holder loss still uses inherited cleanup tokens to end abandoned
programs, including detached ones. Windows keeps its job-object cleanup.
Terminals kept alive through an update retain their old holder's behavior until
recreated.

## Every machine

```sh
lbc machine add my-ssh-host --name box
lbc run --on box --cwd /srv/app cargo test
lbc attach box/t3
```

Use the address returned by your run, not the example's `box/t3`.
Programs and files stay on that machine. Your local core brings its terminals into the same list.
[Machines](docs/machines.md) covers SSH, custom connections, remote files, and reconnecting.

## Status comes from the program

An agent can say it is working, done, blocked, or in error through OSC 7501, the Program Status Protocol.
Core stores what it reports. It does not guess from screen text or recognize agent brands.
A program that does not report status has none.

Use `lbc get review` for a snapshot or `lbc events` to follow changes.

## Build your own client

`lbc schema` prints the request/response API as OpenRPC with JSON Schema.
[The API docs](schema/README.md) cover connecting, terminal streams, events, and status.
Use `lbc connect` to carry the protocol over stdin and stdout, including through SSH.

## Docs

- [Commands](docs/commands.md): generated from the binary's help.
- [Machines](docs/machines.md): work across machines.
- [Templates](docs/templates.md): reusable agent roles.
- [Security](docs/security.md): access, state, and updates.
- [API](schema/README.md): build a client.

For a quick lookup, run `lbc --help` or a command's help, such as `lbc run --help`.
For agents, use `lbc --skill`.

## Work on the code

The checks are [scripts/ci.sh](scripts/ci.sh): format, clippy and the whole test suite. CI runs the same script.
[just](https://just.systems/) has shortcuts for them.

```sh
git submodule update --init --depth 1   # if you cloned without submodules
cargo build --locked -p lumbridge-core
scripts/ci.sh
```

`just install` installs this checkout and restarts your core. Running terminals keep running.
`just docs` refreshes the command reference. `just docs-check` checks it and the examples.
`scripts/scrub-check` looks for private addresses, home folders and tokens before anything is published.

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup and checks, and [SECURITY.md](SECURITY.md) to report a vulnerability.

## License

Lumbridge Core started as a fork of [Herdr](https://github.com/herdrdev/herdr-core) (Apache-2.0). Thanks to its authors.

[Apache-2.0](LICENSE). Copyright 2026 Lumbridge Corp for its own work.
Code from other projects keeps its authors' copyright and notices.
`lbc --licenses` prints the included third-party licenses. After a dependency change, regenerate them with `just notices`.
