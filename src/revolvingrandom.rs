use rand::{thread_rng, Rng};

pub struct RevolvingRandom {
	data: [f32; Self::SIZE],
	idx: usize,
}

impl RevolvingRandom {
	const SIZE: usize = 1024;

	pub fn new() -> Self {
		let mut data = [0.0f32; Self::SIZE];
		thread_rng().fill(&mut data);

		Self { data, idx: 0 }
	}

	pub fn rand(&mut self) -> f32 {
		let ret = self.data[self.idx];

		if self.idx == self.data.len() - 1 {
			self.idx = 0;
		} else {
			self.idx += 1;
		}

		ret
	}

	//FIXME: gen- this doesn't work if max is negative, does it?
	pub fn range(&mut self, min: f32, max: f32) -> f32 {
		self.rand().abs() * (min.abs() + max.abs()) + min
	}
}
