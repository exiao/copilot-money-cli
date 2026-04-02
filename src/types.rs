use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;

use clap::ValueEnum;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// NOTE: Copilot IDs are opaque runtime strings. We use a marker type per ID kind so they
// are not accidentally interchangeable (similar to Swift's Tagged<>).
pub struct OwnedId<T> {
    raw: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T> OwnedId<T> {
    pub fn new(raw: String) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl<T> Clone for OwnedId<T> {
    fn clone(&self) -> Self {
        Self::new(self.raw.clone())
    }
}

impl<T> fmt::Debug for OwnedId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Id").field(&self.raw).finish()
    }
}

impl<T> fmt::Display for OwnedId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl<T> PartialEq for OwnedId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<T> Eq for OwnedId<T> {}

impl<T> Hash for OwnedId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T> FromStr for OwnedId<T> {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.to_string()))
    }
}

impl<T> From<String> for OwnedId<T> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl<T> From<&str> for OwnedId<T> {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl<T> Serialize for OwnedId<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de, T> Deserialize<'de> for OwnedId<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

pub enum TransactionMarker {}
pub enum CategoryMarker {}
pub enum TagMarker {}
pub enum RecurringMarker {}
pub enum AccountMarker {}
pub enum ItemMarker {}

pub type TransactionId = OwnedId<TransactionMarker>;
pub type CategoryId = OwnedId<CategoryMarker>;
pub type TagId = OwnedId<TagMarker>;
pub type RecurringId = OwnedId<RecurringMarker>;
pub type AccountId = OwnedId<AccountMarker>;
pub type ItemId = OwnedId<ItemMarker>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Regular,
    InternalTransfer,
    #[serde(other)]
    Other,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TransactionType::Regular => "REGULAR",
            TransactionType::InternalTransfer => "INTERNAL_TRANSFER",
            TransactionType::Other => "OTHER",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecurringFrequency {
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Quarterly,
    Annually,
    #[serde(other)]
    Other,
}

impl fmt::Display for RecurringFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RecurringFrequency::Daily => "DAILY",
            RecurringFrequency::Weekly => "WEEKLY",
            RecurringFrequency::Biweekly => "BIWEEKLY",
            RecurringFrequency::Monthly => "MONTHLY",
            RecurringFrequency::Quarterly => "QUARTERLY",
            RecurringFrequency::Annually => "ANNUALLY",
            RecurringFrequency::Other => "OTHER",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // -- OwnedId trait tests --------------------------------------------------

    #[test]
    fn owned_id_clone() {
        let id: TransactionId = OwnedId::new("abc".to_string());
        let cloned = id.clone();
        assert_eq!(id, cloned);
        assert_eq!(cloned.as_str(), "abc");
    }

    #[test]
    fn owned_id_debug() {
        let id: TransactionId = OwnedId::new("xyz".to_string());
        let dbg = format!("{:?}", id);
        assert!(dbg.contains("Id"));
        assert!(dbg.contains("xyz"));
    }

    #[test]
    fn owned_id_display() {
        let id: CategoryId = OwnedId::new("cat_42".to_string());
        assert_eq!(format!("{id}"), "cat_42");
    }

    #[test]
    fn owned_id_eq_and_ne() {
        let a: TagId = OwnedId::new("t1".to_string());
        let b: TagId = OwnedId::new("t1".to_string());
        let c: TagId = OwnedId::new("t2".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn owned_id_hash() {
        let a: TransactionId = OwnedId::new("h1".to_string());
        let b: TransactionId = OwnedId::new("h1".to_string());
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn owned_id_from_str() {
        let id: RecurringId = "rec_1".parse().unwrap();
        assert_eq!(id.as_str(), "rec_1");
    }

    #[test]
    fn owned_id_from_string() {
        let id: AccountId = AccountId::from("acct_1".to_string());
        assert_eq!(id.as_str(), "acct_1");
    }

    #[test]
    fn owned_id_from_str_ref() {
        let id: ItemId = ItemId::from("item_1");
        assert_eq!(id.as_str(), "item_1");
    }

    #[test]
    fn owned_id_serde_roundtrip() {
        let id: TransactionId = OwnedId::new("txn_abc".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"txn_abc\"");
        let back: TransactionId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    // -- TransactionType tests ------------------------------------------------

    #[test]
    fn transaction_type_display() {
        assert_eq!(TransactionType::Regular.to_string(), "REGULAR");
        assert_eq!(
            TransactionType::InternalTransfer.to_string(),
            "INTERNAL_TRANSFER"
        );
        assert_eq!(TransactionType::Other.to_string(), "OTHER");
    }

    #[test]
    fn transaction_type_serde_roundtrip() {
        for variant in [TransactionType::Regular, TransactionType::InternalTransfer] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: TransactionType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn transaction_type_other_catch_all() {
        let back: TransactionType = serde_json::from_str("\"SOMETHING_NEW\"").unwrap();
        assert_eq!(back, TransactionType::Other);
    }

    // -- RecurringFrequency tests ---------------------------------------------

    #[test]
    fn recurring_frequency_display() {
        assert_eq!(RecurringFrequency::Daily.to_string(), "DAILY");
        assert_eq!(RecurringFrequency::Weekly.to_string(), "WEEKLY");
        assert_eq!(RecurringFrequency::Biweekly.to_string(), "BIWEEKLY");
        assert_eq!(RecurringFrequency::Monthly.to_string(), "MONTHLY");
        assert_eq!(RecurringFrequency::Quarterly.to_string(), "QUARTERLY");
        assert_eq!(RecurringFrequency::Annually.to_string(), "ANNUALLY");
        assert_eq!(RecurringFrequency::Other.to_string(), "OTHER");
    }

    #[test]
    fn recurring_frequency_serde_roundtrip() {
        for variant in [
            RecurringFrequency::Daily,
            RecurringFrequency::Weekly,
            RecurringFrequency::Biweekly,
            RecurringFrequency::Monthly,
            RecurringFrequency::Quarterly,
            RecurringFrequency::Annually,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: RecurringFrequency = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn recurring_frequency_other_catch_all() {
        let back: RecurringFrequency = serde_json::from_str("\"SEMI_ANNUAL\"").unwrap();
        assert_eq!(back, RecurringFrequency::Other);
    }
}
