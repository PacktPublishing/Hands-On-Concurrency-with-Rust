#[macro_use]
extern crate criterion;
extern crate feruscore;

use criterion::Criterion;
use feruscore::individual::*;
use feruscore::mars::*;

fn bench_imp_and_dwarf(core_size: u16, rounds: u16) -> () {
    let mut mars = MarsBuilder::default().core_size(core_size).freeze();
    let imp = ringers::imp(core_size);
    let dwarf = ringers::dwarf(core_size);
    mars.compete(rounds, &imp, &dwarf);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("imp v. dwarf", |b| {
        b.iter(|| bench_imp_and_dwarf(8000, 100))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
