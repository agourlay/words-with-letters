use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn word_can_build_from_letters_short() -> bool {
    let word = "happy".to_string();
    let chars = vec!['h', 'a', 'p', 'y', 'p'];
    // TODO how to access code from the main module?
    true
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("word_can_build_from_letters short", |b| {
        b.iter(|| word_can_build_from_letters_short())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
