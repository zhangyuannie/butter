use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::zvariant::{self, OwnedValue, Type, Value};

#[derive(
    Serialize, Deserialize, Type, Value, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone,
)]
#[zvariant(signature = "ay")]
pub struct ZPathBuf(Vec<u8>);

impl ZPathBuf {
    #[inline]
    pub fn as_path(&self) -> &Path {
        OsStr::from_bytes(&self.0).as_ref()
    }
}

impl From<PathBuf> for ZPathBuf {
    #[inline]
    fn from(a: PathBuf) -> Self {
        Self(a.into_os_string().into_vec())
    }
}

impl From<ZPathBuf> for PathBuf {
    #[inline]
    fn from(a: ZPathBuf) -> Self {
        OsString::from_vec(a.0).into()
    }
}

#[derive(
    Serialize, Deserialize, Type, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy,
)]
#[zvariant(signature = "ay")]
pub struct ZUuid(Uuid);

impl ZUuid {
    #[inline]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<Uuid> for ZUuid {
    #[inline]
    fn from(a: Uuid) -> Self {
        Self(a)
    }
}

impl From<ZUuid> for Uuid {
    #[inline]
    fn from(a: ZUuid) -> Self {
        a.0
    }
}

impl From<ZUuid> for Value<'_> {
    fn from(a: ZUuid) -> Self {
        a.0.as_bytes().as_slice().to_owned().into()
    }
}

impl TryFrom<Value<'_>> for ZUuid {
    type Error = zvariant::Error;

    fn try_from(a: Value<'_>) -> Result<Self, Self::Error> {
        let a = Vec::<u8>::try_from(a)?;
        match Uuid::from_slice(&a) {
            Ok(uuid) => Ok(Self(uuid)),
            Err(_) => Err(Self::Error::IncorrectType),
        }
    }
}

impl TryFrom<OwnedValue> for ZUuid {
    type Error = zvariant::Error;

    fn try_from(a: OwnedValue) -> Result<Self, Self::Error> {
        let a = Vec::<u8>::try_from(a)?;
        match Uuid::from_slice(&a) {
            Ok(uuid) => Ok(Self(uuid)),
            Err(_) => Err(Self::Error::IncorrectType),
        }
    }
}
