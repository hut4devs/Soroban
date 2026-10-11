# DATA BOUNDARIES

Normative rules for what may exist on-chain versus off-chain in the Hut4Devs
Stellar integration. Companion to [ADR-0001](ADR/0001-trust-trail-commitment-anchor.md)
and [ARCHITECTURE.md](ARCHITECTURE.md). For conflicts: this document wins for
data questions; the ADR wins for scope questions.

## 1. Prohibited on-chain

The following must never be written to contract storage, contract events, or
transaction memos — not even encrypted, truncated, or "anonymized" — except
inside a SHA-256 commitment computed from the canonical preimage defined in
section 3:

- **Personally identifiable data:** names, email addresses, phone numbers,
  physical or postal addresses, government identifiers, dates of birth, profile
  photos, or any free-text field written by a member.
- **Member account details:** credentials, session tokens, password material,
  internal user ids, or the mapping from a commitment id to a member.
- **Accommodation records:** listings, availability, addresses, booking dates,
  photos, documents, host/guest relations, or prices.
- **Payment-provider payloads:** card or bank data, provider tokens, provider
  transaction ids, amounts, currencies, or any field that reveals a payer or
  payee.
- **Secrets:** API keys, seed phrases, mnemonics, signing keys of any kind.
- **Raw document content** of any kind, in any encoding.

If a field cannot be classified as permitted by section 2, it is prohibited.

## 2. Permitted on-chain

Only the following data may appear in contract state or events:

| Data | Notes |
| --- | --- |
| Commitment hash | Raw 32-byte SHA-256 of the canonical preimage (section 3). |
| Commitment id | Backend-generated opaque 64-character lowercase hex string from a CSPRNG. Must not be derivable from member, booking, or payment data (no sequential ids, no hashes of PII). |
| Anchorer authority | The Stellar account authorized to anchor; rotation is an explicit authorized operation. |
| Ledger-derived values | Ledger sequence number / close time as reported by the network. |
| Fixed enums | e.g. record kind `approval`, status `anchored`. No free-text. |

## 3. Commitment canonicalization (normative)

All implementations of the commitment preimage — backend, tests, and any future
contract-side verification — MUST produce byte-identical preimages for the same
logical inputs. The canonical form for `v1`:

**Fields, in fixed order (no optional fields in v1):**

1. `domain` — the literal ASCII string `hut4devs.trail.v1`.
2. `record_kind` — one of the fixed lowercase literals; v1 allows only `approval`.
3. `commitment_id` — 64-character lowercase hexadecimal string, CSPRNG-generated
   by the backend.
4. `approved_at` — Unix time in **seconds**, unsigned 64-bit, decimal, no sign,
   no leading zeros (the value `0` is the single character `0`).
5. `authority` — the Stellar account id (56-character base32, uppercase,
   starting with `G`) of the backend anchorer.

**Encoding:**

- Fields are joined in the order above with the single character `|` (ASCII
  0x7C), with no padding, no trailing separator, and no trailing newline.
- The result is interpreted as UTF-8 bytes; the preimage is exactly those bytes.
- No field may be empty and no field may contain `|`. Callers MUST reject such
  inputs rather than escaping or sanitizing them.
- No whitespace is added or trimmed anywhere; comparison and hashing are
  byte-exact.
- Floats, locale-dependent formatting, and non-ASCII in numeric fields are
  prohibited.

**Commitment:**

- `commitment = SHA-256(canonical_preimage_bytes)`, stored on-chain as raw
  32 bytes and off-chain as 64-character lowercase hex.
- The preimage itself is never sent on-chain and never stored in contract state
  or events.

**Rejection rules (failure paths MUST be tested):**

- Empty required field, or any field containing `|`.
- `commitment_id` with wrong length, uppercase, or non-hex characters.
- `approved_at` that is empty, non-decimal, signed, has leading zeros, or
  overflows 64 bits.
- `authority` with wrong length, wrong prefix, or lowercase characters.

An executable reference implementation of this encoding lives in
[`crates/shared-types`](../crates/shared-types/src/lib.rs) with deterministic
tests; when a contract-side implementation exists it must match this crate's
output byte-for-byte.

## 4. Backend authority

- The backend remains authoritative for off-chain truth: who may approve what,
  whether an approval is valid, and the contents of any record.
- The contract verifies only that the configured anchorer authorized an anchor.
  It never grants, revokes, or interprets application permissions.
- Chain inclusion is **not** proof that an off-chain claim is true. A backend
  record and its anchor can both exist while the underlying approval was
  improperly granted; resolving that stays an off-chain, backend-controlled
  process.
- Personalized off-chain responses (approval status lookups, etc.) are served
  by the backend under its existing authorization, never read from chain state
  alone.

## 5. Review checklist

Before any contract change merges, verify:

- [ ] No prohibited data (section 1) appears in storage, events, or memos.
- [ ] Commitment ids are opaque CSPRNG values, not derived from PII.
- [ ] Preimages match the canonical format (section 3) byte-for-byte.
- [ ] Failure paths from section 3 are covered by deterministic tests.
- [ ] No secrets, seed phrases, or real personal/payment data are committed.
