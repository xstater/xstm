use std::cell::Cell;
use std::sync::atomic::{AtomicIsize, Ordering};
use crate::version::Version;
use crate::Transaction;

pub struct TVar<T> {
    value: Cell<T>,
    local_version: AtomicIsize,
}

// We can only Read/Write TVar in transaction
// TVar can be Sync safely
unsafe impl<T: Copy> Sync for TVar<T> {}

// Public method
impl<T: Copy> TVar<T> {
    pub fn new(value: T) -> Self {
        TVar {
            value: Cell::new(value),
            local_version: AtomicIsize::new(1),
        }
    }

    pub fn read(&self) -> impl Transaction<Output = T> + '_ {
        ReadTransaction { var: self }
    }

    pub fn write(&self, value: T) -> impl Transaction<Output = ()> + '_ {
        WriteTransaction { var: self, value }
    }
}

struct ReadTransaction<'var, T> {
    var: &'var TVar<T>,
}

impl<'trans_var, T: Copy> Transaction for ReadTransaction<'trans_var, T> {
    type Output = T;

    fn atomically<'this: 'var, 'context, 'var>(
        &'this self,
        context: &'context mut crate::Context<'var>,
    ) -> Result<Self::Output, crate::StmError> {
        context.read(&self.var)
    }
}

struct WriteTransaction<'var, T> {
    var: &'var TVar<T>,
    value: T,
}

impl<'trans_var, T: Copy> Transaction for WriteTransaction<'trans_var, T> {
    type Output = ();

    fn atomically<'this: 'var, 'context, 'var>(
        &'this self,
        context: &'context mut crate::Context<'var>,
    ) -> Result<Self::Output, crate::StmError> {
        context.write(&self.var, self.value)
    }
}

// internal methods
impl<T: Copy> TVar<T> {
    pub(crate) fn version(&self) -> isize {
        self.local_version.load(Ordering::SeqCst)
    }

    /// Check the `version_lock` is not locked
    /// and local version <= ref_version
    pub(crate) fn check(&self, ref_version: Version) -> bool {
        let local_version = self.local_version.load(Ordering::SeqCst);

        let tid = std::thread::current().id();
        println!("{tid:?}: Var Checking with local={local_version:?} ref={ref_version:?}");

        local_version > 0 && local_version <= ref_version.as_isize()
    }

    /// Read with version checking
    // pub(crate) fn read_with_check(&self, read_version: Version) -> Option<T> {
    //     // Read the data first
    //     let data = self.value.get();
    //     // post-validation:
    //     if self.check(read_version) {
    //         Some(data)
    //     } else {
    //         None
    //     }
    // }

    pub(crate) fn read_with_double_check(&self, read_version: Version) -> Option<T> {
        // Pre-Validation
        let pre_version = self.local_version.load(Ordering::SeqCst);
        
        if pre_version < 0 {
            return None
        }

        if pre_version > read_version.as_isize() {
            return None
        }

        // read the data
        let data = self.value.get();

        // Post-Validation
        let post_version = self.local_version.load(Ordering::SeqCst);
        
        if post_version != pre_version {
            return None
        }

        Some(data)
    }

    /// Try get the write lock
    pub(crate) fn try_lock(&self) -> Option<Guard<'_, T>> {
        let current_version = self.local_version.load(Ordering::SeqCst);

        if current_version < 0 {
            // already locked by others
            return None;
        }

        // do a CAS action to make the version be negative
        let result = self.local_version.compare_exchange(
            current_version,
            -current_version,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );

        match result {
            // Lock successfully
            Ok(_) => Some(Guard {
                var: self,
                new_local_version: Version::new(current_version).unwrap_or_else(|| {
                    unreachable!("current_version cannot be negative, because we checked before")
                }),
            }),
            Err(_) => None,
        }
    }
}

pub(crate) struct Guard<'var, T> {
    var: &'var TVar<T>,
    new_local_version: Version,
}

impl<'var, T: Copy> Guard<'var, T> {
    // Update the local version of TVar
    pub(crate) fn set_version(&mut self, write_version: Version) {
        self.new_local_version = write_version;
    }

    pub(crate) fn write(&mut self, new_value: T) {
        self.var.value.set(new_value);
    }
}

impl<'var, T> Drop for Guard<'var, T> {
    fn drop(&mut self) {
        // Write the new_local_version
        self.var
            .local_version
            .store(self.new_local_version.as_isize(), Ordering::SeqCst);

        let tid = std::thread::current().id();
        let version = self.new_local_version;
        println!("{tid:?}: Var Lock realeased with new_version={version:?}");
    }
}
