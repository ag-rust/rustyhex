extern mod sdl;

pub mod map;
pub mod ui;

use map::MapView;
use core::rand::RngUtil;

pub struct PlayerController {
	ui : @mut ui::UI
}

pub struct MonsterController(());

impl MonsterController {
	fn new() -> MonsterController {
		 MonsterController(())
	}
}

impl map::MoveController for MonsterController {
	fn get_move(&mut self, cr : @mut map::Creature) -> map::Action {
		let rng = rand::Rng();

		for [map::FORWARD, map::LEFT, map::RIGHT].each |dir| {
			let pos = cr.pos;
			let cd = cr.dir;
			let pos = pos.neighbor(cd.turn(*dir));
			match cr.map.creature_at(&pos) {
				None => {}
				Some(_) => {
					return map::MELEE(*dir);
				}
			}
		};

		match rng.gen_int_range(0, 10) {
			0 => map::TURN(map::LEFT),
			1 => map::TURN(map::RIGHT),
			_ => {
				let cd = cr.dir;
				let pos = cr.pos;
				let front = pos.neighbor(cd);
				let in_front = cr.map.at(&front);
				if in_front.is_passable() {
					map::MOVE(map::FORWARD)
				} else {
					map::TURN(map::LEFT)
				}
			}
		}
	}
}

impl PlayerController {
	fn new(ui : @mut ui::UI) -> PlayerController {
		PlayerController {ui: ui}
	}
}

impl map::MoveController for PlayerController {
	fn get_move(&mut self, _ : @mut map::Creature) -> map::Action {
		self.ui.get_input()
	}
}


fn sdl_main() {
	let mut ui = @mut ui::UI::new();

	let map = @mut map::Map::new();


	let mut creatures = vec::from_fn(30, |_| {
			map.spawn_random_creature(@MonsterController::new())
		}
	);

	let player = map.spawn_random_creature(@PlayerController::new(ui));
	creatures.push(player);

	player.update_visibility();
	ui.set_player(player);
	ui.update();

	loop {
		for creatures.each |creature| {
			if (!creature.alive()) {
				loop;
			}
			if creature.pos == player.pos {
				let redraw = creature.tick();

				if (redraw) {
					player.update_visibility();
				}
			} else {
				creature.tick();
			};

			if (ui.exit) {
				return;
			}
		}

		if (!player.alive()) {
			ui.check_exit_input();
		}

		ui.update();
	}
}

fn main() {
	do sdl::start {
		sdl_main();
	}
}
