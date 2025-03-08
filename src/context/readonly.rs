use std::marker::PhantomData;

use crate::{version::Version, StmError, TVar};

/// A read-only transaction context
/// (Don't log any read or write set)
pub struct Context<'var> {
    _marker: PhantomData<&'var ()>,
    // Indicate the context tried perform a write operation
    tried_writing: bool,
    read_version: Version,
}

impl<'var> Context<'var> {
    pub fn new(read_version: Version) -> Self {
        Context {
            _marker: PhantomData,
            tried_writing: false,
            read_version,
        }
    }

    pub fn read<T: Copy>(&mut self, var: &'var TVar<T>) -> Result<T, StmError> {
        var.read_with_check(self.read_version)
            .ok_or_else(|| {
                match () {
                    #[cfg(not(feature = "retry_info"))]
                    () => StmError::Retry,
                    #[cfg(feature = "retry_info")]
                    () => StmError::Retry("Read Variable But validation was failed")
                }
            })
    }

    pub fn write<T: Copy>(&mut self, _: &'var TVar<T>, _: T) -> Result<(), StmError> {
        // Cannot perform a write operation
        // Just set the flag an return

        self.tried_writing = true;

        Err(match () {
            #[cfg(not(feature = "retry_info"))]
            () => StmError::Retry,
            #[cfg(feature = "retry_info")]
            () => StmError::Retry(
                "Trying Write in Read-Only Transaction Context",
            )
        })
    }

    pub fn tried_writing(&self) -> bool {
        self.tried_writing
    }

    pub fn reset(&mut self, read_version: Version) {
        self.read_version = read_version;
    }

    pub fn try_commit(&mut self) -> Result<(), StmError> {
        // Committing a read-only transaction is always successful
        Ok(())
    }
}
