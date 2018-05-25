#[macro_use]
extern crate criterion;
extern crate rand;

use criterion::Criterion;
use rand::{Rng, SeedableRng, XorShiftRng};

fn insert_and_lookup(mut n: u64) {
    let mut rng: XorShiftRng = SeedableRng::from_seed([1981, 1986, 2003, 2011]);
    let mut hash_map = ::std::collections::HashMap::new();

    while n != 0 {
        let key = rng.gen::<u8>();
        if rng.gen::<bool>() {
            let value = rng.gen::<u32>();
            hash_map.insert(key, value);
        } else {
            hash_map.get(&key);
        }
        n -= 1;
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "insert_and_lookup",
        |b, &&size| b.iter(|| insert_and_lookup(size)),
        &[1, 10, 100, 1_000, 10_000, 100_000],
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
