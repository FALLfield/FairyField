# FairyField pre-OMX migration plan

_Authoritative English migration plan for FairyField V2. Last reconciled with `main` at `238c1368ffb74dc49c220c51bf820a8b5fd78d55` on 2026-07-12._

---

## 🎯 Mission and non-negotiable rules

This migration preserves the published V1 line, establishes an OMX-ready V2 development line, and replaces phase claims with testable product evidence. It is not a bulk rewrite, a tool-count project, or a promise of feature parity with Hermes Agent.

The English files under `docs/omx/` are the only authoritative version of this plan. A local Chinese review translation may exist outside the tracked documentation tree. It is intentionally non-authoritative: requirement changes must be made to this English document first.

The migration is complete only when all of the following are true:

1. V1 source, documentation, and accepted fixes have a pushed, recoverable Git checkpoint.
2. The existing `v1.0.0` tag remains immutable and is never repointed at V2 work.
3. V2 starts on an independent branch and uses a pre-release version.
4. OMX orchestration, project rules, ownership, and test evidence have one unambiguous source of truth.
5. Every public capability is labelled `Verified`, `Beta`, `Experimental`, `Scaffold`, or `Removed` from evidence, not from its file count.
6. All testable behaviour is exercised automatically before a human is asked to test the desktop interface.
7. Core task correctness is complete before voice naturalness and result-presentation polish become the main workstream.

> **Safety rule:** A passing build, a visible UI control, or a successful model response never proves that a user task completed. Completion requires verified side effects, expected failure behaviour, and requirement-level evidence.

## 📍 Reconciled starting point

| Item | Current fact | Migration consequence |
|---|---|---|
| Main and remote | `main` equals `origin/main` at `fd5b332` | Start V2 work from the pushed checkpoint, not a private working tree |
| Pre-V2 checkpoint | Annotated tag `snapshot/pre-omx-v2-20260712` resolves to `fd5b332` | Recovery branch and snapshot preserve the handover point |
| V1 tag | `v1.0.0` resolves to `25ecb6c`; seven commits behind `main` | Preserve it as a historical release marker |
| Working tree at checkpoint | Clean before V2 work began | Each later integration needs its own reviewed checkpoint |
| Application metadata | npm, Cargo, and Tauri still report `1.0.0` | Bump atomically only when the V2 branch is intentionally declared |
| Existing agent loop | Hermes-style observation loop with bounded execution | Reuse tested pieces; do not call it Hermes parity |
| Project OMX prompt rule | Tracked `AGENTS.md` defines an initial capacity of up to 12 concurrent child agents with evidence-based scaling | Re-check the policy after setup or guidance regeneration |
| Project OMX config | Tracked `.codex/config.template.toml` defines the initial `[agents] max_threads = 12` baseline; ignored local config merges it with machine-specific plugin state | Validate the effective runtime and use waves when the full configured capacity is not supported |
| Test reality | Automated tests exist, but there is no complete product capability benchmark | Build a deterministic AgentBench before treating feature claims as release evidence |

The following boundaries are compatibility assets, not disposable implementation details:

- Application identifier `com.fallfield.fairyfield`, unless a deliberate upgrade break is approved.
- `~/.fairyfield/config.json`, `user.json`, `secrets.json`, `fairyfield.db`, and `models/` read and migration paths.
- SQLite/FTS verbatim-memory semantics, temporal fact invalidation, user correction, and existing IPC contracts until a tested replacement exists.
- `soul/SOUL.md`, `soul/identity.txt`, the default character asset, icons, licenses, lockfiles, and model validation logic.

## 📋 Migration sequence

```mermaid
flowchart LR
    accTitle: FairyField V2 Migration Sequence
    accDescr: The migration preserves V1 first, establishes OMX governance and automated evidence, then builds the V2 functional core before voice refinement and release work.

    freeze([📋 Freeze V1 facts]) --> checkpoint[📦 Push recovery checkpoint]
    checkpoint --> governance[⚙️ Establish OMX governance]
    governance --> benchmark[🧪 Build AgentBench]
    benchmark --> core[🔧 Deliver functional core]
    core --> voice[💬 Refine voice and results]
    voice --> release([🚀 Release candidate])

    classDef process fill:#dbeafe,stroke:#2563eb,stroke-width:2px,color:#1e3a5f
    classDef success fill:#dcfce7,stroke:#16a34a,stroke-width:2px,color:#14532d

    class checkpoint,release success
    class freeze,governance,benchmark,core,voice process
```

