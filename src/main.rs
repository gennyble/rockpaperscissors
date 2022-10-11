use std::time::Instant;

use rand::{thread_rng, Rng};
use smitten::{self, Color, Smitten, Vec2};

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
	rock: Vec<Entity>,
	paper: Vec<Entity>,
	scissors: Vec<Entity>,
	earlier: Option<Instant>,
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

		let mut rock = vec![];
		let mut paper = vec![];
		let mut scissors = vec![];

		for (id, (position, direction)) in positions_directions.into_iter().enumerate() {
			let kind_idx = thread_rng().gen_range(0..3);

			match kind_idx {
				0 => rock.push(Entity {
					id,
					kind: Kind::Rock,
					position,
					direction,
				}),
				1 => paper.push(Entity {
					id,
					kind: Kind::Paper,
					position,
					direction,
				}),
				2 => scissors.push(Entity {
					id,
					kind: Kind::Scissors,
					position,
					direction,
				}),
				_ => unreachable!(),
			};
		}

		Self {
			smitten,
			rock,
			paper,
			scissors,
			earlier: None,
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

		self.foreach_entities_mut(|e| {
			e.position += e.direction * Entity::SPEED * delta.as_secs_f32();
		});

		self.collide_walls();
	}

	const ENTITY_DIM: Vec2 = Vec2 { x: 1.0, y: 1.0 };

	pub fn draw(&self) {
		self.smitten.clear();

		// I really didn't need to make a macro here. Could've made a weird little function sintead, but, too late
		macro_rules! draw_vec {
			($iterator:expr, $draw:ident) => {
				$iterator
					.iter()
					.for_each(|ent| self.smitten.rect(ent.position, Self::ENTITY_DIM, $draw));
			};
		}

		draw_vec!(self.rock, ROCK_COLOR);
		draw_vec!(self.paper, PAPER_COLOR);
		draw_vec!(self.scissors, SCISSORS_COLOR);

		self.smitten.swap();
	}

	fn random_position(smit: &Smitten) -> Vec2 {
		let murs = (smit.screen_murs() / 2) - (Self::ENTITY_DIM / 2);

		Vec2::new(
			thread_rng().gen_range(-murs.x..murs.x),
			thread_rng().gen_range(-murs.y..murs.y),
		)
	}

	fn foreach_entities_mut<F>(&mut self, f: F)
	where
		F: FnMut(&mut Entity),
	{
		self.rock
			.iter_mut()
			.chain(self.paper.iter_mut())
			.chain(self.scissors.iter_mut())
			.for_each(f);
	}

	fn collide_walls(&mut self) {
		let walls = self.smitten.screen_murs() / 2;
		let entdim = Self::ENTITY_DIM.x / 2.0;

		self.foreach_entities_mut(|ent| {
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
	const SPEED: f32 = 1.0;
}

enum Kind {
	Rock,
	Paper,
	Scissors,
}
