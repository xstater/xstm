fn main() {
    divan::main();
}


#[divan::bench(args = [1,2,4,8,16,32])]
fn fib(n : u64) -> u64 {
    if n <= 1 {
        1 
    } else {
        fib(n - 2) + fib(n -1)
    }
}
