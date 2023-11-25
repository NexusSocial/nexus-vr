//! Benchmarks common memory operations like memcpy, malloc, free, etc.

mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use universal_capture::InMemory;

use common::{random_frames, RES_1080, RES_1440};

fn bench_frame_copy(c: &mut Criterion) {
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

	let mut group = c.benchmark_group("frame copy");
	group.throughput(criterion::Throughput::Elements(
		N_FRAMES_IN_MEMORY.try_into().unwrap(),
	));

	#[inline]
	fn swap_frames(source: &mut [InMemory], mut dest: &mut InMemory) {
		let source = black_box(source);
		for f in source {
			std::mem::swap(dest, f);
			dest = black_box(dest);
		}
	}

	#[inline]
	fn memcpy_frames(source: &[InMemory], mut dest: &mut InMemory) {
		let source = black_box(source);
		for f in source {
			dest.data.copy_from_slice(f.data.as_slice());
			dest = black_box(dest);
		}
	}

	#[inline]
	fn clone_assign_frames(source: &[InMemory], mut dest: &mut InMemory) {
		let source = black_box(source);
		for f in source {
			// This allocates and copies
			let cloned_vec = f.data.clone();
			// This deallocates the vec that used to be in `dest`
			dest.data = cloned_vec;
			dest = black_box(dest);
		}
	}

	group.bench_with_input(BenchmarkId::new("memcpy", "1080p"), &(), |b, &()| {
		let source = frames_1080.as_slice();
		let dest = &mut dest_1080;
		b.iter(|| memcpy_frames(source, dest))
	});
	group.bench_with_input(BenchmarkId::new("memcpy", "1440p"), &(), |b, &()| {
		let source = frames_1440.as_slice();
		let dest = &mut dest_1440;
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

	group.bench_with_input(BenchmarkId::new("swap", "1080p"), &(), |b, &()| {
		let source = &mut frames_1080;
		let dest = &mut dest_1080;
		b.iter(|| swap_frames(source, dest))
	});
	group.bench_with_input(BenchmarkId::new("swap", "1440p"), &(), |b, &()| {
		let source = &mut frames_1440;
		let dest = &mut dest_1440;
		b.iter(|| swap_frames(source, dest))
	});

	group.finish();
}

criterion_group!(benches, bench_frame_copy);
criterion_main!(benches);
