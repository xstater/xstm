use crate::version::Version;
use std::sync::atomic::{AtomicIsize, Ordering};

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
        version.into()
    }

    pub fn tick(&self) -> Version {
        // add and fetch
        let old = self.version.fetch_add(1, Ordering::SeqCst);
        old.wrapping_add(1).into()
    }
}
