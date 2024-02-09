use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use soapy::{SliceRef, Soa, Soapy};

struct Rng(StdRng);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }

    pub fn next_f32(&mut self) -> f32 {
        f32::from_ne_bytes(self.0.next_u32().to_ne_bytes())
    }
}

#[derive(Soapy, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vec4(
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
}

fn make_vec4_list<T>(rng: &mut Rng, count: usize) -> T
where
    T: FromIterator<Vec4>,
{
    std::iter::repeat_with(|| Vec4::new_rng(rng))
        .take(count)
        .collect()
}

macro_rules! sum_dots {
    ($a:ident, $b:ident) => {
        $a.into_iter()
            .zip($b.into_iter())
            .map(|(a, b)| a.0 * b.0 + a.1 * b.1 + a.2 * b.2 + a.3 * b.3)
            .sum()
    };
}

pub fn sum_dots_vec(a: &[Vec4], b: &[Vec4]) -> f32 {
    sum_dots!(a, b)
}

pub fn sum_dots_soa(a: SliceRef<Vec4>, b: SliceRef<Vec4>) -> f32 {
    sum_dots!(a, b)
}

#[derive(Soapy, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mat4x4(Vec4, Vec4, Vec4, Vec4);

impl Mat4x4 {
    fn new_rng(rng: &mut Rng) -> Self {
        Self(
            Vec4::new_rng(rng),
            Vec4::new_rng(rng),
            Vec4::new_rng(rng),
            Vec4::new_rng(rng),
        )
    }
}

fn make_mat4x4_list<T>(rng: &mut Rng, count: usize) -> T
where
    T: FromIterator<Mat4x4>,
{
    std::iter::repeat_with(|| Mat4x4::new_rng(rng))
        .take(count)
        .collect()
}

macro_rules! mul_mats {
    ($a:ident, $b:ident) => {
        $a.into_iter().zip($b.into_iter()).map(|(a, b)| {
            Mat4x4(
                Vec4(
                    a.0 .0 * b.0 .0,
                    a.0 .1 * b.1 .0,
                    a.0 .2 * b.2 .0,
                    a.0 .3 * b.3 .0,
                ),
                Vec4(
                    a.1 .0 * b.0 .1,
                    a.1 .1 * b.1 .1,
                    a.1 .2 * b.2 .1,
                    a.1 .3 * b.3 .1,
                ),
                Vec4(
                    a.2 .0 * b.0 .2,
                    a.2 .1 * b.1 .2,
                    a.2 .2 * b.2 .2,
                    a.2 .3 * b.3 .2,
                ),
                Vec4(
                    a.3 .0 * b.0 .3,
                    a.3 .1 * b.1 .3,
                    a.3 .2 * b.2 .3,
                    a.3 .3 * b.3 .3,
                ),
            )
        })
    };
}

pub fn mul_mats_vec<'a>(a: &'a [Mat4x4], b: &'a [Mat4x4]) -> impl Iterator<Item = Mat4x4> + 'a {
    mul_mats!(a, b)
}

pub fn mul_mats_soa<'a>(
    a: SliceRef<'a, Mat4x4>,
    b: SliceRef<'a, Mat4x4>,
) -> impl Iterator<Item = Mat4x4> + 'a {
    mul_mats!(a, b)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = Rng::new(42);

    c.bench_function("dots-soa", |b| {
        let soa1: Soa<_> = make_vec4_list(&mut rng, 1 << 16);
        let soa2: Soa<_> = make_vec4_list(&mut rng, 1 << 16);
        b.iter(|| sum_dots_soa(soa1.as_slice(), soa2.as_slice()))
    });

    c.bench_function("dots-vec", |b| {
        let vec1: Vec<_> = make_vec4_list(&mut rng, 1 << 16);
        let vec2: Vec<_> = make_vec4_list(&mut rng, 1 << 16);
        b.iter(|| sum_dots_vec(&vec1, &vec2))
    });

    c.bench_function("mats-soa", |b| {
        let soa1: Soa<_> = make_mat4x4_list(&mut rng, 1 << 16);
        let soa2: Soa<_> = make_mat4x4_list(&mut rng, 1 << 16);
        b.iter(|| mul_mats_soa(soa1.as_slice(), soa2.as_slice()))
    });

    c.bench_function("mats-vec", |b| {
        let vec1: Vec<_> = make_mat4x4_list(&mut rng, 1 << 16);
        let vec2: Vec<_> = make_mat4x4_list(&mut rng, 1 << 16);
        b.iter(|| mul_mats_vec(&vec1, &vec2))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
