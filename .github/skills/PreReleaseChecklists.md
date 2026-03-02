# Pre-Release Review Checklists

## Approach

- **Risk-Reduction Focus**: Prioritize checks that reduce the probability and impact of production incidents.
- **Evidence-Driven Decisions**: Require verifiable evidence for release readiness, not assertions.
- **Security-First Release Posture**: Treat unresolved security uncertainty as a release blocker until risk is explicitly accepted.
- **Operational Readiness Focus**: Validate deployability, observability, rollback, and ownership before release.

## Review Criteria

### Release Scope and Change Control Checklist

- **Release Scope Is Explicit**: Included and excluded changes are clearly defined and traceable to approved work items.
- **Versioning Is Correct**: Version numbers and compatibility semantics match the actual change impact.
- **Change Freeze Compliance**: Release timing and approvals conform to freeze windows and governance requirements.
- **Dependency Change Visibility**: Critical dependency updates and risk implications are explicitly documented.
- **Approval Chain Is Complete**: Required approvers and decision owners are identified and recorded.

### Quality and Verification Checklist

- **Required Tests Are Green**: Unit, integration, and end-to-end suites required for release have passed.
- **Regression Risk Is Assessed**: Known regression vectors are tested or explicitly risk-accepted.
- **Performance and Capacity Validation**: Performance budgets and capacity assumptions are validated for expected load.
- **Security Verification Complete**: Required security checks (for example SAST, dependency, config, and policy checks) are complete.
- **Known Issues Are Dispositioned**: Open defects are resolved, deferred with owner/date, or explicitly accepted with rationale.

### Security and Compliance Checklist

- **No Critical Unresolved Vulnerabilities**: Critical security findings are fixed or formally accepted by authorized owners.
- **Secrets and Sensitive Data Controls**: Release artifacts and runtime configuration do not expose secrets or Non-Public Information.
- **Policy and Regulatory Compliance**: Applicable compliance, legal, and policy requirements are satisfied.
- **Auditability Is Preserved**: Release decisions, artifacts, and approvals are traceable and reviewable.
- **License and Third-Party Obligations**: License obligations and attribution requirements are met.

### Operational Readiness Checklist

- **Deployment Plan Is Actionable**: Deployment sequence, prechecks, and success criteria are explicit.
- **Rollback Plan Is Tested or Proven**: Rollback path is defined, feasible, and validated for time-to-recover expectations.
- **Observability Is Ready**: Required logs, metrics, traces, and alerts are in place and verified.
- **Runbooks and Ownership Are Clear**: Incident runbooks, escalation paths, and on-call ownership are current.
- **Environment Parity Risks Addressed**: Production differences from lower environments are identified and mitigated.

### Artifact and Distribution Checklist

- **Release Artifacts Are Reproducible**: Build outputs are reproducible from declared inputs and controlled toolchains.
- **Artifact Integrity Is Verifiable**: Checksums/signatures (where required) are generated and validated.
- **Configuration Is Versioned and Controlled**: Release-time configuration changes are reviewed, versioned, and auditable.
- **Distribution Metadata Is Correct**: Release notes, package metadata, and channels/targets are accurate.
- **Rollback Artifacts Are Available**: Prior known-good artifacts required for rollback are accessible.

## Things to Watch For

### Failure and Release Risk Multipliers (a.k.a. "Foot-Guns")

- **Last-Minute Scope Creep**: Unreviewed changes are introduced near the release boundary.
- **Evidence Gaps**: Required checks are marked complete without verifiable artifacts.
- **Untested Rollback Paths**: Rollback exists on paper but has not been validated.
- **Silent Environment Drift**: Production differs materially from test/staging assumptions.
- **Security Exception Creep**: Temporary exceptions accumulate without ownership or expiry.
- **Operational Ownership Gaps**: On-call and escalation responsibilities are unclear at release time.
- **Dependency Surprise**: Late dependency changes introduce unassessed risk.
- **Insufficient Communication**: Stakeholders are not informed of release scope, timing, or risk posture.

### Indications of Poor Pre-Release Readiness

- **Ambiguous Go/No-Go Criteria**: Release decision criteria are not explicit or measurable.
- **Unbounded Known-Issue List**: Deferred issues lack owner, impact statement, or resolution timeline.
- **Incomplete Audit Trail**: Approvals, evidence, or decision records are missing.
- **Missing Operational Artifacts**: Runbooks, dashboards, alerts, or rollback instructions are incomplete.
- **Inconsistent Release Metadata**: Versioning, changelog, and artifact metadata disagree.
- **Decision by Urgency Alone**: Schedule pressure overrides unresolved high-impact risk.

## Glossary

- Common review terms (for example "Pass", "Fail", "N/A", "Evidence", "Severity", and "P0"-"P3") are defined in the [Skills glossary](GLOSSARY.md).
- "Go/No-Go": A formal release decision checkpoint that evaluates readiness against explicit criteria.
- "Release Candidate": A build proposed for release, subject to final verification and approval.
- "Rollback Window": The period during which rollback can be executed with acceptable risk and impact.
- "Change Freeze": A controlled period during which changes are restricted to reduce release risk.
- "Release Evidence": Verifiable artifacts demonstrating readiness, such as test reports, approval records, and scan outputs.
- "Release Artifact": The deployable output produced by the release process.
