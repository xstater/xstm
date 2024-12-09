mod version;
mod version_clock;

mod transaction;
pub use transaction::Transaction;

mod context;
pub use context::Context;

mod var;
pub use var::TVar;

mod stm;
pub use stm::Stm;

#[cfg(feature = "retry_info")]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum StmError {
    Retry(&'static str),
}
#[cfg(not(feature = "retry_info"))]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum StmError {
    Retry,
}
