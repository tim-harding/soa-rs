use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use soa_rs::{Soa, Soars};

struct Rng(StdRng);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }

    fn next_f32(&mut self) -> f32 {
        f32::from_ne_bytes(self.0.next_u32().to_ne_bytes())
    }

    fn collect_vec4<T>(&mut self, count: usize) -> T
    where
        T: FromIterator<Vec4>,
    {
        std::iter::repeat_with(|| Vec4::new_rng(self))
            .take(count)
            .collect()
    }
}

#[derive(Soars, Debug, Clone, Copy, PartialEq, PartialOrd)]
#[soa_derive(Debug, PartialEq, PartialOrd)]
struct Vec4(
    #[align(64)] f32,
    #[align(64)] f32,
    #[align(64)] f32,
    #[align(64)] f32,
);

impl Vec4 {
    fn new_rng(rng: &mut Rng) -> Self {
        Self(
            rng.next_f32(),
            rng.next_f32(),
            rng.next_f32(),
            rng.next_f32(),
        )
    }

    fn dot(&self, other: &Self) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2 + self.3 * other.3
    }
}

impl Vec4Ref<'_> {
    fn dot(&self, other: &Self) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2 + self.3 * other.3
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = Rng::new(42);

    let soa1: Soa<_> = rng.collect_vec4(1 << 16);
    let soa2: Soa<_> = rng.collect_vec4(1 << 16);
    c.bench_function("soa", |b| {
        b.iter(|| {
            soa1.iter()
                .zip(soa2.iter())
                .map(|(a, b)| a.dot(&b))
                .sum::<f32>()
        })
    });

    let vec1: Vec<_> = rng.collect_vec4(1 << 16);
    let vec2: Vec<_> = rng.collect_vec4(1 << 16);
    c.bench_function("vec", |b| {
        b.iter(|| {
            vec1.iter()
                .zip(vec2.iter())
                .map(|(a, b)| a.dot(b))
                .sum::<f32>()
        })
    });

    c.bench_function("chunked-soa", |b| {
        b.iter(|| {
            soa1.chunks_exact(8)
                .zip(soa2.chunks_exact(8))
                .fold([0.; 8], |acc, (a, b)| {
                    std::array::from_fn(|i| acc[i] + a.idx(i).dot(&b.idx(i)))
                })
                .into_iter()
                .sum::<f32>()
        })
    });

    #[rustfmt::skip]
    c.bench_function("chunked-vec", |b| {
        b.iter(|| {
            vec1.chunks_exact(8).zip(vec2.chunks_exact(8)).fold(
                [0.; 8],
                |acc, (a, b)| {
                    std::array::from_fn(|i| {
                        acc[i] + a[i].dot(&b[i])
                    })
                },
            ).into_iter().sum::<f32>()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
