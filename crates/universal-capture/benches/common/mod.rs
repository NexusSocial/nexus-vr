use rand::Fill;

pub struct Dims {
	width: usize,
	height: usize,
}

impl Dims {
	pub const fn size(&self) -> usize {
		self.width * self.height
	}
}

pub const RES_1080: Dims = Dims {
	width: 1920,
	height: 1080,
};

pub const RES_1440: Dims = Dims {
	width: 2560,
	height: 1440,
};

pub fn random_frames(res: Dims, num_frames: usize) -> Vec<Vec<u8>> {
	let mut rng = rand::thread_rng();
	let mut frames = vec![vec![0; res.size()]; num_frames];
	for v in frames.iter_mut() {
		v.as_mut_slice().try_fill(&mut rng).unwrap();
	}
	frames
}
