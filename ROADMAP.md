# Roadmap

autocommit is pre-stable. This document captures the intended trajectory of the project up to v1.0, ordered by release. **No dates are committed.** The project is sponsored on a best-effort basis by Nubster, and releases ship when they are ready.

The roadmap mirrors the [GitHub milestones](https://github.com/pierrick-fonquerne/autocommit/milestones) one-for-one. Each section here is the public, prose form of a milestone; each milestone groups the issues that must close before the release ships.

## Out of scope

autocommit generates scoped Conventional Commits messages offline from a staged diff. The following will never be in scope, regardless of demand:

- **Cloud inference.** autocommit only talks to a local model endpoint. No hosted API, no telemetry, no API keys.
- **Commit linting / enforcement.** Validating hand-written commits and gating CI is the job of cocogitto; autocommit only guarantees the messages it generates.
- **Release management.** Versioning, changelog graduation and tagging stay with cocogitto / cargo-release.
- **A general-purpose git client.** autocommit produces a commit message; it is not a porcelain replacement.

These boundaries are deliberate. If a feature request crosses one of them, it belongs in another project.

## v0.1.0: MVP core

**Goal.** A developer stages changes, runs `autocommit`, and gets a valid scoped Conventional Commit message generated from the diff by a local model.

**Scope:**

- `git` staged-diff reader.
- Deterministic `type` / `scope` classification from changed paths.
- Large-diff reduction (condense, then file-list-only).
- Ollama HTTP client (blocking, rustls), description-only prompt contract.
- `assemble` + Conventional Commits `validate` with a fail-open fallback.
- `autocommit` and `autocommit --dry-run` CLI commands.
- `autocommit.toml` configuration loader (global then local override).
- Unit and integration tests for the classification layer, runnable without a model.

## v0.2.0: Integration

**Goal.** autocommit is installable and usable transparently inside the everyday git workflow.

**Scope:**

- `autocommit init` interactive configuration wizard (language, endpoint, model, hook).
- `autocommit install-hook` and the `prepare-commit-msg` git hook.
- `autocommit config --get/--set`.
- PowerShell interactive installer (`install.ps1`): installs Ollama if missing, pulls the model, wires the hook.
- scoop manifest for `scoop install autocommit`.

## v0.3.0: Comfort

**Goal.** autocommit is pleasant beyond the terminal and easy to distribute.

**Scope:**

- IDE integration (VS Code extension) calling the same CLI.
- Advanced packaging and signed releases.
- Per-repository scope mapping presets.

## Post-1.0 backlog

The items below have been discussed but are not committed to any release. Each will require its own design pass.

- **Additional engines.** An abstraction allowing backends other than Ollama (e.g. an embedded llama.cpp build) behind the same endpoint contract.
- **Body and footer generation.** Optional commit body and `BREAKING CHANGE:` footer drafting, not just the header.
- **Team presets.** Shareable organisation-wide configuration profiles.

## How this roadmap is maintained

Changes to this document are made by pull request, with a `docs(roadmap):` Conventional Commit. The scope of any released version is locked once its tag is pushed; the scope of later releases stays adjustable until the previous release ships.

If you spot something missing, redundant or out of scope, open an issue against the relevant milestone and tag it `discussion`.
