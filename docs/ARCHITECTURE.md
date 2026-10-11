# Stellar / Soroban Architecture

## Proposed layers
- Contracts: small deterministic Soroban contracts with explicit authorization and state invariants.
- Shared types: serialization and domain-neutral types where justified.
- Client/integration: typed Stellar SDK calls, simulation/submission, confirmation polling, and error mapping.
- Backend adapter: associate transaction identifiers with off-chain records without moving application authority on-chain.
- Scripts/runbooks: reproducible build, testnet deployment, verification, and recovery.

## On-chain/off-chain boundary
Keep personal identity, accommodation documents, provider payloads, and sensitive metadata off-chain. Backend remains authoritative for application permissions and off-chain payment reconciliation.

## Progression
Local deterministic tests → Stellar Testnet → independent security review → explicit production-readiness decision. Mainnet is not automatic.

## Required decisions
Concrete user problem; state/transition invariants; authorized actors and revocation; data minimization and commitment format; upgradeability; resource/cost bounds; event semantics; and operational key custody.

Recorded decisions live in [ADR/](ADR/); the first use case is [ADR-0001](ADR/0001-trust-trail-commitment-anchor.md). Data rules are normative in [DATA_BOUNDARIES.md](DATA_BOUNDARIES.md).
