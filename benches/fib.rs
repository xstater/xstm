use std::sync::Arc;

use divan::Bencher;
use xstm::{Context, Stm, StmError, TVar, Transaction};

struct Fib {
    var_a: Arc<TVar<u128>>,
    var_b: Arc<TVar<u128>>,
}

impl Transaction for Fib {
    type Output = u128;

    fn atomically<'this: 'var, 'context, 'var>(
        &'this self,
        context: &'context mut Context<'var>,
    ) -> Result<Self::Output, StmError> {
        let a = context.read(&self.var_a)?;
        let b = context.read(&self.var_b)?;
        let result = a + b;

        context.write(&self.var_a, b)?;
        context.write(&self.var_b, result)?;

        Ok(result)
    }
}

fn main() {
    divan::main();
}

fn thread_counts() -> Vec<usize> {
    vec![0 /* all threads */, 1, 2, 4, 8, 16, 32]
}

#[divan::bench(threads = thread_counts() )]
fn stm(bencher: Bencher) {
    let stm = Arc::new(Stm::new());

    let var_a = Arc::new(TVar::new(1));
    let var_b = Arc::new(TVar::new(1));

    bencher.bench(|| {
        let fib = Fib {
            var_a: var_a.clone(),
            var_b: var_b.clone(),
        };
        stm.atomically(fib)
    });
}

#[divan::bench(threads = thread_counts())]
fn lock(bencher: Bencher) {
    let var_a = Arc::new(std::sync::Mutex::new(1));
    let var_b = Arc::new(std::sync::Mutex::new(1));

    bencher.bench(|| {
        let mut guard_a = var_a.lock().unwrap();
        let mut guard_b = var_b.lock().unwrap();

        let a = *guard_a;
        let b = *guard_b;
        let result = a + b;

        *guard_a = b;
        *guard_b = result;
    });
}