| Phase | Outcome | Exit evidence |
|---|---|---|
| P0 | Freeze the V1 facts | Inventory, secret scan, and recorded test baseline |
| P1 | Preserve a recoverable checkpoint | Pushed branch, annotated tag, and clean-clone recovery exercise |
| P2 | Establish OMX governance and scale | One policy chain, an initial 12-child capacity with runtime-supported scaling, and clear worktree rules |
| P3 | Establish automated product evidence | AgentBench, headless integration coverage, and capability truth labels |
| P4 | Decide architecture and product order | ADRs for runtime, security, memory, voice, and transparency |
| P5 | Create a V2 engineering baseline | Pre-release branch, CI, contracts, and status document |
| P6 | Admit work to the V2 backlog | Migration acceptance checklist complete |

## 🔐 P0 and P1: preserve V1 before changing it

### Freeze the actual state

Before any cleanup or V2 refactor, record the following in a migration checkpoint note:

```bash
git rev-parse --show-toplevel
git branch --show-current
git rev-parse HEAD
git describe --tags --always --dirty
git status --short
git diff --stat
git diff --cached --stat
git stash list
git remote -v
```

Classify each changed or untracked path before staging it:

| Class | Meaning | Required handling |
|---|---|---|
| A | Verified V1 fix or test | Commit as an auditable source change |
| B | Migration or truth documentation | Commit separately from source behaviour |
| C | Useful experiment | Preserve in an experiment branch, not a release claim |
| D | Local runtime state | Ignore; never commit |
| E | Secret or private user data | Rotate or remove before any push |
| F | Rebuildable or obsolete material | Delete only after the checkpoint is recoverable |

Do not use `git add .` or `git add -A` to create a migration checkpoint. Review each staged path with `git diff --cached -- <path>`.

### Verify the baseline without a release build

Run the test and development checks that apply to the current checkpoint:

```bash
npm run test
npm run build
cd src-tauri
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
cargo check --features sherpa-onnx
cargo test --features sherpa-onnx
cargo clippy --features sherpa-onnx -- -D warnings
```

Do not run `npm run tauri build` during normal migration or feature verification. A controlled `npm run tauri dev` smoke test is allowed after automated checks; close the application and all child processes afterward.

### Create the recovery point

Use reviewable commits and immutable references, for example:

```text
archive/v1-source-candidate-YYYYMMDD
snapshot/pre-omx-v2-YYYYMMDD
v2/omx-redevelopment
```

1. Create the archive branch from the accepted V1 source candidate.
2. Commit source fixes, truthful documentation, and OMX governance separately.
3. Re-run the baseline matrix and create an annotated snapshot tag.
4. Push the archive branch and tag.
5. Clone the remote into a temporary directory, check out the tag, and reproduce the baseline checks without local secrets or untracked source.
6. Create `v2/omx-redevelopment` from the verified tag.

Keep existing stashes until the pushed checkpoint and clean-clone recovery exercise both succeed. A stash is a convenience, not a remote backup.

## 👥 P2: OMX governance and scalable agent capacity

### One policy chain

After the checkpoint, rules are interpreted in this order:

1. Platform, security, and user authorization constraints.
2. The root `AGENTS.md` OMX operating contract.
3. V2 product, architecture, roadmap, and current-status documents.
4. An approved task specification and ADR.
5. Historical phase documents as reference only.

Retire contradictory rules instead of letting them coexist. In particular:

| Legacy rule | V2 decision |
|---|---|
| Start every related agent for every task | Replace with an explicit capacity plan based on independent work packages and evidence needs |
| One agent must handle a task alone | Replace with bounded, parallel implementation and review lanes where the task graph justifies them |
| Every agent always needs a permanent worktree | Use one clean temporary worktree per concurrent write task; read-only work may remain isolated logically |
| Update many phase documents after every change | Update `STATUS.md` per integration, roadmap per milestone, and README/CHANGELOG per release |
| Check Hermes or MemPalace before every edit | Consult the relevant reference only for agent, tools, memory, safety, or voice design work |
| Superpowers is mandatory | Remove it from the V2 workflow |
| Zero Python is permanent doctrine | Decide packaged-runtime dependencies through an ADR and measured product constraints |
| A file or a test count proves a feature | Require requirement-to-evidence verification and failure-path tests |

