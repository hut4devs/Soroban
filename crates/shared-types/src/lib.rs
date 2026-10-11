//! Executable reference implementation of the Trust Trail commitment
//! canonicalization format (issue #4, `docs/DATA_BOUNDARIES.md` section 3).
//!
//! This crate is the byte-for-byte authority for how a commitment preimage is
//! built from approval inputs. A future contract-side implementation must
//! produce identical bytes for identical inputs. It intentionally contains no
//! hashing: the SHA-256 step happens via the Soroban env (or a backend crypto
//! library), and only the preimage encoding needs cross-implementation
//! agreement here.

/// Domain separator; the literal first field of every v1 preimage.
pub const DOMAIN: &str = "hut4devs.trail.v1";

/// The only record kind allowed in v1.
pub const RECORD_KIND_APPROVAL: &str = "approval";

/// Character joining canonical fields. Must not appear inside any field.
pub const FIELD_SEPARATOR: char = '|';

/// Required length of a commitment id: 64 lowercase hex characters.
pub const COMMITMENT_ID_LEN: usize = 64;

/// Required length of a Stellar account id: 56 base32 characters.
pub const AUTHORITY_LEN: usize = 56;

/// Reasons an input set is rejected as non-canonical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalizationError {
    /// A required field was empty.
    EmptyField,
    /// A field contained the `|` separator.
    FieldContainsSeparator,
    /// Commitment id was not 64 lowercase hex characters.
    InvalidCommitmentId,
    /// `approved_at` was not a canonical unsigned 64-bit decimal.
    InvalidApprovedAt,
    /// Authority was not a 56-character uppercase base32 `G…` account id.
    InvalidAuthority,
}

/// Parses a `approved_at` string per the canonical rules: non-empty, decimal
/// digits only, no sign, no leading zeros, fits in `u64`.
pub fn parse_approved_at(raw: &str) -> Result<u64, CanonicalizationError> {
    if raw.is_empty() {
        return Err(CanonicalizationError::InvalidApprovedAt);
    }
    if !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(CanonicalizationError::InvalidApprovedAt);
    }
    if raw.len() > 1 && raw.starts_with('0') {
        return Err(CanonicalizationError::InvalidApprovedAt);
    }
    raw.parse::<u64>()
        .map_err(|_| CanonicalizationError::InvalidApprovedAt)
}

/// Validates a commitment id: exactly [`COMMITMENT_ID_LEN`] lowercase hex
/// characters.
pub fn validate_commitment_id(id: &str) -> Result<(), CanonicalizationError> {
    let ok =
        id.len() == COMMITMENT_ID_LEN && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
    if ok {
        Ok(())
    } else {
        Err(CanonicalizationError::InvalidCommitmentId)
    }
}

/// Validates a backend anchorer authority: exactly [`AUTHORITY_LEN`]
/// uppercase base32 characters starting with `G`.
pub fn validate_authority(authority: &str) -> Result<(), CanonicalizationError> {
    let mut chars = authority.chars();
    let first_ok = chars.next() == Some('G');
    let rest_ok = chars.clone().count() == AUTHORITY_LEN - 1
        && chars.all(|c| matches!(c, 'A'..='Z' | '2'..='7'));
    if first_ok && rest_ok {
        Ok(())
    } else {
        Err(CanonicalizationError::InvalidAuthority)
    }
}

