extern mod sdl;

pub mod map;
pub mod ui;

fn sdl_main() {
	let ui = ui::UI::new();

	let map = map::Map::new();

	let mut player = map::Creature::new(map::Position {x: 0, y: 0}, map::N);

	player.set_map(map);

	loop {

		player.update_visibility();

		ui.update(player, map);

		match ui.get_input() {
			ui::MOVE_FORWARD => player.move_forward(),
			ui::MOVE_BACKWARD => player.move_backward(),
			ui::TURN_LEFT => player.turn_left(),
			ui::TURN_RIGHT => player.turn_right(),
			ui::EXIT => return
		};
	}
}

fn main() {
	do sdl::start {
		sdl_main();
	}
}
