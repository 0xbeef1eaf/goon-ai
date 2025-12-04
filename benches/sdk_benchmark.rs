use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use goon_ai::sdk::generator::generate_definitions;

fn benchmark_sdk_generation(c: &mut Criterion) {
    let allowed_modules = vec![
        "image".to_string(),
        "video".to_string(),
        "audio".to_string(),
        "hypno".to_string(),
        "wallpaper".to_string(),
        "write_lines".to_string(),
    ];

    c.bench_function("generate_definitions", |b| {
        b.iter(|| generate_definitions(&allowed_modules))
    });
}

criterion_group!(benches, benchmark_sdk_generation);
criterion_main!(benches);
