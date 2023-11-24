//! Benchmarks common memory operations like memcpy, malloc, free, etc.

mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

use common::{random_frames, RES_1080, RES_1440};

fn bench_frame_copy(c: &mut Criterion) {
	/// Number of source frames we hold in memory as inputs.
	const N_FRAMES_IN_MEMORY: usize = 128;

	let frames_1080 = random_frames(RES_1080, N_FRAMES_IN_MEMORY);
	let frames_1440 = random_frames(RES_1440, N_FRAMES_IN_MEMORY);
	let mut dest_1080 = vec![0; RES_1080.size()];
	let mut dest_1440 = vec![0; RES_1440.size()];

	let mut group = c.benchmark_group("frame copy");
	group.throughput(criterion::Throughput::Elements(
		N_FRAMES_IN_MEMORY.try_into().unwrap(),
	));

	#[inline]
	fn memcpy_frames(source: &[Vec<u8>], mut dest: &mut [u8]) {
		let source = black_box(source);
		for f in source {
			dest.copy_from_slice(f.as_slice());
			dest = black_box(dest);
		}
	}

	#[inline]
	fn clone_assign_frames(source: &[Vec<u8>], mut dest: &mut Vec<u8>) {
		let source = black_box(source);
		for f in source {
			// This allocates and copies
			let cloned_vec = f.clone();
			// This deallocates the vec that used to be in `dest`
			*dest = cloned_vec;
			dest = black_box(dest);
		}
	}

	group.bench_with_input(BenchmarkId::new("memcpy", "1080p"), &(), |b, &()| {
		let source = frames_1080.as_slice();
		let dest = dest_1080.as_mut_slice();
		b.iter(|| memcpy_frames(source, dest))
	});
	group.bench_with_input(BenchmarkId::new("memcpy", "1440p"), &(), |b, &()| {
		let source = frames_1440.as_slice();
		let dest = dest_1440.as_mut_slice();
		b.iter(|| memcpy_frames(source, dest))
	});

	group.bench_with_input(BenchmarkId::new("clone assign", "1080p"), &(), |b, &()| {
		let source = frames_1080.as_slice();
		let dest = &mut dest_1080;
		b.iter(|| clone_assign_frames(source, dest))
	});
	group.bench_with_input(BenchmarkId::new("clone assign", "1440p"), &(), |b, &()| {
		let source = frames_1440.as_slice();
		let dest = &mut dest_1440;
		b.iter(|| clone_assign_frames(source, dest))
	});

	group.finish();
}

criterion_group!(benches, bench_frame_copy);
criterion_main!(benches);
