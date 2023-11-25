//! Benchmarks capture via in-memory `Vec<u8>`s.

mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use universal_capture::{InMemory, Source};

use common::{random_frames, RES_1080, RES_1440};

fn bench_capture(c: &mut Criterion) {
	/// Number of source frames we hold in memory as inputs.
	const N_FRAMES_IN_MEMORY: usize = 128;

	let mut frames_1080 = random_frames(RES_1080, N_FRAMES_IN_MEMORY);
	let mut frames_1440 = random_frames(RES_1440, N_FRAMES_IN_MEMORY);
	let mut dest_1080 = InMemory {
		data: vec![0; RES_1080.size()],
		dims: RES_1080,
	};
	let mut dest_1440 = InMemory {
		data: vec![0; RES_1440.size()],
		dims: RES_1440,
	};

	let mut group = c.benchmark_group("capture");
	group.throughput(criterion::Throughput::Elements(
		N_FRAMES_IN_MEMORY.try_into().unwrap(),
	));

	#[inline]
	fn do_capture(source: &mut [InMemory], mut dest: &mut InMemory) {
		let source = black_box(source);
		for f in source {
			f.capture(dest).unwrap();
			dest = black_box(dest);
		}
	}

	group.bench_function("1080p", |b| {
		let source = &mut frames_1080;
		let dest = &mut dest_1080;
		b.iter(|| do_capture(source, dest))
	});
	group.bench_function("1440p", |b| {
		let source = &mut frames_1440;
		let dest = &mut dest_1440;
		b.iter(|| do_capture(source, dest))
	});

	group.finish();
}

criterion_group!(benches, bench_capture);
criterion_main!(benches);
