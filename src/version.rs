use std::{fmt::Display, ops::{Add, Neg}};

/// A version number
/// negative means it was locked
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Version {
    version: isize,
}

impl Version {
    pub fn is_locked(self) -> bool {
        self.version < 0
    }

    pub fn check(self, ref_version: Version) -> bool {
        self.version >= 0 && self <= ref_version
    }
}

impl From<isize> for Version {
    fn from(value: isize) -> Self {
        Self { version: value }
    }
}

impl Into<isize> for Version {
    fn into(self) -> isize {
        self.version
    }
}

impl Add for Version {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self.version + rhs.version).into()
    }
}

impl Add<isize> for Version {
    type Output = Self;

    fn add(self, rhs: isize) -> Self::Output {
        (self.version + rhs).into()
    }
}

impl Neg for Version {
    type Output = Self;

    fn neg(self) -> Self::Output {
        (-self.version).into()
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.version.fmt(f)
    }
}