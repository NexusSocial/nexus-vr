//! Benchmarks common memory operations like memcpy, malloc, free, etc.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Fill;
use std::hint::black_box;

struct Dims {
	width: usize,
	height: usize,
}

impl Dims {
	const fn size(&self) -> usize {
		self.width * self.height
	}
}

const RES_1080: Dims = Dims {
	width: 1920,
	height: 1080,
};

const RES_1440: Dims = Dims {
	width: 2560,
	height: 1440,
};

fn random_frames(res: Dims, num_frames: usize) -> Vec<Vec<u8>> {
	let mut rng = rand::thread_rng();
	let mut frames = vec![vec![0; res.size()]; num_frames];
	for v in frames.iter_mut() {
		v.as_mut_slice().try_fill(&mut rng).unwrap();
	}
	frames
}

fn bench_memcpy(c: &mut Criterion) {
	let n_frames = 128;
	let frames_1080 = random_frames(RES_1080, n_frames);
	let frames_1440 = random_frames(RES_1440, n_frames);
	let mut dest_1080 = vec![0; RES_1080.size()];
	let mut dest_1440 = vec![0; RES_1440.size()];

	let mut group = c.benchmark_group("memcpy");
	group.throughput(criterion::Throughput::Elements(
		n_frames.try_into().unwrap(),
	));

	#[inline]
	fn memcpy_frames(source: &[Vec<u8>], mut dest: &mut [u8]) {
		let source = black_box(source);
		for f in source {
			dest.copy_from_slice(f.as_slice());
			dest = black_box(dest);
		}
	}

	group.bench_function("memcpy 1080", |b| {
		b.iter(|| memcpy_frames(frames_1080.as_slice(), &mut dest_1080))
	});
	group.bench_function("memcpy 1440", |b| {
		b.iter(|| memcpy_frames(frames_1440.as_slice(), &mut dest_1440))
	});

	group.finish();
}

criterion_group!(benches, bench_memcpy);
criterion_main!(benches);
