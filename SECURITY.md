# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately via GitHub's security advisory flow:
**[Report a vulnerability](https://github.com/MythologIQ-Labs-LLC/CodeGenome/security/advisories/new)**

Do not open public issues for security reports.

## Scope notes

- CODEGENOME is a **research prototype**; it is not hardened for untrusted inputs or multi-tenant deployment.
- The MCP server (`codegenome serve`) is a **local stdio service** intended to be launched by a local AI-assistant client. It has no network transport and no authentication layer.
- Write operations (reindex, belief assertion) pass through the governance write gate; reports of gate bypasses are especially valuable, as tamper-evidence and memory-poisoning resistance are core claims of this project.

## Supply chain

- Dependabot alerts and PRs are enabled.
- CI runs `cargo audit` (advisory) and gitleaks secret scanning on every PR.
- A pre-commit secret-scanning hook ships in `.githooks/` (see CONTRIBUTING.md for setup).
