# Contributing to autocommit

autocommit is currently in **pre-alpha**: the repository standard and design are in place, and the engine is being implemented. The repository is public on [pierrick-fonquerne/autocommit](https://github.com/pierrick-fonquerne/autocommit) and contributions are welcome.

## Conventions

autocommit follows the Nubster general coding standards. In short:

- **Trunk-Based Development**, feature branches `feature/<issue>-<slug>` from `main`, never commit directly on `main`.
- **Conventional Commits**, all commit messages follow the `type(scope): description` format, enforced by `cog verify` in the commit-msg hook.
- **Rust style**, workspace lints `clippy::all` and `clippy::pedantic` set to `deny`, MSRV pinned in `rust-toolchain.toml` and `Cargo.toml`.
- **No competitor mentions**, the source code, commit messages, pull requests and documentation never name competing tools.
- **English on the public API, French on internal artifacts**, rustdoc comments and public types are written in English; commit messages, issues and project documentation may be written in French.

## Local setup

```bash
# Pin the Rust toolchain via rustup
rustup show

# Install local git hooks
lefthook install

# Pull a local model for end-to-end runs (Apache-2.0)
ollama pull codestral-mamba

# Run tests
cargo test --workspace --all-features
```

The deterministic classification layer is testable without a model: most unit and integration tests run with no Ollama endpoint at all.

## Discussion before code

Until v0.1.0, all design decisions go through a `discussion/` thread on the repository before any pull request is opened. This includes the CLI surface, the configuration schema, the classification heuristics and the prompt contract with the local model.

## Contributor License Agreement

Contributions to this project are governed by the Nubster Contributor License Agreement, hosted at [github.com/nubster-opensources/cla](https://github.com/nubster-opensources/cla).

On your first pull request, the CLA Assistant bot will automatically prompt you to sign the CLA. Once signed, your signature applies to all current and future contributions.

The CLA is a license grant (not a copyright assignment): you keep the copyright on your contributions and grant Nubster a broad license to use, sub-license, and re-license them.

## License

By contributing, you agree that your contributions are dual-licensed under the [MIT License](./LICENSE-MIT) and the [Apache License, Version 2.0](./LICENSE-APACHE), at the user's option.

Copyright © Nubster.
