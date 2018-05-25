#[macro_use]
extern crate criterion;
extern crate naive_hashmap;
extern crate rand;

use criterion::{Criterion, Fun};
use rand::{Rng, SeedableRng, XorShiftRng};

fn insert_and_lookup_specialized(mut n: u64) {
    let mut rng: XorShiftRng = SeedableRng::from_seed([1981, 1986, 2003, 2011]);
    let mut hash_map = naive_hashmap::HashMapU8::new();

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

fn insert_and_lookup_standard(mut n: u64) {
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

macro_rules! insert_lookup {
    ($fn:ident, $s:expr) => {
        fn $fn(c: &mut Criterion) {
            let specialized = Fun::new("specialized", |b, i| b.iter(|| insert_and_lookup_specialized(*i)));
            let standard = Fun::new("standard", |b, i| b.iter(|| insert_and_lookup_standard(*i)));

            let functions = vec![specialized, standard];

            c.bench_functions(&format!("HashMap/{}", $s), functions, &$s);
        }
    }
}

insert_lookup!(insert_lookup_100000, 100_000);
insert_lookup!(insert_lookup_10000, 10_000);
insert_lookup!(insert_lookup_1000, 1_000);
insert_lookup!(insert_lookup_100, 100);
insert_lookup!(insert_lookup_10, 10);
insert_lookup!(insert_lookup_1, 1);

criterion_group!{
    name = benches;
    config = Criterion::default();
    targets = insert_lookup_100000, insert_lookup_10000, insert_lookup_1000, insert_lookup_100, insert_lookup_10, insert_lookup_1
}
criterion_main!(benches);
