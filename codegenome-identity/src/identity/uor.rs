use serde::{Deserialize, Serialize};

/// The atomic identity. Same bytes -> same address. Always.
/// This is the puzzle piece's name — not its shape.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UorAddress(pub [u8; 32]);

impl UorAddress {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// First 8 hex characters for display.
    pub fn short_hex(&self) -> String {
        hex::encode(&self.0[..4])
    }
}

impl std::fmt::Debug for UorAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UOR({}...)", hex::encode(&self.0[..4]))
    }
}

impl std::fmt::Display for UorAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0xf) as usize] as char);
        }
        s
    }
}