### Initial capacity: twelve concurrent child agents, dynamically scalable

V2 starts with an **initial configured complex-mission capacity of up to 12 concurrent child agents**, plus the lead/integrator. Twelve is an improvement over the current six-child default and a capacity baseline, not a mandatory spawn count or a permanent project ceiling. The lead activates only the smallest justified set of independent lanes. The actual ceiling is the supported OMX runtime capacity that the lead has validated against the task graph, model budget, integration bandwidth, and local machine resources.

The tracked `AGENTS.md`, portable `.codex/config.template.toml`, and effective ignored local OMX configuration must agree before the initial 12-agent model is used. The portable baseline is:

```toml
[agents]
max_threads = 12
max_depth = 2
```

Copy or merge the portable template into the ignored local `.codex/config.toml` before applying the setting. Keep local marketplace paths, hook trust, runtime state, and authentication out of Git. Treat `omx setup --scope project --merge-agents` as a mutating regeneration: stop active teams, use a clean disposable worktree, and inspect the `AGENTS.md` and local config diff before adopting it. Do not hand-edit setup-owned native-agent files. Re-read the tracked guidance after regeneration because setup or doctor success alone does not prove policy alignment or safe concurrency. The installed OMX generator may still supply a six-child default when settings are absent, so the explicit project configuration and root policy must be rechecked together. Interpret `max_threads` as a global fleet budget, not twelve children per parent. If the installed OMX version rewrites a conflicting limit or cannot honour 12 threads, record that as a blocker and schedule the mission in waves. If hardware, budget, and the verified runtime support more than 12, raise the project configuration through the same documented path; never silently exceed the verified runtime limit.

### Capacity planning rules

| Mission shape | Child-agent budget | Required structure |
|---|---:|---|
| Small, local fix | 0–2 | One executor; optional tester or reviewer |
| Bounded feature | 3–6 | One planner, 1–3 independent builders, testing and review coverage |
| Cross-module feature | 6–9 | One plan owner, independent module lanes, one test lane, at least two independent reviews |
| Complex mission | 10–runtime maximum | One plan owner, optional architect, enough disjoint build, test, and review lanes to match the approved task graph |

The lead computes the budget before spawning agents:

```text
child budget = min(runtime-supported threads, task budget, independent build lanes + test lanes + review lanes + necessary planning/architecture lanes)
```

Only count a lane if it has an owner, a bounded scope, a worktree or read-only boundary, acceptance evidence, and a merge/review path. Several agents may implement, test, or review one mission; only one lead owns integration and completion claims.

Planning has one accountable plan owner. Independent researchers or architects may provide evidence, but they do not produce competing plans. High-risk work receives multiple independent reviewers: one behaviour/security reviewer and one test/requirements reviewer; add a third reviewer for data migration, credentials, or destructive operations.

### Documentation and generated-file boundary

Track the portable OMX configuration template, the root `AGENTS.md` policy, and non-rebuildable project templates. Keep resolved local config, runtime state, logs, sessions, caches, local marketplace paths, databases, models, and temporary reports out of Git. Determine whether generated native-agent definitions are tracked only after a clean-clone regeneration exercise proves the boundary.

The V2 authoritative documentation set is deliberately small:

| Document | Purpose | Update trigger |
|---|---|---|
| `README.md` | Publicly true product entry point | Release candidate |
| `AGENTS.md` | OMX and engineering contract, including the initial 12-child capacity and scaling conditions | Workflow or supported-capacity change; re-check after regeneration |
| `docs/v2/PRODUCT.md` | Users, outcomes, non-goals, acceptance | Product decision |
| `docs/v2/ARCHITECTURE.md` | Boundaries, contracts, ADR index | Architecture decision |
| `docs/v2/ROADMAP.md` | Milestones and dependencies | Milestone change |
| `docs/v2/STATUS.md` | Current facts, blockers, ready work | Each integration batch |
| `CHANGELOG.md` | Released user-visible changes | Release |

## 🧪 P3: automated capability evidence

### Test everything that can be tested without a human

Human testing remains essential for subjective voice quality, microphone comfort, device permissions, and companion feel. It must be the final confirmation layer, not the first detector of ordinary failures.

Before a task is handed to a human, automation must cover every practical non-visual behaviour through command-line, contract, integration, or headless UI tests:

