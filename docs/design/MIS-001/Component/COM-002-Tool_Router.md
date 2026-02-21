# Component (Subsystem): COM-002 Tool Router

Dispatches tool requests to the appropriate subsystem, enforces execution semantics for chaining, and coordinates response shaping.



## Attributes

- **default_chain_behavior**: "stop-on-failure"
- **execution_modes**: ["sync","async"]


## Links

- composes [COM-003](COM-003-Workspace_Indexer.md)
- composes [COM-005](COM-005-Context_Provider.md)
- composes [COM-007](COM-007-File_Operations_Engine.md)
- composes [COM-008](COM-008-Plan_Manager.md)
- composes [COM-009](COM-009-Task_Runner.md)
- composes [COM-010](COM-010-Job_Manager.md)
- composes [COM-012](COM-012-Capability_Registry.md)
- composes [COM-013](COM-013-Log_Manager.md)
- composes [COM-014](COM-014-Environment_Variable_Manager.md)
- composes [COM-015](COM-015-Chain_Orchestrator.md)
- composes [COM-016](COM-016-Conformance_Runner.md)
- implements [FEA-011](../Feature/FEA-011-Action_Chaining_Orchestration.md)
- executes [STM-002](../State_Machine/STM-002-Tool_Router_State_Machine.md)
- runs on [NOD-001](../Node/NOD-001-Workspace_Host_Machine.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-21T00:00:00Z | copilot | create |
| 2026-02-21T02:15:00Z | copilot | change |
