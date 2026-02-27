# Security Policy

We take security seriously and appreciate responsible disclosure.

## Reporting a vulnerability

Do **not** report security vulnerabilities through public GitHub issues, pull requests, or discussions.

Use one of the private channels below:

- GitHub Security Advisories (preferred): open the repository’s **Security** tab, then **Advisories**, then create a new draft advisory.

If you cannot use GitHub Security Advisories for some reason, open a private message to the maintainers with the details (avoid public threads).

### What to include

Please include:

- A clear description of the issue and the affected component.
- Impact assessment (what an attacker can do).
- Steps to reproduce or a proof of concept.
- Version/commit information.
- Any mitigations or workarounds you are aware of.
- Relevant references (for example CVEs).

### What to expect

We will respond on a best-effort basis.

- We will acknowledge receipt when we can.
- If the report is confirmed, we will coordinate a fix and a disclosure timeline.
- We can credit reporters in the advisory on request.

## Supported versions

| Version   | Supported |
| --------- | --------- |
| `main`    | Yes       |
| `< 1.0.0` | No        |

## Scope

In scope (examples):

- Remote code execution, injection, and sandbox escapes.
- Authentication and authorization bypass.
- Sensitive data exposure.
- Privilege escalation.
- Supply chain and dependency integrity issues.

Out of scope (examples):

- Denial of service without a realistic security impact.
- Social engineering.
- Physical attacks.
- Vulnerabilities in third-party services not controlled by the project.

## Security updates

When a vulnerability is confirmed, we will:

- Develop and test a fix.
- Release a patch as soon as reasonably possible.
- Document the change in release notes.
- Publish a GitHub security advisory when appropriate.

## Bug bounty

Unless stated otherwise, this project does not offer a paid bug bounty.
