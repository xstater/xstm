use crate::{version::Version, version_clock::VersionClock, StmError, TVar};

use read_set::ReadSet;

mod any_var;
mod read_set;
mod write_set;
use write_set::WriteSet;

// A write transaction context
// will log read and write set
pub struct Context<'var> {
    read_version: Version,
    write_set: WriteSet<'var>,
    read_set: ReadSet<'var>,
}

impl<'var> Context<'var> {
    pub fn new(read_version: Version) -> Self {
        Context {
            read_version,
            write_set: WriteSet::new(),
            read_set: ReadSet::new(),
        }
    }

    pub fn read<T: Copy>(&mut self, var: &'var TVar<T>) -> Result<T, StmError> {
        // Log it to read_set
        self.read_set.log(var);

        // Check we wrote before
        if let Some(wrote_value) = self.write_set.try_read(var) {
            Ok(wrote_value)
        } else {
            // read from TVar
            var.read_with_check(self.read_version)
                .ok_or_else(|| match () {
                    #[cfg(not(feature = "retry_info"))]
                    () => StmError::Retry,
                    #[cfg(feature = "retry_info")]
                    () => StmError::Retry("Post-validation failed"),
                })
        }
    }

    pub fn write<T: Copy>(&mut self, var: &'var TVar<T>, value: T) -> Result<(), StmError> {
        // log it to write_set
        self.write_set.log(var, value);

        Ok(())
    }

    pub fn reset(&mut self, read_version: Version) {
        self.read_version = read_version;
        self.write_set.clear();
        self.read_set.clear();
    }

    pub fn try_commit(&mut self, clock: &VersionClock) -> Result<(), StmError> {
        // try get lock write set

        let mut guard = self.write_set
            .try_lock(10)
            .ok_or_else(|| match () {
                #[cfg(not(feature = "retry_info"))]
                () => StmError::Retry,
                #[cfg(feature = "retry_info")]
                () => StmError::Retry("Lock write-set failed"),
            })?;
            
        // tick the global version clock
        let write_version = clock.tick();

        // when wv = rv + 1
        // Don't need to validate
        if write_version > self.read_version + 1 {
            // validate the read set
            for read_entry in self.read_set.iter_vars() {
                let mut version = read_entry.lock.version();

                if version.is_locked() {
                    // check it was locked by ourselves
                    let locked_by_self = guard
                        .iter_vars()
                        .any(|write_entry| read_entry == write_entry);

                    if !locked_by_self {
                        // locked by others
                        return Err(match () {
                            #[cfg(not(feature = "retry_info"))]
                            () => StmError::Retry,
                            #[cfg(feature = "retry_info")]
                            () => StmError::Retry(
                                "Validate Read-Set Failed: TVar was locked by other transaction",
                            ),
                        });
                    }

                    // locked by self
                    // make it positive for compare with read_version
                    version = -version
                }

                // check the version
                if version > self.read_version {
                    return Err(match () {
                        #[cfg(not(feature = "retry_info"))]
                        () => StmError::Retry,
                        #[cfg(feature = "retry_info")]
                        () => StmError::Retry(
                            "Validate Read-Set Failed: TVar was changed by other transaction",
                        ),
                    });
                }
            }
        }

        // Write the data and the version
        guard.set_version(write_version);

        guard.write_data_from_buffer();

        // guard dropped here

        Ok(())
    }
}
