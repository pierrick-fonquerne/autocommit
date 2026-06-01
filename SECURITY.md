# Security policy

## Supported versions

autocommit follows the [semver policy](docs/SEMVER_POLICY.md). During the 0.x phase, only the latest minor release receives security fixes.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

The supported window will be widened once autocommit reaches 1.0.

## Threat model

autocommit runs locally. It reads the staged git diff, sends a reduced form of it to a **local** model endpoint (Ollama, native or in Docker), and writes a commit message. The threat model assumes:

- The Ollama endpoint is operated by the developer or their organisation and is trusted.
- Diff content may contain secrets; autocommit never transmits it off the configured local endpoint and never logs raw diff content.

## Reporting a vulnerability

If you find a security vulnerability in autocommit, please **do not** open a public issue. Disclosure rules:

1. Email a detailed report to **security@nubster.com** with the subject prefix `[autocommit security]`.
2. The report should include:
   - A description of the vulnerability and the attacker model.
   - Affected versions.
   - Reproduction steps or a proof of concept.
   - The impact you anticipate (secret leak, command injection, denial of service, etc.).
   - Suggested mitigation if you have one.
3. You will receive an acknowledgement within **7 calendar days**. If you do not, please follow up at the same address.
4. We will work with you to validate, scope and remediate the issue. A coordinated disclosure timeline will be agreed in writing. The default embargo period is **90 days** from acknowledgement.
5. Once a fix is published, you will be credited in the release notes unless you prefer to remain anonymous.

## Out of scope

The following are explicitly **out of scope** for vulnerability reports:

- Issues in unsupported versions.
- Vulnerabilities in third-party dependencies that are already publicly disclosed and tracked upstream. Report them to the upstream project.
- Reports based on theoretical attacks without a working proof of concept.
- Exposure resulting from pointing autocommit at an untrusted or remote model endpoint, which is outside the supported configuration.

## Public security advisories

Confirmed and fixed vulnerabilities are published on the Security Advisories page of the repository. RustSec advisories are also coordinated for severe issues when applicable.
