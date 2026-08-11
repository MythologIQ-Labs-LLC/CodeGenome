//! Binary codec for derived state — parse caches, overlay stores,
//! embedding stores. Wraps postcard behind a stable signature so a
//! future format change is a one-file edit.
//!
//! postcard replaces bincode 1.x, which is unmaintained
//! (RUSTSEC-2025-0141); bincode's own final release recommends it.
//!
//! Everything encoded here is regenerable local state: decode failure
//! is always treated as a cache miss / absent store by callers, never
//! as data loss. Existing stores written with the old bincode format
//! simply fail to decode and are rebuilt on the next index. Identity
//! hashes (`identity::address_of`) operate on raw source bytes and
//! never on this encoding.

pub fn to_vec<T: serde::Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, String> {
    postcard::to_allocvec(value).map_err(|e| e.to_string())
}

pub fn from_slice<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    postcard::from_bytes(bytes).map_err(|e| e.to_string())
}
