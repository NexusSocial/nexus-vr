use rand::Fill;
use universal_capture::{Dims, InMemory};

pub const RES_1080: Dims = Dims {
	width: 1920,
	height: 1080,
};

pub const RES_1440: Dims = Dims {
	width: 2560,
	height: 1440,
};

pub fn random_frames(res: Dims, num_frames: usize) -> Vec<InMemory> {
	let mut rng = rand::thread_rng();
	let mut frames = vec![
		InMemory {
			data: Vec::with_capacity(res.width * res.height),
			dims: res
		};
		num_frames
	];
	for v in frames.iter_mut() {
		v.data.as_mut_slice().try_fill(&mut rng).unwrap();
	}
	frames
}
