use criterion::{criterion_group, criterion_main, Criterion};
use rand::{RngCore, SeedableRng};
use soapy::{Soa, Soapy};

#[derive(Soapy, Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Vec4(f32, f32, f32, f32);

impl Vec4 {
    pub fn dot(self, other: Self) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2 + self.3 * other.3
    }
}

fn random_vec4s(count: usize, seed: u64) -> impl Iterator<Item = Vec4> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut next_f32 = move || f32::from_ne_bytes(rng.next_u32().to_ne_bytes());
    (0..count).map(move |_| Vec4(next_f32(), next_f32(), next_f32(), next_f32()))
}

fn dots<T>(a: T, b: T) -> Vec<f32>
where
    T: IntoIterator<Item = Vec4>,
{
    a.into_iter()
        .zip(b.into_iter())
        .map(|(l, r)| l.dot(r))
        .collect()
}

fn make_lists<T>(count: usize) -> (T, T)
where
    T: FromIterator<Vec4>,
{
    (
        random_vec4s(count, 42).collect(),
        random_vec4s(count, 69).collect(),
    )
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Vec dots", |b| {
        let (vec1, vec2) = make_lists::<Vec<_>>(4096);
        b.iter(|| dots(vec1.clone(), vec2.clone()))
    });
    c.bench_function("Soa dots", |b| {
        let (soa1, soa2) = make_lists::<Soa<_>>(4096);
        b.iter(|| dots(soa1.clone(), soa2.clone()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
