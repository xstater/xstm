use std::sync::Arc;
use xstm::{Context, Stm, StmError, TVar, Transaction};

const VARS_COUNT: usize = 10;

struct Update<'a> {
    vars: &'a [TVar<i32>],
}

impl<'a> Transaction for Update<'a> {
    type Output = ();

    fn atomically<'this: 'var, 'context, 'var>(
        &'this self,
        context: &'context mut Context<'var>,
    ) -> Result<Self::Output, StmError> {
        for var in self.vars {
            let x = context.read(var)?;
            context.write(var, x + 1)?
        }

        Ok(())
    }
}

#[tokio::test]
async fn basic() {
    let vars = std::iter::repeat_n(0, VARS_COUNT)
        .map(|_| TVar::new(0))
        .collect::<Vec<_>>();
    let vars = Arc::new(vars);

    let stm = Arc::new(Stm::new());

    let thread_count = 32;
    let repeat_count = 1000;

    let mut handles = Vec::new();
    for _ in 0..thread_count {
        let vars_ = vars.clone();
        let stm_ = stm.clone();

        let handle = tokio::task::spawn(async move {
            for _ in 0..repeat_count {
                let update = Update { vars: &vars_ };
                stm_.atomically(update);
            }
        });

        handles.push(handle);
    }
    let mut results = Vec::new();

    for handle in handles {
        results.push(handle.await.unwrap());
    }

    let vars = Arc::try_unwrap(vars).unwrap();
    for var in vars.into_iter() {
        let value = stm.atomically(var.read());
        assert_eq!(value, thread_count * repeat_count)
    }
}
