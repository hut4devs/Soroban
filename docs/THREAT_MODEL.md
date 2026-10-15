# THREAT MODEL

- Status: Draft, awaiting review (required before any Testnet deployment)
- Date: 2026-10-15
- Issue: #3
- Related: [ADR-0001](ADR/0001-trust-trail-commitment-anchor.md), [DATA_BOUNDARIES.md](DATA_BOUNDARIES.md), [ARCHITECTURE.md](ARCHITECTURE.md)

## 1. Scope and method

This model covers the first contract, the **Trust Trail commitment anchor**
defined in ADR-0001, together with the pieces that touch it: the backend
anchorer, its signing key, and the Testnet deployment path.

The contract in the repository is currently a scaffold (`ping` only). This model
is written against the ADR design, so every "Required" mitigation below is a
requirement on the implementation issue, not a description of existing code.
Update this document in the same PR that changes contract behavior.

Out of scope: Mainnet (needs its own decision), internals of the off-chain
backend except where they cross a trust boundary, and the Stellar network and
consensus itself.

Method: STRIDE-style walk over assets, actors, and trust boundaries, plus the
five areas the issue requires (authorization, replay, malicious inputs, key
loss, resource exhaustion).

### Security objectives

| ID | Objective |
| --- | --- |
| O1 | Integrity: an anchored commitment can never be changed or overwritten. |
| O2 | Authorization: only the configured anchorer can anchor or rotate authority. |
| O3 | Confidentiality: nothing prohibited by DATA_BOUNDARIES section 1 reaches the chain. |
| O4 | Availability and cost: per-approval cost is bounded; approvals never depend on chain availability. |
| O5 | Honest semantics: chain inclusion is never treated as proof that an off-chain claim is true. |

## 2. Assets

| ID | Asset | Why it matters |
| --- | --- | --- |
| A1 | Anchored commitments (`commitment_id -> hash`) | The whole value of the system is that these never change. |
| A2 | Anchorer authority (configured account) | Controls who can write and rotate. |
| A3 | Anchorer signing key | Compromise or loss of it equals compromise or loss of A2. |
| A4 | Off-chain approval records, id-to-member mapping, audit log | The source of truth and the only place personal data lives. |
| A5 | Contract state availability (TTL, archival) | Verification reads fail if state is archived. |
| A6 | Anchorer account fee balance | Anchoring stops if it is empty. |
| A7 | Repository, CI, and deploy tooling | Path by which secrets or malicious code could reach Testnet. |

## 3. Actors

- **Backend anchorer** (trusted, fallible): holds A3, submits anchors.
- **Operators and maintainers** (trusted, fallible): deploy, rotate keys, merge.
- **External verifier** (untrusted): reads state and events through RPC.
- **Arbitrary caller** (untrusted): can invoke any public function.
- **Passive observer** (untrusted): reads all on-chain data.
- **Compromised backend or insider** (adversary): holds A3 and approval power.
- **Supply-chain attacker** (adversary): malicious dependency or contribution.

## 4. Trust boundaries

- **TB1 Verifier to chain:** verifiers trust only on-chain state plus
  their own recomputed commitment.
- **TB2 Backend to Stellar network:** only a transaction signed by the
  anchorer account may anchor.
- **TB3 Application logic to key custody:** approval logic must not
  hold raw key material beyond signing.
- **TB4 Chain to off-chain truth:** inclusion is not truth
  (DATA_BOUNDARIES section 4); the backend stays authoritative.
- **TB5 Repository and CI to Testnet:** no secrets in the repo; deploys
  are reproducible and reviewed.

## 5. Threats and mitigations

Status: **Required** (implemented and tested before Testnet), **Accepted**
(named owner, see section 7), **Decision** (open, see section 6).

### 5.1 Authorization

- **AUTH-1** Non-anchorer calls `anchor`. Impact: false commitments.
  Mitigation: `require_auth` on the stored anchorer for every write;
  test with an unauthorized caller. Status: Required.
