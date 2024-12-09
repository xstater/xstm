/// A version number that always be positive
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Version {
    version: isize,
}

impl Version {
    pub fn new(version: isize) -> Option<Version> {
        if version < 0 {
            None
        } else {
            Some(Version { version })
        }
    }

    pub fn as_isize(&self) -> isize {
        self.version
    }
}
