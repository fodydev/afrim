#[cfg(feature = "rhai")]
use afrim_translator::Engine;
use afrim_translator::Translator;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use indexmap::IndexMap;

pub fn translate(c: &mut Criterion) {
    // Generates the dataset.
    let mut dictionary = IndexMap::new();
    let words = [
        "Aassiaiel",
        "Asiel",
        "AazAiel",
        "Aziel",
        "Aazaryas",
        "Uzziah",
        "Azaria",
        "Aazazya",
        "Azaziah",
        "Aazraiel",
        "Azriel",
        "Aebdaiel",
        "Abdeel",
    ];
    (0..100_000).for_each(|_| {
        words.into_iter().for_each(|word| {
            dictionary.insert(word.to_owned(), vec![word.to_owned()]);
        });
    });

    // Initializes the translator.
    let mut translator = Translator::new(dictionary, false);

    // Registers a translator.
    #[cfg(feature = "rhai")]
    {
        let engine = Engine::new();
        let script = engine
            .compile("fn translate(input) { [input, \"\", input, false] }")
            .unwrap();
        translator.register("give_back".to_owned(), script);
    }

    // Generates candidates for testing.
    let mut candidates = Vec::new();
    "Aazraiel".chars().fold("".to_owned(), |word, character| {
        let new_word = format!("{word}{character}");
        candidates.push(new_word.clone());

        new_word
    });

    // Setup the benchmark.
    let mut group = c.benchmark_group("translate");
    for candidate in candidates.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(candidate),
            candidate,
            |b, candidate| {
                b.iter(|| {
                    translator.translate(black_box(candidate));
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, translate);

criterion_main!(benches);
