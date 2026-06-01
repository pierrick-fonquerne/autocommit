# autocommit

> Generate scoped Conventional Commits messages offline, straight from your staged diff, with a local LLM.

[![CI](https://github.com/pierrick-fonquerne/autocommit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/pierrick-fonquerne/autocommit/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](./docs/MSRV_POLICY.md)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Status](https://img.shields.io/badge/status-pre--alpha-orange)](#status)
[![Made with Rust](https://img.shields.io/badge/made%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)

autocommit is a fast, offline command-line tool that turns your staged git diff into a valid scoped Conventional Commit message (`type(scope): description`). It runs entirely on your machine against a local model served by Ollama: no cloud, no API keys, no per-commit cost.

Deterministic heuristics decide the commit `type` and `scope` from the changed paths; a small local model drafts only the human-readable description; a Conventional Commits validator guarantees the grammar. If the model is slow or unavailable, autocommit degrades gracefully to a deterministic message and **never blocks your commit**.

## Status

🚧 **Pre-alpha.** The repository standard is in place and the design is frozen; the engine is being implemented. Nothing is published yet.

| Capability | Phase 1 | Phase 2 | Phase 3 |
| --- | --- | --- | --- |
| Staged diff reading | ⏳ | ⏳ | ⏳ |
| Deterministic `type` / `scope` classification | ⏳ | ⏳ | ⏳ |
| Large-diff reduction | ⏳ | ⏳ | ⏳ |
| Local LLM description drafting (Ollama) | ⏳ | ⏳ | ⏳ |
| Conventional Commits validation + fail-open | ⏳ | ⏳ | ⏳ |
| `autocommit` CLI with `--dry-run` | ⏳ | ⏳ | ⏳ |
| `autocommit init` configuration wizard | — | ⏳ | ⏳ |
| `prepare-commit-msg` git hook | — | ⏳ | ⏳ |
| PowerShell installer + scoop manifest | — | ⏳ | ⏳ |
| IDE integration | — | — | ⏳ |

See the [ROADMAP](./ROADMAP.md) for the milestone breakdown and the [CHANGELOG](./CHANGELOG.md) for the detailed history.

## Quick start

> Pre-alpha: the commands below describe the intended experience and are not all wired yet.

Prerequisites: a working [Ollama](https://ollama.com) endpoint (native Windows install or a Docker container) and a git repository.

```powershell
# 1. Install (scoop manifest shipped in Phase 2)
scoop install autocommit

# 2. Pull a local model (Apache-2.0, commercial-use friendly)
ollama pull codestral-mamba   # falls back to mistral:7b if unavailable

# 3. Configure once (language, endpoint, model, hook)
autocommit init

# 4. Use it
git add .
autocommit              # proposes "type(scope): description", asks for confirmation
autocommit --dry-run    # prints the message without committing
```

With the git hook installed, a plain `git commit` pre-fills the message for you to review.

## How it works

```
git diff --cached
  -> classify   (type + scope, deterministic, no LLM)
  -> summarize  (truncate/condense large diffs)
  -> draft      (local LLM writes the description only)
  -> assemble   ("type(scope): description")
  -> validate   (Conventional Commits guard, fail-open)
```

The model is configurable: autocommit talks to any Ollama endpoint over HTTP, whether it runs natively on Windows or inside Docker.

## Why autocommit

- **Offline and free.** No cloud inference, no API keys, no per-commit billing.
- **Deterministic where it matters.** The commit grammar, type and scope come from code, not from a probabilistic model.
- **Fail-open.** A commit tool must never get in your way; if the model is down, you still get a valid message.
- **Fast.** A native Rust binary with a cold start measured in milliseconds, fit for a `prepare-commit-msg` hook.

## What autocommit is **not**

- **Not a commit linter.** Validation guarantees the message it generates; enforcing conventions on hand-written commits is the job of tools like cocogitto.
- **Not a cloud assistant.** It deliberately avoids hosted models.
- **Not a release manager.** Versioning, changelog and tagging stay with cocogitto / cargo-release.

## Contributing

Contributions are welcome. Please read [`CONTRIBUTING.md`](./CONTRIBUTING.md) for the workflow and conventions, and [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md) for the community guidelines. For vulnerability reports, see [`SECURITY.md`](./SECURITY.md).

Stability and versioning are documented in [`docs/SEMVER_POLICY.md`](./docs/SEMVER_POLICY.md) and [`docs/MSRV_POLICY.md`](./docs/MSRV_POLICY.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

Copyright © Pierrick Fonquerne.
