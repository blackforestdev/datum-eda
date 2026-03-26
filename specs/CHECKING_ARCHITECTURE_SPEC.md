# Checking Architecture Specification

## 1. Purpose

Defines the shared contract between ERC and DRC while preserving them as
separate checking engines with different inputs and invariants.

---

## 2. Domains

```rust
pub enum CheckDomain {
    ERC,
    DRC,
}
```

- ERC consumes schematic connectivity and electrical semantics
- DRC consumes board geometry, board connectivity, and physical rules

---

## 3. Shared Result Surface

```rust
pub enum CheckSeverity {
    Error,
    Warning,
    Info,
}

pub struct CheckSummary {
    pub errors: u32,
    pub warnings: u32,
    pub infos: u32,
    pub waived: u32,
}
```

Both ERC and DRC reports must expose:
- `passed`
- `violations`
- `summary`
- stable violation ordering for deterministic output

---

## 4. Shared Waiver Model

```rust
pub struct CheckWaiver {
    pub uuid: Uuid,
    pub domain: CheckDomain,   // ERC | DRC
    pub target: WaiverTarget,
    pub rationale: String,
    pub created_by: Option<String>,
}

pub enum WaiverTarget {
    Object(Uuid),
    RuleObject {
        rule: String,
        object: Uuid,
    },
    RuleObjects {
        rule: String,
        objects: Vec<Uuid>,   // sorted by UUID for deterministic matching
    },
}
```

- Waivers are authored data
- Waivers target a specific checking domain
- Waivers never delete findings; they suppress failure state only
- Waiver application must be explicit in reports
- `RuleObjects.objects` must be stored in deterministic UUID order
- Waiver matching is identity-based, not name-based
- Imported object renames do not invalidate waivers if UUID identity is stable

M2 minimum:
- waivers may target exact object UUIDs or exact rule/object tuples
- waived findings remain visible but do not fail the check
- waivers must serialize deterministically

---

## 5. Cross-Domain Checks

Cross-domain checks are not part of ERC or DRC by default.
They belong to synchronization/comparison subsystems unless explicitly
promoted into a dedicated third checking domain later.

Examples:
- schematic intent not realized on board
- board object exists with no schematic origin
- constraint propagation mismatch

M2 excludes these checks from pass/fail.

---

## 6. CLI/MCP Exposure

M2 required commands/tools:
- `run_erc`
- `run_drc`
- `get_violations`
- `explain_violation`

`explain_violation` must accept or infer the checking domain so results are
interpreted against the correct graph and location type.
