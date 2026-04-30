# 0001. Adopt design doc and ADR workflow

# Status
accepted

# Context

UsageGate has been developed without a formal way to capture design rationale.
Decisions get made in commits, chat, and the maintainer's head — none of which
are durable or discoverable for future contributors or future-self.

As the project moves toward an OSS + cloud model, we need:

- A review checkpoint before implementing non-trivial changes (to avoid rework)
- A durable record of *why* a given approach was chosen, not just *what* was built
- Documentation that lives with the code and goes through the same review pipeline as code

# Decision

Adopt a two-document workflow, with both kinds of docs living in the repo under `docs/`:

- **Design docs** (`docs/design-doc/`) — forward-looking proposals for non-trivial
  or hard-to-reverse changes. **Must be approved before implementation begins.**
- **ADRs** (`docs/adr/`) — one decision per file, recording decisions as they are
  made (often during implementation). Immutable once accepted; overturned by writing
  a new ADR that supersedes the old one.

The workflow, criteria for when a design doc is required, and templates are
documented in `CONTRIBUTING.md`. The PR template requires contributors to link
the relevant design doc and any ADRs added in the PR.

# Alternatives Considered

- **Status quo (no formal design docs / ADRs).** Rejected: rationale keeps getting
  lost; no audit trail; onboarding new contributors is harder than it needs to be.
- **Design docs only, no ADRs.** Rejected: design docs are forward-looking and
  approved upfront. They don't naturally capture the finer-grained decisions
  that emerge during implementation. Forcing them to would either bloat the doc
  or leave decisions undocumented.
- **ADRs only, no design docs.** Rejected: ADRs are snapshots of decisions
  already made. Without a design-doc step, there is no review checkpoint before
  implementation, so rework risk goes up for larger changes.
- **External wiki / Notion / Google Docs.** Rejected: design docs should be
  reviewed in the same PR pipeline as code, version-controlled alongside it,
  and survive independently of any external SaaS account.

# Consequences

- Positive:
  - Decisions are traceable, version-controlled, and reviewable in PRs
  - The design-doc approval gate reduces wasted implementation effort on larger changes
  - ADRs build up a narrative of *why* the system looks the way it does
- Negative / trade-offs:
  - Process overhead: contributors must write a doc before / during implementation
  - In the current MVP-priority phase, heavy upfront design can slow delivery.
    Mitigated by the explicit "when is a design doc required" criteria in
    `CONTRIBUTING.md` — small fixes / refactors / follow-ups skip the design doc
- Affected modules / operations:
  - All future feature work follows the flow in `CONTRIBUTING.md`
  - The PR template enforces linking design docs and ADRs

# References

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [docs/design-doc/template.md](../design-doc/template.md)
- [docs/adr/template.md](./template.md)
- [.github/PULL_REQUEST_TEMPLATE.md](../../.github/PULL_REQUEST_TEMPLATE.md)
