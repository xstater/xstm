use std::sync::atomic::{AtomicIsize, Ordering};
use crate::version::Version;

pub struct VersionClock {
    version: AtomicIsize,
}

impl VersionClock {
    pub fn new() -> VersionClock {
        VersionClock {
            version: AtomicIsize::new(1),
        }
    }

    pub fn sample(&self) -> Version {
        let version = self.version.load(Ordering::SeqCst);
        Version::new(version).unwrap()
    }

    pub fn tick(&self) -> Version {
        let old = self.version.fetch_add(1, Ordering::SeqCst);
        Version::new(old + 1).unwrap()
    }
}