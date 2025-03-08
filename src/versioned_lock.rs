use std::sync::atomic::{AtomicIsize, Ordering};

use crate::version::Version;

#[derive(Debug)]
pub struct VersionedLock {
    local_version: AtomicIsize,
}

impl VersionedLock {
    pub fn new() -> VersionedLock {
        VersionedLock {
            local_version: AtomicIsize::new(1)
        }
    }

    /// Get the version
    pub fn version(&self) -> Version {
        self.local_version.load(Ordering::SeqCst).into()
    }

    /// Try get the write lock
    pub fn try_lock(&self) -> Option<Guard<'_>> {
        let current_version = self.version();
        let current_version_isize = current_version.into();

        if current_version.is_locked() {
            // already locked by others
            return None;
        }

        // do a CAS action to make the version be negative
        let result = self.local_version.compare_exchange(
            current_version_isize,
            -current_version_isize,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );

        match result {
            // Lock successfully
            Ok(_) => Some(Guard {
                version: &self.local_version,
                new_local_version: current_version,
            }),
            Err(_) => None,
        }
    }
}

pub struct Guard<'lock> {
    version: &'lock AtomicIsize,
    new_local_version: Version,
}

impl<'lock> Guard<'lock> {
    /// Update the local version
    /// The new version will be written when dropping
    pub fn set_version(&mut self, write_version: Version) {
        self.new_local_version = write_version;
    }
}

impl<'lock> Drop for Guard<'lock> {
    fn drop(&mut self) {
        // Write the new_local_version to version
        self.version
            .store(self.new_local_version.into(), Ordering::SeqCst);
    }
}
