# Control: CTL-003 Safe Task Execution Policy

Constrain task execution to safe defaults: workspace working directory, environment scrubbing, command allow/deny rules, output limits, timeouts, and a policy that prevents Alfred from becoming a general-purpose remote execution agent.



## Attributes

_No attributes defined._

## Links

- protects [AST-002](../Asset/AST-002-Secrets_and_Credentials.md)
- protects [AST-003](../Asset/AST-003-Nonworkspace_Resources.md)
- enforces [CNS-010](../Constraint/CNS-010-Permission_Model_and_Guardrails.md)
- enforces [CNS-006](../Constraint/CNS-006-No_External_Service_Dependencies.md)
- governs [DST-002](../Data_Store/DST-002-Job_and_Session_Store.md)
- governs [DST-003](../Data_Store/DST-003-Plan_Store.md)
- mitigates [RIS-001](../Risk/RIS-001-Unauthorized_Data_Disclosure.md)
- mitigates [RIS-002](../Risk/RIS-002-Host_Compromise.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-20T00:15:00Z | copilot | create |
