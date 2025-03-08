use crate::version::Version;
use crate::versioned_lock::VersionedLock;
use crate::Transaction;
use std::cell::Cell;
use std::fmt::Debug;

pub struct TVar<T> {
    value: Cell<T>,
    versioned_lock: VersionedLock,
}

// We can only Read/Write TVar in transaction
// TVar can be Sync safely
unsafe impl<T: Copy> Sync for TVar<T> {}

// Public method
impl<T: Copy> TVar<T> {
    pub fn new(value: T) -> Self {
        TVar {
            value: Cell::new(value),
            versioned_lock: VersionedLock::new(),
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
    pub(crate) fn value_ptr(&self) -> *const T {
        self.value.as_ptr()
    }

    pub(crate) fn get_lock(&self) -> &'_ VersionedLock {
        &self.versioned_lock
    }

    pub(crate) fn read_with_check(&self, read_version: Version) -> Option<T> {
        // Pre-Validation
        let pre_version = self.versioned_lock.version();

        if !pre_version.check(read_version) {
            return None;
        }

        // read the data
        let data = self.value.get();

        // Post-Validation
        let post_version = self.versioned_lock.version();

        // check the data was not changed
        if post_version != pre_version {
            return None;
        }

        Some(data)
    }
}

impl<T: Debug + Copy> Debug for TVar<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TVar")
            .field("value", &self.value)
            .field("versioned_lock", &self.versioned_lock)
            .finish()
    }
}