- **AUTH-2** Non-anchorer calls authority rotation. Impact: takeover.
  Mitigation: same `require_auth` check; test rejection.
  Status: Required.
- **AUTH-3** Init front-running: attacker initializes first and becomes
  anchorer. Impact: total takeover at deploy. Mitigation: set the
  anchorer atomically at deployment or restrict one-time init to the
  deployer with `require_auth`; init cannot run twice; test both.
  Status: Required.
- **AUTH-4** Rotation to a wrong or uncontrolled address. Impact:
  permanent lockout (see KEY-1). Mitigation: two-step handoff where the
  new authority accepts with its own auth; test that an unaccepted
  proposal changes nothing. Status: Decision D2.
- **AUTH-5** Backend grants an approval improperly, then anchors it.
  Impact: valid-looking anchor of a false claim. Mitigation: out of
  contract scope by design; verifiers must not treat inclusion as
  truth (O5). Status: Accepted (R1).
- **AUTH-6** Contract starts interpreting application permissions.
  Impact: authority drift on-chain. Mitigation: no member, role, or
  permission concepts in the contract; review checklist item.
  Status: Required.

### 5.2 Replay

- **REP-1** Resubmitting a captured anchor transaction. Impact:
  duplicate write or event. Mitigation: Stellar sequence numbers and
  Soroban auth nonce plus expiration protect the transaction layer; the
  contract also enforces one entry per `commitment_id`.
  Status: Required.
- **REP-2** Re-anchoring an existing id with the same hash. Impact:
  duplicate event, confusing consumers. Mitigation: recommended
  idempotent success with no write and no event; test all three cases
  (new id, same hash, different hash). Status: Decision D1.
- **REP-3** Re-anchoring an existing id with a different hash. Impact:
  overwrite of A1. Mitigation: always rejected; test. Status: Required.
- **REP-4** Reusing an old approval hash under a new id. Impact: fake
  fresh approval. Mitigation: the preimage binds `commitment_id` and
  `approved_at`; ids are fresh CSPRNG values per approval.
  Status: Required.
- **REP-5** Same commitment presented on another deployment or network.
  Impact: Testnet evidence mistaken for production evidence.
  Mitigation: the v1 preimage has a domain string but no network or
  contract id, so verifiers must pin network passphrase and contract
  id; consider binding both in a future preimage version.
  Status: Accepted (R2).

### 5.3 Malicious or malformed inputs

- **INP-1** Hash of the wrong length. Impact: unverifiable or ambiguous
  state. Mitigation: hash parameter typed as fixed 32 bytes; test.
  Status: Required.
- **INP-2** Malformed `commitment_id` (wrong length, uppercase,
  non-hex, oversized). Impact: unbounded storage keys, PII smuggling.
  Mitigation: validate 64 lowercase hex characters if the id is a
  string, or use a fixed 32-byte type; reject everything else; test.
  Status: Required.
- **INP-3** Free text or PII passed through any parameter or event
  field. Impact: violates O3 permanently, since the ledger is
  immutable. Mitigation: no free-text parameters; events carry only
  permitted fields and fixed enums; test that events contain only
  permitted data. Status: Required.
- **INP-4** Canonicalization ambiguity (delimiter injection,
  whitespace, leading zeros, float formatting). Impact: two inputs
  with one preimage, or divergent verifiers. Mitigation: the
  reject-not-escape rules from DATA_BOUNDARIES section 3; reference
  implementation in `crates/shared-types`; failure paths tested.
  Status: Required.
- **INP-5** Backend and verifier compute different preimages. Impact:
  false tamper alarms or missed tampering. Mitigation: shared golden
  test vectors in `crates/shared-types`; any contract-side
  implementation must match byte-for-byte. Status: Required.
