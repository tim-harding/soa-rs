use criterion::{criterion_group, criterion_main, Criterion};
use rand::{RngCore, SeedableRng};
use soapy::{Slice, Soa, Soapy};

#[derive(Soapy, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vec4(f32, f32, f32, f32);

fn random_vec4s(count: usize, seed: u64) -> impl Iterator<Item = Vec4> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut next_f32 = move || f32::from_ne_bytes(rng.next_u32().to_ne_bytes());
    (0..count).map(move |_| Vec4(next_f32(), next_f32(), next_f32(), next_f32()))
}

pub fn sum_dots_vec(a: Vec<Vec4>, b: Vec<Vec4>) -> f32 {
    a.into_iter()
        .zip(b.into_iter())
        .map(|(l, r)| l.0 * r.0 + l.1 * r.1 + l.2 * r.2 + l.3 * r.3)
        .sum()
}

pub fn sum_dots_soa(a: Soa<Vec4>, b: Soa<Vec4>) -> f32 {
    Slice::try_fold_zip(&a, &b, 0.0, |acc, l, r| {
        std::ops::ControlFlow::Continue(acc + l.0 * r.0 + l.1 * r.1 + l.2 * r.2 + l.3 * r.3)
    })
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
    c.bench_function("dots-vec", |b| {
        let lists = make_lists::<Vec<_>>(4096);
        b.iter_batched(
            || lists.clone(),
            |(soa1, soa2)| sum_dots_vec(soa1, soa2),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("dots-soa", |b| {
        let lists = make_lists::<Soa<_>>(4096);
        b.iter_batched(
            || lists.clone(),
            |(soa1, soa2)| sum_dots_soa(soa1, soa2),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
