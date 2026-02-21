# Threat Diamond: THD-001 Workspace Boundary Escape

A malicious/compromised agent attempts to read or write outside the configured workspace boundary (e.g., via path traversal, symlink tricks, or ambiguous path normalization).



## Attributes

_No attributes defined._

## Links

- threatens [AST-003](../Asset/AST-003-Nonworkspace_Resources.md)
- threatens [AST-001](../Asset/AST-001-Workspace_Contents.md)
- involves [ADV-001](../Adversary/ADV-001-Malicious_or_Compromised_Agent.md)
- uses [THC-001](../Threat_Capability/THC-001-Prompt_Injection_and_Tool_Misuse.md)
- impacts [VIC-001](../Victim/VIC-001-Developer_User.md)
- creates [RIS-001](../Risk/RIS-001-Unauthorized_Data_Disclosure.md)
- creates [RIS-004](../Risk/RIS-004-Boundary_Bypass_via_SymlinksJunctions.md)
- creates [RIS-005](../Risk/RIS-005-NonAtomic_File_Replace_Due_to_Locking.md)
- creates [RIS-006](../Risk/RIS-006-Nondeterministic_Output_Across_OSFilesystems.md)
- creates [RIS-009](../Risk/RIS-009-Path_Encoding_NonUTF8_Filenames_Break_Protocol.md)
- creates [RIS-003](../Risk/RIS-003-Workspace_Corruption.md)


## Version

## Audit Log

| Timestamp | Editor | Change |
|-----------|--------|--------|
| 2026-02-20T00:15:00Z | copilot | create |
| 2026-02-21T01:16:57Z | copilot | change |
