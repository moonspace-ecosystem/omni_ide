/// Trait for converting between Git object ID representations.
/// GitButler uses `gix::ObjectId`, Zed uses a wrapper around `git2::Oid`.
/// Both ultimately represent a 20-byte SHA-1 hash, so we bridge via raw bytes.
pub trait ObjectIdConvert: Sized {
    fn to_bytes(&self) -> [u8; 20];
    fn from_bytes(bytes: &[u8; 20]) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawObjectId([u8; 20]);

impl RawObjectId {
    pub fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl ObjectIdConvert for RawObjectId {
    fn to_bytes(&self) -> [u8; 20] {
        self.0
    }

    fn from_bytes(bytes: &[u8; 20]) -> Self {
        Self(*bytes)
    }
}

impl ObjectIdConvert for [u8; 20] {
    fn to_bytes(&self) -> [u8; 20] {
        *self
    }

    fn from_bytes(bytes: &[u8; 20]) -> Self {
        *bytes
    }
}

/// Convert between two ObjectId types that both implement ObjectIdConvert.
pub fn convert_object_id<From: ObjectIdConvert, To: ObjectIdConvert>(source: &From) -> To {
    To::from_bytes(&source.to_bytes())
}