- **INP-6** Brute-forcing a commitment to learn its contents. Impact:
  disclosure of `approved_at` or the id. Mitigation: the 256-bit
  random id makes the preimage infeasible to guess, and no PII is in
  the preimage. Status: Required.
- **INP-7** Contract trusts `approved_at` as verified time. Impact:
  wrong time semantics. Mitigation: `approved_at` is backend-supplied
  and exists only inside the hash; use the ledger-derived value for
  "when anchored". Status: Required.

### 5.4 Key loss and compromise

- **KEY-1** Anchorer key lost. Impact: no new anchors and no rotation,
  so the contract is frozen; existing anchors stay verifiable.
  Mitigation: the off-chain audit log stays primary so approvals
  continue (ADR); document a tested recovery path of redeploying and
  re-anchoring from the audit log; Mainnet needs a key-custody ADR and
  should consider a multisig Stellar account as anchorer.
  Status: Accepted (R3).
- **KEY-2** Anchorer key stolen. Impact: attacker anchors arbitrary
  hashes for new ids and may rotate authority to themselves; existing
  anchors cannot be overwritten (REP-3). Mitigation: reconciliation
  job comparing on-chain anchors with backend records and alerting on
  unknown ids; documented key-revocation procedure; a separate admin
  role (D2) so a stolen anchoring key cannot rotate.
  Status: Decision D2.
- **KEY-3** Secret or seed phrase committed to the repo or logs.
  Impact: full compromise. Mitigation: `.gitignore` for key files,
  secret scanning in CI, scripts read keys only from the environment
  or Stellar CLI identities, never from files in the repo.
  Status: Required.
- **KEY-4** Same key reused across Testnet and Mainnet. Impact: a
  Testnet leak compromises production. Mitigation: distinct keys per
  environment, recorded in the runbook. Status: Required.

### 5.5 Resource exhaustion and availability

- **RES-1** Unbounded state growth. Impact: rising rent and cost.
  Mitigation: one entry per approval, fixed-size values, no per-member
  state (ADR cost bound). Status: Required.
- **RES-2** Whole commitment map stored in instance storage. Impact:
  every call loads all of it, so budgets blow up as it grows.
  Mitigation: each commitment under its own persistent key; instance
  storage holds only the authority and config. Status: Required.
- **RES-3** Loops over unbounded data. Impact: CPU and budget
  exhaustion. Mitigation: no iteration over stored collections; all
  operations are O(1) by key. Status: Required.
- **RES-4** Third party spams anchors. Impact: cost and noise.
  Mitigation: writes require anchorer auth (AUTH-1); a compromised key
  is bounded by fees and detected by reconciliation (KEY-2).
  Status: Required.
- **RES-5** State archival: persistent entries expire and verification
  reads fail. Impact: verifiers cannot confirm anchors. Mitigation:
  persistent (not temporary) storage; extend TTL on write; run a
  TTL-extension job; document how to restore archived entries.
  Status: Required.
- **RES-6** Anchorer account runs out of fee balance. Impact:
  anchoring stops. Mitigation: balance monitoring, simulate before
  submit, retry with backoff; approvals continue off-chain (ADR).
  Status: Required.
- **RES-7** Oversized or many events. Impact: noise and cost.
  Mitigation: exactly one fixed-size anchor event per new commitment.
  Status: Required.
- **RES-8** Public RPC unavailable. Impact: verification and anchoring
  delayed. Mitigation: anchoring is optional and asynchronous;
  document more than one RPC provider in the runbook.
  Status: Accepted (R4).

### 5.6 Other threats

- **OTH-1** Upgrade function lets a key swap contract logic. Impact:
  all invariants void. Mitigation: the v1 Testnet contract ships with
  no upgrade path unless a future ADR defines one (ARCHITECTURE
  requires an upgradeability decision). Status: Required.