/// Builds the canonical v1 preimage string for an approval commitment:
/// `hut4devs.trail.v1|approval|<commitment_id>|<approved_at>|<authority>`.
///
/// The returned string's UTF-8 bytes are the exact preimage; hash them with
/// SHA-256 to obtain the commitment. Rejects non-canonical inputs per
/// `docs/DATA_BOUNDARIES.md` section 3.
pub fn canonical_preimage(
    commitment_id: &str,
    approved_at: u64,
    authority: &str,
) -> Result<String, CanonicalizationError> {
    if commitment_id.is_empty() {
        return Err(CanonicalizationError::EmptyField);
    }
    if authority.is_empty() {
        return Err(CanonicalizationError::EmptyField);
    }
    validate_commitment_id(commitment_id)?;
    validate_authority(authority)?;
    // `approved_at` is a `u64`, so its decimal form is canonical by
    // construction (no sign, no leading zeros).
    Ok(format!(
        "{DOMAIN}|{RECORD_KIND_APPROVAL}|{commitment_id}|{approved_at}|{authority}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use CanonicalizationError::*;

    // 64 lowercase hex characters.
    const ID: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn authority_of_len(len: usize, prefix: char) -> String {
        let mut s = String::with_capacity(len);
        s.push(prefix);
        // `len - 1` more chars from the base32 alphabet.
        let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".chars().collect();
        for i in 0..len - 1 {
            s.push(alphabet[i % alphabet.len()]);
        }
        s
    }

    /// A valid 56-character uppercase base32 `G…` account id.
    fn valid_authority() -> String {
        authority_of_len(AUTHORITY_LEN, 'G')
    }

    #[test]
    fn preimage_matches_documented_format_exactly() {
        let authority = valid_authority();
        let preimage = canonical_preimage(ID, 1_728_000_000, &authority).unwrap();
        assert_eq!(
            preimage,
            format!("hut4devs.trail.v1|approval|{ID}|1728000000|{authority}")
        );
        // No trailing separator or whitespace; separator count is fixed.
        assert!(!preimage.ends_with('|'));
        assert_eq!(preimage.matches('|').count(), 4);
    }

    #[test]
    fn preimage_is_deterministic() {
        let authority = valid_authority();
        let a = canonical_preimage(ID, 42, &authority).unwrap();
        let b = canonical_preimage(ID, 42, &authority).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_inputs_produce_distinct_preimages() {
        let authority = valid_authority();
        let base = canonical_preimage(ID, 42, &authority).unwrap();
        let other_id = format!("f{}", &ID[1..]);
        assert_ne!(base, canonical_preimage(&other_id, 42, &authority).unwrap());
        assert_ne!(base, canonical_preimage(ID, 43, &authority).unwrap());
        // Same length and alphabet, different final character.
        let mut other_authority = authority.clone();
        other_authority.pop();
        other_authority.push('Z');
        assert_ne!(base, canonical_preimage(ID, 42, &other_authority).unwrap());
    }

    #[test]
    fn zero_timestamp_is_canonical() {
        assert_eq!(parse_approved_at("0").unwrap(), 0);
        assert!(canonical_preimage(ID, 0, &valid_authority()).is_ok());
    }

    #[test]
    fn u64_max_timestamp_round_trips() {
        assert_eq!(parse_approved_at("18446744073709551615").unwrap(), u64::MAX);
        let preimage = canonical_preimage(ID, u64::MAX, &valid_authority()).unwrap();
        assert!(preimage.contains("|18446744073709551615|"));
    }

    #[test]
    fn approved_at_rejects_non_canonical_strings() {
        for bad in [
            "",
            "007",
            "00",
            "-1",
            "+1",
            "12a",
            " 1",
            "1 ",
            "1.0",
            "18446744073709551616", // u64 overflow
        ] {
            assert_eq!(
                parse_approved_at(bad),
                Err(InvalidApprovedAt),
                "input {bad:?}"
            );
        }
    }

    #[test]
    fn commitment_id_rejects_bad_values() {
        let cases: Vec<String> = vec![
            String::new(),              // empty
            "|".to_string() + &ID[1..], // separator
            ID[..63].to_string(),       // too short
            format!("{ID}a"),           // too long
            ID.to_uppercase(),          // uppercase
            format!("g{}", &ID[1..]),   // non-hex letter
            ID.replace('a', "z"),       // non-hex letter
        ];
        let authority = valid_authority();
        for bad in cases {
            assert_eq!(
                validate_commitment_id(&bad),
                Err(InvalidCommitmentId),
                "input {bad:?}"
            );
            assert!(canonical_preimage(&bad, 42, &authority).is_err());
        }
        // The empty id is additionally reported as EmptyField by the
        // preimage builder.
        assert_eq!(canonical_preimage("", 42, &authority), Err(EmptyField));
        assert_eq!(validate_commitment_id(ID), Ok(()));
    }

    #[test]
    fn authority_rejects_bad_values() {
        let cases: Vec<(String, CanonicalizationError)> = vec![
            // Wrong length (55).
            (authority_of_len(AUTHORITY_LEN - 1, 'G'), InvalidAuthority),
            // Wrong length (57).
            (authority_of_len(AUTHORITY_LEN + 1, 'G'), InvalidAuthority),
            // Wrong prefix.
            (authority_of_len(AUTHORITY_LEN, 'X'), InvalidAuthority),
            // Lowercase.
            (
                authority_of_len(AUTHORITY_LEN, 'G').to_lowercase(),
                InvalidAuthority,
            ),
            // Contains '1', not in the base32 alphabet.
            (
                format!("G{}1", &authority_of_len(AUTHORITY_LEN - 2, 'A')[1..]),
                InvalidAuthority,
            ),
            // Contains the separator.
            (
                format!("G|{}", &authority_of_len(AUTHORITY_LEN - 2, 'A')[1..]),
                InvalidAuthority,
            ),
        ];
        for (bad, expected) in cases {
            assert_eq!(validate_authority(&bad), Err(expected), "input {bad:?}");
            assert!(canonical_preimage(ID, 42, &bad).is_err());
        }
        assert_eq!(validate_authority(&valid_authority()), Ok(()));
    }
}
