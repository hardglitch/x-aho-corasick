use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use aho_corasick::{AhoCorasick, PatternID};
use x_aho_corasick::{FastPatternMatcher, Match};
use daachorse::DoubleArrayAhoCorasick;

#[allow(clippy::unwrap_used)]
fn bench_proof(c: &mut Criterion) {
    let patterns = vec!["ma", "mama", "cat", "at", "ма", "кошка", "лужа", "каша"];
    let text = "Мама (mama) вела кошку (cat) к луже, а в каше была (was) малина (macatma)!".to_lowercase();

    let ac = AhoCorasick::new(&patterns).expect("Something went wrong");
    let ac_matches = ac.find_iter(&text)
        .map(|mat| (mat.start(), mat.pattern()))
        .collect::<Vec<(usize, PatternID)>>();

    let test_data = vec![
        (0, PatternID::must(4)),
        (4, PatternID::must(4)),
        (10, PatternID::must(0)),
        (12, PatternID::must(0)),
        (37, PatternID::must(2)),
        (85, PatternID::must(4)),
        (99, PatternID::must(0)),
        (101, PatternID::must(2)),
        (104, PatternID::must(0)),
    ];

    assert_eq!(ac_matches, test_data);
	
	
	let pma = DoubleArrayAhoCorasick::new(&patterns).unwrap();
	let pma_matches = pma.find_iter(&text)
        .map(|mat| (mat.start(), mat.value()))
        .collect::<Vec<(usize, usize)>>();

    let test_data = vec![
        (0, 4),
        (4, 4),
        (10, 0),
        (12, 0),
        (37, 2),
        (85, 4),
        (99, 0),
        (101, 2),
        (104, 0),
    ];

    assert_eq!(pma_matches, test_data);


    let matcher = FastPatternMatcher::new(&patterns);
    let matches = matcher.find_all_in(&text).collect::<Vec<Match>>();
    let test_data = [
        Match::new(0, 4),
        Match::new(4, 4),
        Match::new(10, 0),
        Match::new(12, 0),
        Match::new(37, 2),
        Match::new(85, 4),
        Match::new(99, 0),
        Match::new(101, 2),
        Match::new(104, 0),
    ];

    assert_eq!(matches, test_data);

    // Bench
    let mut group = c.benchmark_group("Comparison");

    // aho-corasick
    group.bench_function("aho-corasick (search)", |b|
        b.iter(|| {
            let matches: Vec<_> = ac.find_iter(black_box(&text)).collect();
            black_box(matches);
        })
    );

    // daachorse
    group.bench_function("daachorse (search)", |b|
        b.iter(|| {
            let matches: Vec<_> = pma.find_iter(black_box(&text)).collect();
            black_box(matches);
        })
    );

    // x-aho-corasick
    group.bench_function("x-aho-corasick (search)", |b|
        b.iter(|| {
            let matches: Vec<_> = matcher.find_all_in(black_box(&text)).collect();
            black_box(matches);
        })
    );
}

criterion_group!(benches, bench_proof);
criterion_main!(benches);
