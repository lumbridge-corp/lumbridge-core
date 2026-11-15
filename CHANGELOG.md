# Changelog

Changes to Lumbridge Core.

## [Unreleased]

## [0.1.0] - 2026-11-11

The first release of Lumbridge Core. It is built from source, on Linux and macOS.
`lbc --version` prints 0.1.19, the crate version.

- Binary and crate `lumbridge-core`, CLI `lbc`, environment variables `LBC_*` and
  `LUMBRIDGE_*`, folders `lumbridge-core`, bundle identifier `com.lumbridgecorp.core`.
- `lbc update` takes only `--from FILE`. There is no self-updater, no release download
  and no install script: nothing contacts any release or update service.
- The README has a quick start from a clean machine. Security reports go to security@lumbridgecorp.com.
- `scripts/scrub-check` fails on private addresses, home folders, email addresses and token formats in the tree.
