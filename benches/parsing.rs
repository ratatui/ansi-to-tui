pub use ansi_to_tui::IntoText;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    const BASIC: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ascii/archlinux.ascii"
    ));
    c.bench_function("Parsing bench", |b| {
        b.iter(|| {
            let s = black_box(&BASIC);
            black_box(s.into_text()).unwrap();
        })
    });
    const CODE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/ascii/code.ascii"));
    c.bench_function("Parsing bench code", |b| {
        b.iter(|| {
            let s = black_box(&CODE);
            black_box(s.into_text()).unwrap();
        })
    });

    const HYPERLINK: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ascii/hyperlink.ascii"
    ));
    c.bench_function("Parsing hyperlink code", |b| {
        b.iter(|| {
            let s = black_box(&HYPERLINK);
            black_box(s.into_text()).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
