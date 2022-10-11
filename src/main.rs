mod revolvingrandom;

use std::time::Instant;

use rand::{thread_rng, Rng};
use smitten::{self, Color, Smitten, Vec2};

use revolvingrandom::RevolvingRandom;

const ROCK_COLOR: Color = Color::grey(0.5);
const PAPER_COLOR: Color = Color::rgb(0.7, 0.7, 0.4);
const SCISSORS_COLOR: Color = Color::rgb(0.8, 0.3, 0.3);

fn main() {
	let mut world = World::new();

	loop {
		world.tick();
		world.draw();
	}
}

struct World {
	smitten: Smitten,
	things: Vec<Entity>,
	earlier: Option<Instant>,
	revolve: RevolvingRandom,
}

impl World {
	pub fn new() -> Self {
		let smitten = Smitten::new((1280, 960), "Rock Paper Scissors", 24);

		let positions_directions: Vec<(Vec2, Vec2)> = std::iter::repeat_with(|| {
			(
				Self::random_position(&smitten),
				Self::random_position(&smitten).normalize(),
			)
		})
		.take(50)
		.collect();

		let mut things = vec![];

		for (id, (position, direction)) in positions_directions.into_iter().enumerate() {
			let kind_idx = thread_rng().gen_range(0..3);

			let kind = match kind_idx {
				0 => Kind::Rock,
				1 => Kind::Paper,
				2 => Kind::Scissors,
				_ => unreachable!(),
			};

			things.push(Entity {
				id,
				kind,
				position,
				direction,
			})
		}

		Self {
			smitten,
			things,
			earlier: None,
			revolve: RevolvingRandom::new(),
		}
	}

	pub fn tick(&mut self) {
		let _events = self.smitten.events();

		// We can't
		let delta = match self.earlier {
			None => {
				self.earlier = Some(Instant::now());
				return;
			}
			Some(earlier) => {
				let now = Instant::now();
				let delta = now.duration_since(earlier);
				self.earlier = Some(now);
				delta
			}
		};

		// Do some jiggle 🥺
		// We can't use foreach_entities_mut here because I want self.revovle
		// and I need a &mut on that, too, and the compiler doesn't know that
		// the former only uses rock, paper, and scissors. It's protecting us
		// here. From the crime of two mutable borrows
		self.things.iter_mut().for_each(|e| {
			let jitter = 0.05;
			e.position += Vec2::new(
				self.revolve.range(-jitter, jitter),
				self.revolve.range(-jitter, jitter),
			)
		});

		self.things.iter_mut().for_each(|e| {
			e.position += e.direction * Entity::SPEED * delta.as_secs_f32();
		});

		self.collide_walls();
		self.collide_entities();
	}

	const ENTITY_DIM: Vec2 = Vec2 { x: 1.0, y: 1.0 };

	pub fn draw(&self) {
		self.smitten.clear();

		self.things.iter().for_each(|ent| {
			let draw = match ent.kind {
				Kind::Rock => ROCK_COLOR,
				Kind::Paper => PAPER_COLOR,
				Kind::Scissors => SCISSORS_COLOR,
			};

			self.smitten.rect(ent.position, Self::ENTITY_DIM, draw)
		});

		self.smitten.swap();
	}

	fn random_position(smit: &Smitten) -> Vec2 {
		let murs = (smit.screen_murs() / 2) - (Self::ENTITY_DIM / 2);

		Vec2::new(
			thread_rng().gen_range(-murs.x..murs.x),
			thread_rng().gen_range(-murs.y..murs.y),
		)
	}

	fn collide_walls(&mut self) {
		let walls = self.smitten.screen_murs() / 2;
		let entdim = Self::ENTITY_DIM.x / 2.0;

		self.things.iter_mut().for_each(|ent| {
			if ent.position.x + entdim > walls.x {
				ent.position.x = walls.x - entdim;
				ent.direction.x *= -1.0;
			} else if ent.position.x - entdim < -walls.x {
				ent.position.x = -walls.x + entdim;
				ent.direction.x *= -1.0;
			}

			if ent.position.y + entdim > walls.y {
				ent.position.y = walls.y - entdim;
				ent.direction.y *= -1.0;
			} else if ent.position.y - entdim < -walls.y {
				ent.position.y = -walls.y + entdim;
				ent.direction.y *= -1.0;
			}
		})
	}

	fn collide_entities(&mut self) {
		let mut seen = vec![];

		loop {
			let mut thing = match self.things.pop() {
				Some(thing) => thing,
				None => break,
			};

			for th in self.things.iter_mut() {
				if thing.collides_with(th) {
					match thing.kind {
						Kind::Rock => match th.kind {
							Kind::Rock => (),
							Kind::Paper => thing.kind = Kind::Paper,
							Kind::Scissors => th.kind = Kind::Rock,
						},
						Kind::Paper => match th.kind {
							Kind::Rock => th.kind = Kind::Paper,
							Kind::Paper => (),
							Kind::Scissors => thing.kind = Kind::Scissors,
						},
						Kind::Scissors => match th.kind {
							Kind::Rock => thing.kind = Kind::Rock,
							Kind::Paper => th.kind = Kind::Scissors,
							Kind::Scissors => (),
						},
					}
				}
			}

			seen.push(thing);
		}

		self.things = seen;
	}
}

type EntityId = usize;

struct Entity {
	id: EntityId,
	kind: Kind,
	position: Vec2,
	/// Movement direction. Where is it going?
	direction: Vec2,
}

impl Entity {
	const SPEED: f32 = 2.0;

	pub fn collides_with(&self, other: &Entity) -> bool {
		//gen- Why is this not ENTITY_DIM.x * 2?
		self.position.distance_with(other.position) < World::ENTITY_DIM.x
	}
}

enum Kind {
	Rock,
	Paper,
	Scissors,
}