- **OTH-2** Timing and address correlation by an observer. Impact:
  reveals that approvals happened and roughly when. Mitigation: public
  by nature of anchoring; no identifiers are on-chain, so linkage to a
  person needs off-chain data; do not anchor in a way that leaks
  volume per member. Status: Accepted (R5).
- **OTH-3** Data erasure requests versus an immutable ledger. Impact:
  compliance friction. Mitigation: only random ids and hashes are
  on-chain, so erasure is done by deleting the off-chain id-to-member
  mapping; confirm the approach with maintainers before Mainnet.
  Status: Accepted (R6).
- **OTH-4** Consumers treat an event as proof of an off-chain claim.
  Impact: false trust. Mitigation: document in README and runbook;
  verification always recomputes and compares, and states its limits
  (AUTH-5). Status: Required.
- **OTH-5** Malicious or vulnerable dependency. Impact: compromised
  build. Mitigation: commit `Cargo.lock`, pin the Soroban SDK, run
  `cargo audit` in CI, review dependency changes in PRs.
  Status: Required.
- **OTH-6** Deploying to the wrong network, or Mainnet by accident.
  Impact: production exposure. Mitigation: the runbook verifies the
  network passphrase before every deploy; CONTRIBUTING forbids
  Mainnet without explicit approval. Status: Required.
- **OTH-7** Cross-contract or reentrancy abuse. Impact: state
  corruption. Mitigation: the contract makes no external contract
  calls in v1; adding any requires a model update. Status: Required.

## 6. Open decisions

- **D1** Behavior when the same id and same hash is anchored again.
  Recommendation: idempotent success with no write and no event, so
  backend retries after timeouts are safe.
- **D2** The ADR lets the anchorer also rotate authority. Should
  rotation be a separate admin role? Recommendation: yes. Keep the
  admin (cold, rarely used) separate from the anchorer (hot) and make
  rotation two-step. If maintainers keep a single role, record it as an
  accepted risk with an owner.

Resolve both in the implementation PR or an ADR update before Testnet
deployment.

## 7. Accepted risks

Owners are roles; the reviewing maintainer records a GitHub handle
against each at review time.

- **R1** A compromised or dishonest backend can anchor a false
  approval and the chain cannot detect it. Owner: backend owner.
- **R2** Preimage v1 has no network or contract id binding.
  Owner: maintainers.
- **R3** Anchorer key loss freezes the contract; recovery is redeploy
  and re-anchor from the audit log. Owner: maintainers (key custodian).
- **R4** Anchoring and verification depend on public RPC availability.
  Owner: backend owner.
- **R5** Anchoring time and anchorer address are publicly visible.
  Owner: maintainers.
- **R6** Erasure is handled off-chain by removing the id mapping.
  Owner: maintainers.

## 8. Required tests for the implementation

Each Required item above maps to a deterministic test. Minimum set:

- Unauthorized anchor and unauthorized rotation are rejected
  (AUTH-1, AUTH-2).
- Initialization cannot be front-run or repeated (AUTH-3).
- Same id with a different hash is rejected; same id with the same
  hash follows D1 (REP-2, REP-3).
- Wrong-length hash and malformed ids are rejected (INP-1, INP-2).
- Anchor events contain only permitted fields (INP-3).
- Canonicalization rejection paths and golden vectors pass
  (INP-4, INP-5).
- Writes use per-id persistent keys and TTL is extended
  (RES-2, RES-5).

## 9. Review gate

No Testnet deployment may happen until:

- [ ] A maintainer other than the author has reviewed this document
  and recorded sign-off on the PR.
- [ ] Decisions D1 and D2 are resolved, or recorded as accepted risks
  with owners.
- [ ] Every Required mitigation has an automated test or a documented
  reason it does not apply.
- [ ] Each accepted risk in section 7 has a named person as owner.
- [ ] The DATA_BOUNDARIES review checklist (section 5 there) passes.

Re-review is required when the contract interface, storage layout,
authority model, or preimage version changes, and before any Mainnet
decision.