| Layer | Runs without a human | Evidence required |
|---|---|---|
| Unit | Parsing, Unicode, policy, state transitions | Deterministic assertions |
| Contract | Rust/TypeScript events, provider/tool/MCP schemas | Producer and consumer agree |
| Integration | Agent plus mock provider, real filesystem, real SQLite | Expected events and artefacts |
| Headless UI | Chat state, approval, cancellation, error rendering | Stable rendered state or event trace |
| Capability benchmark | Multi-step task scripts with fixtures | Requirement-level pass/fail report |
| Gated live smoke | Real provider, browser, or account only when configured | Explicit opt-in report, no destructive target |
| Manual acceptance | Voice device, naturalness, visual feel, installation | Recorded human result and environment |

Every production tool must declare a deterministic test mode or a contract mock. A credentials- or hardware-dependent tool requires both a mock contract test and an opt-in live smoke test. A tool without either remains `Experimental` and is not advertised as usable.

### Build the Hermes-inspired AgentBench

Use the sibling Hermes checkout as a source of capability categories and adversarial cases, not as a code-copying target. Its relevant patterns include tool observation, retries, file safety, browser/web handling, interrupt paths, coding evidence, memory boundaries, approval, and process lifecycle. FairyField's benchmark must be Rust/TypeScript-native and specific to the desktop product.

Create a deterministic suite such as:

```text
tests/agent_bench/
  manifest.yaml
  fixtures/web/
  fixtures/files/
  fixtures/audio/
  providers/scripted/
  cases/
  reports/
```

Each case records an initial state, scripted provider responses, tool fixtures, expected event sequence, expected artefacts, expected safety decision, and a requirement-level verdict. Network access, personal files, credentials, and a real Desktop are replaced by controlled fixtures unless the case is explicitly an opt-in smoke test.

`TST-001` must make AgentBench executable rather than descriptive. It owns the root runner command `npm run test:agent-bench`, the manifest schema, stable case-ID rules, and CI report generation. Case IDs use `<FAMILY>-<NNN>` such as `WEB-UTF8-001`; Assurance/Release owns the runner and manifest, while the owning module maintains its fixtures and cases. The manifest maps every advertised `capability_id` to one or more deterministic cases and, where appropriate, an opt-in live smoke. The runner produces a machine-readable CI artefact containing the commit, case IDs, duration, pass/fail/skipped state, and requirement verdict.

| Capability family | Minimum automated cases before it can be `Verified` |
|---|---|
| Conversation and planning | Direct answer, plan creation, budget exhaustion, truthful finalisation |
| Tool observation and repair | Tool result continues the loop, recoverable failure, duplicate-call stop |
| Web and content safety | Chinese/English/Japanese Unicode, timeout, empty result, hostile page content |
| File and artefact work | Approved Desktop path, atomic write, overwrite policy, read-back verification |
| Permission and command safety | Path escape, denied command, dangerous Git action, secret redaction |
| Cancellation and recovery | Cancel during provider/tool work, no post-cancel write, retry with new hypothesis |
| Memory | Save, restart, retrieve, correction, deletion, prompt isolation |
| Coding work | Scoped worktree, test failure feedback, diff collection, cancellation, reviewer evidence |
| Voice pipeline | Recorded Mandarin/English fixtures, VAD states, partial/final ASR, TTS text cleaning |

The first functional-core benchmark has at least 24 deterministic cases. This is a coverage floor, not a pass-rate target: every required deterministic P0/P1 case must pass. An optional live case that is skipped does not earn a capability the `Verified` label. Expand the suite whenever a bug escapes to human testing. A test case is not counted merely because it starts a tool; it passes only when its expected artefact, event sequence, safety decision, and final user-facing result all match.

### Test gates

| Gate | When it runs | Minimum scope |
|---|---|---|
| Fast gate | Every task before review | Targeted regression, formatter, type/compiler checks |
| Affected-system gate | Every integration | Related frontend, Rust, feature flags, contracts, and AgentBench cases |
| Full headless gate | Integration branch and nightly | Entire automated matrix with fixtures |
| Optional live gate | Provider/tool release candidate | Explicit environment, isolated account or sandbox, no destructive side effects |
| Human gate | After all automated gates | Real microphone, naturalness, visual interaction, installation and upgrade |

## ⚙️ P4 and P5: decide architecture, then build the functional core

