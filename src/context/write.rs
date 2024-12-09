use crate::{var::Guard, version::Version, version_clock::VersionClock, StmError, TVar};

#[cfg(feature = "small_alloc")]
use smallvec::SmallVec;

// A write transaction context
// will log read and write set
pub struct Context<'var> {
    read_version: Version,

    #[cfg(not(feature = "small_alloc"))]
    read_set: Vec<Box<dyn ReadLog + 'var>>,
    #[cfg(not(feature = "small_alloc"))]
    write_set: Vec<Box<dyn WriteLog + 'var>>,

    // small_alloc us small_vec
    #[cfg(feature = "small_alloc")]
    read_set: SmallVec<[Box<dyn ReadLog + 'var>; 8]>,
    #[cfg(feature = "small_alloc")]
    write_set: SmallVec<[Box<dyn WriteLog + 'var>; 8]>,
}

impl<'var> Context<'var> {
    pub fn new(read_version: Version) -> Self {
        Context {
            read_version,

            #[cfg(not(feature = "small_alloc"))]
            read_set: Vec::new(),
            #[cfg(not(feature = "small_alloc"))]
            write_set: Vec::new(),

            #[cfg(feature = "small_alloc")]
            read_set: SmallVec::new(),
            #[cfg(feature = "small_alloc")]
            write_set: SmallVec::new(),
        }
    }

    pub fn read<T: Copy>(&mut self, var: &'var TVar<T>) -> Result<T, StmError> {
        // Log it to read_set
        if let None = self.try_get_read_log_ref(var) {
            // Not logged
            // log it to read_set
            let log = ReadLogEntry { var };
            self.read_set.push(Box::new(log));
        }
        // Check we wrote before
        if let Some(log) = self.try_get_write_log_mut(var) {
            // read data from buffer
            let buffer = unsafe { &*(log.buffer_ptr() as *const () as *const T) };
            let buffer = *buffer;

            // do an check ?
            var.check(self.read_version)
                .then_some(buffer)
                .ok_or_else(|| match () {
                    #[cfg(not(feature = "retry_info"))]
                    () => StmError::Retry,
                    #[cfg(feature = "retry_info")]
                    () => StmError::Retry("Post-validation failed"),
                })
        } else {
            // Read data from var
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
        // Check we wrote before
        if let Some(log) = self.try_get_write_log_mut(var) {
            let value_mut = &mut unsafe { *(log.buffer_ptr() as *mut T) };
            *value_mut = value;
        } else {
            // create a new log
            let new_log = Box::new(WriteLogEntry { var, buffer: value });
            self.write_set.push(new_log);
        }

        Ok(())
    }

    pub fn reset(&mut self, read_version: Version) {
        self.read_version = read_version;
        self.read_set.clear();
        self.write_set.clear();
    }

    pub fn try_commit(&mut self, clock: &VersionClock) -> Result<(), StmError> {
        // try get lock write set
        let mut guards = Vec::new();

        for write_log in &self.write_set {
            let guard = write_log.try_lock().ok_or_else(|| match () {
                #[cfg(not(feature = "retry_info"))]
                () => StmError::Retry,
                #[cfg(feature = "retry_info")]
                () => StmError::Retry("Lock write-set failed"),
            })?;
            guards.push(guard);
        }

        // tick the global version clock
        let write_version = clock.tick();

        // when wv = rv + 1
        // Don't need to validate
        if write_version.as_isize() > self.read_version.as_isize() + 1 {
            let id = std::thread::current().id();
            let read_version = self.read_version.as_isize();
            // validate the read set
            for read_log in &self.read_set {
                let mut version = read_log.version();

                let wv = write_version.as_isize();
                println!("    - {id:?} Validate Read-Set: RV={read_version} WV={wv} lock={version}");

                if version < 0 {
                    // check it was locked by ourselves
                    let locked_by_self = self
                        .write_set
                        .iter()
                        .any(|write_log| std::ptr::eq(write_log.var_ptr(), read_log.var_ptr()));

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

                    version = -version
                }

                // check the version
                if version > self.read_version.as_isize() {
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

        for guard in &mut guards {
            // Write the buffer to var
            guard.write_from_buffer();

            // update the version
            guard.set_version(write_version);
        }

        // Lock released here

        Ok(())
    }
}

// Internal methods
impl<'var> Context<'var> {
    fn try_get_read_log_ref<T: Copy>(
        &self,
        var: &'var TVar<T>,
    ) -> Option<&'_ Box<dyn ReadLog + 'var>> {
        let var_ptr = var as *const _ as *const ();
        self.read_set
            .iter()
            .find(|entry| std::ptr::eq(entry.var_ptr(), var_ptr))
    }

    fn try_get_write_log_mut<T>(
        &mut self,
        var: &'var TVar<T>,
    ) -> Option<&'_ mut Box<dyn WriteLog + 'var>> {
        let var_ptr = var as *const _ as *const ();
        self.write_set
            .iter_mut()
            .find(|entry| std::ptr::eq(entry.var_ptr(), var_ptr))
    }
}

trait ReadLog {
    fn var_ptr(&self) -> *const ();

    fn version(&self) -> isize;
}

trait WriteLog: ReadLog {
    fn buffer_ptr(&mut self) -> *mut ();

    fn try_lock(&self) -> Option<Box<dyn WriteLogGuard + '_>>;
}

struct ReadLogEntry<'var, T> {
    pub var: &'var TVar<T>,
}

impl<'var, T: Copy> ReadLog for ReadLogEntry<'var, T> {
    fn var_ptr(&self) -> *const () {
        &*self.var as *const TVar<T> as *const ()
    }

    fn version(&self) -> isize {
        self.var.version()
    }
}

struct WriteLogEntry<'var, T> {
    var: &'var TVar<T>,
    buffer: T,
}

impl<'var, T: Copy> ReadLog for WriteLogEntry<'var, T> {
    fn var_ptr(&self) -> *const () {
        &*self.var as *const TVar<T> as *const ()
    }

    fn version(&self) -> isize {
        self.var.version()
    }
}

impl<'var, T: Copy> WriteLog for WriteLogEntry<'var, T> {
    fn buffer_ptr(&mut self) -> *mut () {
        &mut self.buffer as *mut T as *mut ()
    }

    fn try_lock(&self) -> Option<Box<dyn WriteLogGuard + '_>> {
        self.var
            .try_lock()
            .map(|guard| WriteLogEntryGuard {
                guard,
                buffer: self.buffer,
            })
            .map(|guard| Box::new(guard) as Box<dyn WriteLogGuard>)
    }
}

trait WriteLogGuard {
    fn write_from_buffer(&mut self);

    fn set_version(&mut self, write_version: Version);
}

struct WriteLogEntryGuard<'var, T> {
    guard: Guard<'var, T>,
    buffer: T,
}

impl<'var, T: Copy> WriteLogGuard for WriteLogEntryGuard<'var, T> {
    fn write_from_buffer(&mut self) {
        self.guard.write(self.buffer);
    }

    fn set_version(&mut self, write_version: Version) {
        self.guard.set_version(write_version);
    }
}
