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
		match rng.gen_int_range(0, 10) {
			0 => map::TURN(map::LEFT),
			1 => map::TURN(map::RIGHT),
			_ => {
				let cd = cr.dir;
				let pos = cr.pos;
				let pos = pos.neighbor(cd);
				let in_front = cr.map.at(&pos);
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
		let mut redraw = false;
		for creatures.each |creature| {
			let old_pos = creature.pos;

			let causes_redraw = if creature.pos == player.pos {
				let redraw = creature.tick();

				if (redraw) {
					player.update_visibility();
				}
				redraw
			} else {
				let mut redraw = creature.tick();

				if (redraw) {
					let pos = creature.pos;
					if !player.sees(&old_pos) && !player.sees(&pos) {
						redraw = false;
					}
				}
				redraw
			};

			if (ui.exit) {
				return;
			}
			if causes_redraw {
				redraw = true;
			}
		}

		if redraw {
			ui.update();
		}
	}
}

fn main() {
	do sdl::start {
		sdl_main();
	}
}
