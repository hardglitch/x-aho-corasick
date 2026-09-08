use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::path::PathBuf;
use aho_corasick::AhoCorasick;
use x_aho_corasick::FastPatternMatcher;

#[allow(clippy::unwrap_used, clippy::expect_used)]
fn bench(c: &mut Criterion) {
    let p = PathBuf::from("benches").join("text.txt");
    let text = std::fs::read_to_string(&p).expect("File not found").to_lowercase();

    let p = PathBuf::from("benches").join("patterns.txt");
    let content = std::fs::read(p).expect("File not found");
    let patterns: Vec<&[u8]> =
        content
            .split(|&b| b == b'\n')
            .filter_map(|line| {
                let trimmed =
                    if line.ends_with(b"\r") { &line[..line.len() - 1] }
                    else { line };
                if !trimmed.is_empty() { Some(trimmed) } else { None }
            })
            .collect();

    // --- Bench ---

    let mut group = c.benchmark_group("Comparison");

    // aho-corasick
    let ac = AhoCorasick::new(&patterns).expect("Something went wrong");
    group.bench_function("aho-corasick (search)", |b|
        b.iter(|| {
            let matches: Vec<_> = ac.find_iter(black_box(&text)).collect();
            black_box(matches);
        })
    );

    // x-aho-corasick
    let matcher = FastPatternMatcher::new(&patterns);
    group.bench_function("x-aho-corasick (search)", |b|
        b.iter(|| {
            let matches: Vec<_> = matcher.find_all_in(black_box(&text)).collect();
            black_box(matches);
        })
    );
}

criterion_group!(benches, bench);
criterion_main!(benches);
