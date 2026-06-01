# Semantic Versioning policy

autocommit follows [Semantic Versioning 2.0.0](https://semver.org/) with explicit conventions for the 0.x phase.

## 0.x phase (pre-1.0)

While the major version is 0, breaking changes are allowed on a minor version bump:

- `0.1.x` -> `0.1.y` (patch): bug fixes, performance improvements, internal refactors, additive non-breaking changes. No change observable by a downstream user.
- `0.x.y` -> `0.X.0` (minor): may introduce breaking changes. Removed items must have been deprecated for at least one previous minor release whenever feasible.

Reasoning: autocommit ships early to gather feedback. Locking into Semver-strict semantics before the CLI surface is stable would prevent the changes we know we still need.

## 1.0 and beyond

Once 1.0 is reached, autocommit commits to strict Semver:

- Major (`X.0.0`): breaking changes to the public surface.
- Minor (`1.Y.0`): backwards-compatible additions.
- Patch (`1.x.Z`): backwards-compatible bug fixes.

## Public surface definition

Because autocommit ships as a binary, its public surface is its observable contract, not a Rust API:

- The set of subcommands and their flags (`autocommit`, `--dry-run`, `--hook`, `init`, `install-hook`, `config`).
- The `autocommit.toml` configuration schema (keys, types, defaults).
- The grammar of generated messages (Conventional Commits `type(scope): description`).
- Process exit codes.

Items that are explicitly NOT part of the public surface:

- The exact wording produced by the local model for a given diff (inherently non-deterministic).
- Internal module structure of the crate.
- Log line formatting at non-default verbosity.

## Deprecation cycle

When a flag, subcommand or config key is to be removed:

1. It is marked deprecated in the release that introduces the replacement, with a runtime warning.
2. It keeps working unchanged for the entire next minor cycle.
3. It is removed in the minor release after that, at the earliest, documented in CHANGELOG.md under `Removed`.

## Breaking change documentation

Every breaking change is announced in CHANGELOG.md under `Changed` or `Removed`, with the replacement and a migration note when the change is non-mechanical.

## MSRV

The MSRV (Minimum Supported Rust Version) is governed by [MSRV_POLICY.md](MSRV_POLICY.md). An MSRV bump is treated as a minor version bump.
