//! BLAKE3 digests are pinned to the official test vectors.
//!
//! `.pcraft` containers are content-addressed by BLAKE3 (`tiles/<hash>.zst`,
//! `blobs/<hash>.zst`), so the digest must stay byte-identical no matter which
//! backend the crate builds: the Rust one (`pure`) or, if someone ever drops
//! that feature, the C/asm one.
//!
//! Vectors: `test_vectors/test_vectors.json` from BLAKE3-team/BLAKE3, the file
//! the reference implementation itself tests against. Its cases hash an input
//! filled with the repeating 251-byte sequence `0, 1, ..., 250, 0, 1, ...`, and
//! the keyed cases use the 32-byte ASCII string below. Values here are the
//! first 32 bytes of the published extended outputs.

use blake3::Hasher;

const OFFICIAL_KEY: &[u8; 32] = b"whats the Elvish word for friend";

/// `input_len: 0`, `hash`.
const EMPTY_UNKEYED: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

/// `input_len: 0`, `keyed_hash`.
const EMPTY_KEYED: &str = "92b2b75604ed3c761f9d6f62392c8a9227ad0ea3f09573e783f1498a4ed60d26";

/// `input_len: 1024`, `hash`.
const PATTERN_1024: &str = "42214739f095a406f3fc83deb889744ac00df831c10daa55189b5d121c855af7";

/// The `input_len: 1024` case input: the official repeating 251-byte pattern.
const FIXTURE: &[u8] = include_bytes!("fixtures/blake3-vector-1024.bin");

fn pattern(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

#[test]
fn empty_input_matches_official_hashes() {
    assert_eq!(blake3::hash(b"").to_hex().as_str(), EMPTY_UNKEYED);
    let keyed = Hasher::new_keyed(OFFICIAL_KEY).finalize();
    assert_eq!(keyed.to_hex().as_str(), EMPTY_KEYED);
}

#[test]
fn fixture_matches_official_1024_byte_vector() {
    assert_eq!(FIXTURE.len(), 1024);
    assert_eq!(FIXTURE, pattern(FIXTURE.len()).as_slice(), "fixture is the official input pattern");
    assert_eq!(blake3::hash(FIXTURE).to_hex().as_str(), PATTERN_1024);
}

#[test]
fn streaming_matches_one_shot() {
    let mut hasher = Hasher::new();
    hasher.update(&FIXTURE[..1]);
    hasher.update(&FIXTURE[1..251]);
    hasher.update(&FIXTURE[251..]);
    assert_eq!(hasher.finalize().to_hex().as_str(), PATTERN_1024);
}
