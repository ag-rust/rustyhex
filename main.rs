extern mod sdl;

pub mod map;
pub mod ui;

use map::MapView;

pub struct PlayerController {
	ui : &'self mut ui::UI
}

pub struct MonsterController(());

impl MonsterController {
	static fn new() -> MonsterController {
		 MonsterController(())
	}
}

impl map::MoveController for MonsterController {
	fn get_move(&mut self, map : &mut map::Map, cr : @mut map::Creature) -> map::Action {
		let rng = rand::Rng();
		match rng.gen_int_range(0, 10) {
			0 => map::TURN_LEFT,
			1 => map::TURN_RIGHT,
			_ => {
				let in_front = map.at(&cr.pos.neighbor(cr.dir));
				if in_front.is_passable() {
					map::MOVE_FORWARD
				} else {
					map::TURN_LEFT
				}
			}
		}
	}
}

impl<'self> PlayerController<'self> {
	static fn new(ui : &'r mut ui::UI) -> PlayerController/&r {
		PlayerController {ui: ui}
	}
}

impl map::MoveController for PlayerController<'self> {
	fn get_move(&mut self, _ : &mut map::Map, _ : @mut map::Creature) -> map::Action {
		self.ui.get_input()
	}
}

fn sdl_main() {
	let mut ui = ~ui::UI::new();

	let mut map = ~map::Map::new();
	let mut monster_ai = ~MonsterController::new();

	let mut player = map.spawn_creature(&map::Position{x: 0, y: 0}, map::N);

	let mut player = match player {
		Some(p) => p,
		None => fail!(~"Couldn't spawn player!")
	};
	let mut creatures = vec::from_fn(30, |_| {
			map.spawn_random_creature()
		}
	);

	creatures.push(player);
	player.update_visibility(map);
	ui.update(player, map);

	loop {
		for creatures.each |creature| {
			let old_pos = creature.pos;

			let redraw = if creature.pos == player.pos {
				let mut player_ai = ~PlayerController::new(ui);
				let redraw = creature.tick(map, player_ai);

				if (redraw) {
					player.update_visibility(map);
				}
				redraw
			} else {
				let mut redraw = creature.tick(map, monster_ai);

				if (redraw) {
					if !player.sees(map, &old_pos) && !player.sees(map, &creature.pos) {
						redraw = false;
					}
				}
				redraw
			};

			if (ui.exit) {
				return;
			}

			if redraw {
				ui.update(player, map);
			}
		}
	}
}

fn main() {
	do sdl::start {
		sdl_main();
	}
}
