# Contributing

For local setup and commands, see [README.dev.md](./README.dev.md).
This document describes **how to land new features and design changes**.

## Implementation Flow

```
  ┌─ Decide whether a design doc is required
  │
  ├─ Required → write under docs/design-doc/ → review → approve → start implementing
  │
  └─ Not required → start implementing

  While implementing, add an ADR under docs/adr/ each time a design decision is made

  → Open a PR (link the design doc and any related ADRs)
```

### 1. Decide whether a design doc is required

Write a design doc **before** implementing if any of the following apply:

- Changes a public API, DB schema, config file, or other externally visible interface
- Hard to roll back (e.g. requires data migration)
- Spans multiple modules / introduces new structure
- Adds a new external dependency (middleware / SaaS / library)

Otherwise (small bug fixes, internal refactors, work that follows an existing
design), no design doc is needed — a PR description is enough.

### 2. Write the design doc

- Copy `docs/design-doc/template.md` to `docs/design-doc/<feature-name>.md`
- Open a PR for review
- **Do not start implementation before approval** — this is what prevents rework
- Once approved, update the Status to `approved` and move on to implementation

### 3. Implement

As design decisions come up during implementation, record each one as an ADR.
A "design decision" here means things like:

- The algorithm / data structure / middleware you chose, and why
- Trade-offs someone might later ask "why did you do it this way?" about
- Branch points where you want to preserve the alternatives you considered

Use ADRs to capture the finer-grained decisions that didn't make it into the
design doc.

### 4. Add an ADR

- Copy `docs/adr/template.md` to `docs/adr/NNNN-kebab-case-title.md`
  - `NNNN` is a zero-padded sequence number (e.g. `0001-use-valkey-for-rate-limit.md`)
- **One decision per file.** Don't bundle multiple decisions into one ADR
- Don't edit ADRs after they're accepted. If a decision is overturned, write a
  new ADR and set the old one's Status to `superseded`

### 5. Open the PR

- If you wrote a design doc, link it from the PR description
- Link any ADRs you added
- Run the [pre-commit checks](./CLAUDE.md) (`cargo fmt` / `cargo clippy` / `cargo test`)
