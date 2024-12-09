mod ext;

// pub use ext::*;

use crate::{Context, StmError};

pub trait Transaction{
    type Output;

    fn atomically<'this: 'var, 'context, 'var>(&'this self, context: &'context mut Context<'var>) -> Result<Self::Output, StmError>;
} 

// how to forbid (|trans| Ok(trans) )