# macOS bundle and signing

There is no release pipeline: Lumbridge Core is built from source, locally
(`just install`). No Apple credentials or notarization are involved, and builds
are signed ad hoc.

`scripts/mac-bundle` assembles **Lumbridge Core.app** with the Info.plist in
`distribution/macos-Info.plist` (the same metadata is embedded in the
executable; it names **Lumbridge Core** and explains the privacy requests made
by terminal programs). `scripts/mac-sign` signs the executable and then the
bundle with the hardened runtime, the entitlements in
`distribution/macos-entitlements.plist` and the stable identifier
`com.lumbridgecorp.core`. Keep that identifier stable so privacy grants stick;
grants do not carry over to a build with another identifier, and ad-hoc builds may
need their grants again after a rebuild.

The hardened-runtime entitlement
`com.apple.security.cs.allow-dyld-environment-variables` preserves `DYLD_*`
for terminal programs. It does not disable library validation for core itself,
nor override restrictions in Apple-protected child programs. Core needs no JIT,
debugging or unsigned-memory exceptions. Camera, audio-input and Apple Events
entitlements permit user-approved terminal workloads; they do not grant access
without consent. There is no App Sandbox.

`lbc update --from <bundle or executable>` stages and verifies the complete
bundle, checks that it runs, then atomically exchanges it with the installed
bundle. Entry points in `LBC_INSTALL_DIR` (default `~/.local/bin`) resolve to the
newest bundle at the fixed physical path
`~/.local/share/lumbridge-core/active/Lumbridge Core.app`, via the `current`
symlink. Previous bundles are kept aside while their cores or holders run; later
installs reclaim unused previous bundles.

After launchd detachment the core uses a SETEXEC responsibility disclaimer;
each holder does the same. Terminal programs never disclaim: the live holder
must remain their privacy subject across core restarts. Missing SPI falls back
to inherited responsibility with a diagnostic. Existing pre-upgrade holders
cannot be repaired by restarting the core: users must recreate those terminals
if privacy access is broken. Updates never kill them automatically.

## Verification

`tests/macos.rs` verifies launchd isolation and the embedded identity. Compile
`crates/lumbridge-core/tests/macos-dyld.c` with `cc` for a non-Apple probe: with
no arguments it checks that DYLD_LIBRARY_PATH equals
`/lumbridge-identity-test`; with core PID and caller PID arguments it checks their
coalitions differ. Start only an isolated core with scratch XDG paths and
LBC_SESSION, set that DYLD value on startup, then run the probe using both
`lbc exec` and `lbc run`. Reject a smoke if its core.log contains
`launchd startup failed`, even if child commands succeed.

These checks do not attempt LAN access or trigger privacy prompts. A GUI Mac
still needs a separately approved Local Network test.
