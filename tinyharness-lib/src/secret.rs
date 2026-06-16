use serde::{Deserialize, Serialize, Serializer};

/// Minimal no-new-dependency replacement for `secrecy::SecretString`.
/// Exposes the secret only on explicit `.expose_secret()` calls. The secret
/// cannot be implicitly leaked by the `Debug` implementation, e.g. in logs.
///
/// Retains serialization/deserialization via `serde`.
///
/// Deviations from `secrecy`:
/// 1. Does not zero on `Drop`: the heap bytes remain after deallocation, and,
///    e.g., a core dump could contain leaked secrets.
/// 2. Implements `Clone`: `secrecy` uses its own `SecretBox` smart pointer for
///    space efficiency. Since we can reasonably expect only a handful of
///    secrets in memory for TinyHarness, that optimization is not worth the
///    added complexity.
#[derive(Clone, Default)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Expose the underlying secret. Use this as the point the actual value is
    /// required (e.g., in constructing an HTTP request). Do not log the exposed
    /// value.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn masked(&self) -> String {
        if self.0.is_ascii() && self.len() > 8 {
            format!("{}...{}", &self.0[..4], &self.0[self.len() - 4..])
        } else {
            "****".to_string()
        }
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(SecretString)
    }
}