### Required ADRs

| ADR | Decision | Required evidence |
|---|---|---|
| ADR-001 | In-process Rust versus local service boundaries | Startup, package size, crash isolation, platform cost |
| ADR-002 | Provider and streaming protocol | Tool calling, cancellation, retry, failover, and mock compatibility |
| ADR-003 | Tool permissions and artefact proof | Path containment, approval, audit, and false-success prevention |
| ADR-004 | Minimum long-term memory model | Provenance, correction, deletion, privacy, and retrieval quality |
| ADR-005 | Mandarin/English voice route | ASR accuracy, first audio, interruption, resources, and licences |
| ADR-006 | OMX/Codex worktree execution | Concurrency, conflict handling, recovery, and cleanup |
| ADR-007 | Platform and release scope | macOS path, signing, upgrades, and CI |
| ADR-008 | Task-result transparency and voice presentation | User-visible state, privacy, concise speech, and no raw reasoning disclosure |

### Functional-first milestone order

```text
M0  Governance, contracts, and AgentBench foundation
M1  Safe text task loop: plan, execute, verify, cancel, and report
M2  Controlled memory: provenance, retrieval, correction, export, and deletion
M3  Coding work: isolated worktree, coding runtime, tests, review, and recovery
M4  Voice and companion response: bilingual ASR/TTS, interruption, lip sync, and result presentation
M5  Limited integrations: only tools that pass the full registration gate
M6  Release candidate and public release
```

M4 is intentionally after M1 through M3. Device preflight and recorded-audio tests may start earlier, but TTS naturalness, speech pacing, and how Fairy narrates task results are refinement work after the functional core is reliably useful.

ADR-008 defines the future black-box boundary. The default product policy should expose user intent, planned action, permissions requested, high-level progress, verified artefacts, failures, and next action. It must not expose raw chain-of-thought, hidden model reasoning, secrets, or noisy internal logs. The screen may retain a compact evidence trail; spoken output should be a short, natural summary rather than a recital of tool calls or Markdown symbols.

### Create the V2 baseline

Create `v2/omx-redevelopment` from the recovery checkpoint and use `2.0.0-alpha.1` only after its engineering baseline is real. Establish:

1. A shared request/tool/voice event contract and error taxonomy.
2. Request-scoped cancellation, budgets, structured progress, and final evidence.
3. Mock provider/tool adapters that make AgentBench deterministic.
4. CI for frontend tests, Rust default and optional features, formatting, lint, secret scanning, licence checks, and AgentBench.
5. A registry gate: no tool becomes user-visible without manifest, permissions, test mode, failure semantics, and evidence.
6. `docs/v2/STATUS.md` with real capabilities, known limitations, current blockers, and the next ready task.

## ✅ Migration admission and rollback

Enter day-to-day OMX development only after every item is true:

- [ ] V1 archive branch and annotated snapshot tag are pushed and recoverable from a clean clone
- [ ] `v1.0.0` is unchanged
- [ ] Secrets, user data, models, logs, and build artefacts are excluded from the checkpoint
- [ ] OMX setup and doctor pass on a clean checkout
- [ ] The portable configuration template, effective local runtime configuration, and generated guidance permit an initial configured capacity of up to 12 child agents; a bounded twelve-lane smoke has recorded its limits; and a documented wave plan is in place for any unverified or over-capacity mission
- [ ] Every parallel lane has a bounded scope, base commit, clean worktree, owner, test evidence, and integration owner
- [ ] One English authority chain exists; local translations cannot override it
- [ ] The AgentBench manifest, deterministic fixtures, and first 24 cases are runnable in CI
- [ ] A capability inventory labels every public claim honestly
- [ ] Required ADRs have decisions or explicit owners, deadlines, and blockers
- [ ] V1 configuration and database copies open through tested V2 compatibility paths
- [ ] CI uses the declared Rust version and audits the actual `src-tauri/Cargo.lock`
- [ ] `STATUS.md` identifies the first ready task and its acceptance evidence

Stop and preserve evidence if a checkpoint cannot be restored, a baseline test regresses without explanation, a secret is found, generated OMX rules disagree with the configured agent capacity, or an ADR conflict blocks implementation. Roll back by checking out the pushed snapshot or archive branch; never use destructive reset to erase unclassified work.

The next operating document is [the OMX development playbook](DEVELOPMENT_PLAYBOOK.md).
