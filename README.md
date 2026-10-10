# Hut4Devs Stellar / Soroban

Dedicated repository for evaluating and implementing blockchain capabilities that genuinely improve Hut4Devs. The current application does not yet have an implemented Soroban contract. Start with learning, threat modeling, testnet experiments, and narrowly scoped contracts—not claims of deployed blockchain functionality.

## Principles
- Start on Stellar Testnet; no Mainnet deployment merely for demonstration.
- Keep member identity, accommodation details, payment-provider data, and personal information off public-chain state.
- Prefer minimal verifiable commitments over publishing source documents.
- Smart contracts do not replace backend authorization or governance.
- Define threat model, invariants, deployment strategy, and key-management plan before production use.

## Target structure
```text
contracts/
  trust-trail/
    Cargo.toml
    src/lib.rs
    tests/
crates/
  shared-types/
  client/
scripts/
  build.sh
  test.sh
  deploy-testnet.sh
docs/
  ARCHITECTURE.md
  THREAT_MODEL.md
  DATA_BOUNDARIES.md
  TESTNET_RUNBOOK.md
  ADR/
tests/
  fixtures/
  scenarios/
```

## Scope candidates
- Stellar accounts, assets, transactions, signing, SDKs, and RPC learning.
- Soroban workspace and deterministic contract tests.
- A narrowly scoped contract for verifying approved, non-sensitive commitments only after product and threat-model review.
- Backend integration that records transaction identifiers without treating chain inclusion as proof of off-chain truth.

Mainnet and real-value transfers are out of scope until explicitly approved after testing, security review, and operational readiness.
