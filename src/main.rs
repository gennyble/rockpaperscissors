mod revolvingrandom;

use std::{
	mem::MaybeUninit,
	time::{Duration, Instant},
};

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

		if world.done {
			break;
		}

		world.draw();
	}
}

struct World {
	smitten: Smitten,
	things: Vec<Entity>,
	earlier: Option<Instant>,
	revolve: RevolvingRandom,
	/// homoogeneous
	homo: bool,
	done: bool,
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
		.take(100)
		.collect();

		let mut things = vec![];

		for (_id, (position, direction)) in positions_directions.into_iter().enumerate() {
			let kind_idx = thread_rng().gen_range(0..3);

			let kind = match kind_idx {
				0 => Kind::Rock,
				1 => Kind::Paper,
				2 => Kind::Scissors,
				_ => unreachable!(),
			};

			things.push(Entity {
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
			homo: false,
			done: false,
		}
	}

	pub fn tick(&mut self) {
		let _events = self.smitten.events();

		if self.done {
			return;
		}

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

		self.things.iter_mut().for_each(|e| {
			e.position += e.direction * Entity::SPEED * delta.as_secs_f32();
		});

		// do some jiggle 🥺
		self.things.iter_mut().for_each(|e| {
			let jitter = 0.05;
			e.position += Vec2::new(
				self.revolve.range(-jitter, jitter),
				self.revolve.range(-jitter, jitter),
			)
		});

		if !self.homo {
			self.collide_walls();
			self.tick_entities(delta);
		} else {
			self.kill_offscreen_things();

			if self.things.len() == 0 {
				self.done = true;
			}
		}
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

	fn tick_entities(&mut self, delta: Duration) {
		let mut did_collide = false;
		let mut last_kind = Kind::Paper;

		// We can't mutate two things at once, so we do this stuff
		let mut seen = vec![];
		loop {
			let mut thing = match self.things.pop() {
				Some(thing) => thing,
				None => break,
			};

			// Entity-Entity collision
			for th in self.things.iter_mut() {
				if Self::collide_entities(&mut thing, th) {
					did_collide = true;
				}
			}

			// Chasing
			let closest = Self::closest_of_kind(
				&thing,
				&mut self.things.iter().chain(seen.iter()),
				thing.kind.beats(),
			);

			if let Some(close) = closest {
				let direction = (thing.position - close.position).normalize();
				thing.direction = direction * -1.0;
			}

			last_kind = thing.kind;
			seen.push(thing);
		}
		self.things = seen;

		// Only do the check if there was a collision, which is the only way kinds can change
		if did_collide {
			let homo = self.things.iter().all(|e| e.kind == last_kind);

			if homo && !self.homo {
				self.homo = homo;
				self.things
					.iter_mut()
					.for_each(|e| e.direction = e.direction * -1.0);
			}
		}
	}

	fn collide_entities(a: &mut Entity, b: &mut Entity) -> bool {
		if a.collides_with(b) {
			match a.kind {
				Kind::Rock => match b.kind {
					Kind::Rock => (),
					Kind::Paper => a.kind = Kind::Paper,
					Kind::Scissors => b.kind = Kind::Rock,
				},
				Kind::Paper => match b.kind {
					Kind::Rock => b.kind = Kind::Paper,
					Kind::Paper => (),
					Kind::Scissors => a.kind = Kind::Scissors,
				},
				Kind::Scissors => match b.kind {
					Kind::Rock => a.kind = Kind::Rock,
					Kind::Paper => b.kind = Kind::Scissors,
					Kind::Scissors => (),
				},
			}

			return true;
		}

		false
	}

	fn closest_of_kind<'a, I>(us: &Entity, iter: &mut I, kind: Kind) -> Option<&'a Entity>
	where
		I: Iterator<Item = &'a Entity>,
	{
		let mut entity = None;
		let mut distance = f32::MAX;

		iter.for_each(|ent| {
			if ent.kind == kind {
				let dist = us.position.distance_with(ent.position);

				if dist < distance {
					distance = dist;
					entity = Some(ent);
				}
			}
		});

		entity
	}

	fn kill_offscreen_things(&mut self) {
		let murs = (self.smitten.screen_murs() + Self::ENTITY_DIM) / 2.0;

		let count_before = self.things.len();
		self.things.retain(|e| {
			let pos = e.position;

			pos.x < murs.x && pos.x > -murs.x && pos.y < murs.y && pos.y > -murs.y
		});

		if self.things.len() != count_before {
			let count = self.things.len();
			let difference = count_before - count;
			let maybeplural = if difference == 1 { "thing" } else { "things" };

			println!("Killed {difference} {maybeplural} for being off screen, {count} remain");
		}
	}
}

struct Entity {
	kind: Kind,
	position: Vec2,
	/// Movement direction. Where is it going?
	direction: Vec2,
}

impl Entity {
	const SPEED: f32 = 1.5;

	pub fn collides_with(&self, other: &Entity) -> bool {
		//gen- Why is this not ENTITY_DIM.x * 2?
		self.position.distance_with(other.position) < World::ENTITY_DIM.x
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Kind {
	Rock,
	Paper,
	Scissors,
}

impl Kind {
	pub fn beats(&self) -> Self {
		match self {
			Kind::Rock => Kind::Scissors,
			Kind::Paper => Kind::Rock,
			Kind::Scissors => Kind::Paper,
		}
	}

	#[allow(dead_code)]
	pub fn beaten_by(&self) -> Self {
		match self {
			Kind::Rock => Kind::Paper,
			Kind::Paper => Kind::Scissors,
			Kind::Scissors => Kind::Rock,
		}
	}
}
