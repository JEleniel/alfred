# Control: CTL-007 Argv-Only Task Execution (No Shell Default)

Default to argv-based spawning (no shell) to reduce injection risk and ensure cross-platform behavior; require explicit opt-in for shell execution with clear portability warnings.



## Attributes

_No attributes defined._

## Links

- protects [AST-002](../Asset/AST-002-Secrets_and_Credentials.md)
- protects [AST-003](../Asset/AST-003-Nonworkspace_Resources.md)
- enforces [CNS-018](../Constraint/CNS-018-No_Shell_by_Default_for_Task_Execution.md)
- obstructs [THC-001](../Threat_Capability/THC-001-Prompt_Injection_and_Tool_Misuse.md)
- mitigates [RIS-007](../Risk/RIS-007-Shell_Quoting_Injection_Portability_Issues.md)
- mitigates [RIS-001](../Risk/RIS-001-Unauthorized_Data_Disclosure.md)
- mitigates [RIS-002](../Risk/RIS-002-Host_Compromise.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T01:16:57Z | copilot | create |
