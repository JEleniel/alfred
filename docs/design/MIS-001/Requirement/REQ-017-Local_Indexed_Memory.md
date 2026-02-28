# Requirement: REQ-017 Local Indexed Memory

Alfred MUST provide a local, indexed, searchable memory store that supports CRUD for individual facts and full-text search.



## Attributes

- **offline_only**: true
- **persistence**: "local"
- **tool_surface**: ["memory"]


## References

_No references defined._

## Links

- requires [CAP-014](../Capability/CAP-014-Local_Memory.md)
- imposes [CNS-006](../Constraint/CNS-006-No_External_Service_Dependencies.md)
- imposes [CNS-010](../Constraint/CNS-010-Permission_Model_and_Guardrails.md)
- imposes [CNS-012](../Constraint/CNS-012-Service_Level_Objectives.md)
- imposes [CNS-015](../Constraint/CNS-015-Deterministic_Output_Normalization.md)
- imposes [CNS-005](../Constraint/CNS-005-Deterministic_Error_Taxonomy.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-22T12:44:59Z | copilot | create |
