use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use soapy::{SliceRef, Soa, Soapy};

pub struct Rng(StdRng);

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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vec4Arrays<const N: usize>([f32; N], [f32; N], [f32; N], [f32; N]);

impl<const N: usize> Vec4Arrays<N> {
    pub fn new_rng(rng: &mut Rng) -> Self {
        let mut out = Self([0.0; N], [0.0; N], [0.0; N], [0.0; N]);
        for i in 0..N {
            out.0[i] = rng.next_f32();
            out.1[i] = rng.next_f32();
            out.2[i] = rng.next_f32();
            out.3[i] = rng.next_f32();
        }
        out
    }

    pub fn become_iter(self) -> impl Iterator<Item = (f32, f32, f32, f32)> {
        self.0
            .into_iter()
            .zip(self.1)
            .zip(self.2)
            .zip(self.3)
            .map(|(((a0, a1), a2), a3)| (a0, a1, a2, a3))
    }
}

pub fn sum_dots_arrays<const N: usize>(a: Vec4Arrays<N>, b: Vec4Arrays<N>) -> f32 {
    a.become_iter()
        .zip(b.become_iter())
        .map(|(a, b)| a.0 * b.0 + a.1 * b.1 + a.2 * b.2 + a.3 * b.3)
        .sum()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = Rng::new(42);

    let array1 = <Vec4Arrays<{ 1 << 16 }>>::new_rng(&mut rng);
    let array2 = <Vec4Arrays<{ 1 << 16 }>>::new_rng(&mut rng);
    c.bench_function("dots-array", |b| b.iter(|| sum_dots_arrays(array1, array2)));

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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
