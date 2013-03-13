extern mod sdl;
mod map;

use map::MapView;

use core::str;
use core::io;
use sdl::video;
use sdl::img;
use sdl::event;

const SCREEN_WIDTH: uint = 800;
const SCREEN_HEIGHT: uint = 600;
const SCREEN_BPP: uint = 32;

fn load_or_die(file : ~str) -> ~video::Surface {
	match sdl::img::load(&Path(str::concat(&[~"data/", copy file, ~".png"]))) {
		result::Ok(image) => {
			image
		},
		result::Err(str) => {
			fail!(str);
		}
	}
}


fn sdl_main() {
	sdl::init(&[sdl::sdl::InitEverything]);
	img::init([img::InitPNG]);

	sdl::wm::set_caption("rustyhex", "rustyhex");

	let screen = match video::set_video_mode(
			SCREEN_WIDTH as int, SCREEN_HEIGHT as int, SCREEN_BPP as int,
			&[],&[video::DoubleBuf]
			) {
		result::Ok(image) => {
			image
		},
		result::Err(str) => {
			io::print(str);
			return;
		}
	};

	let tiles = load_or_die(~"tiles");

	let view = ~map::View {
		x_offset: (SCREEN_WIDTH - map::HEX_FULL_WIDTH) as int / 2,
		y_offset: (SCREEN_HEIGHT - map::HEX_FULL_HEIGHT) as int * 7 / 8
		};
	let map = map::Map::new();

	let mut player = map::Creature::new(map::Position {x: 0, y: 0}, map::N);

	player.set_map(map);

	loop {
		screen.fill(video::RGB(0,0,0));

		player.update_visibility();

		let relmap = map::RelativeMap::new(map, &player.position, player.direction);

		do player.each_in_front() | pos : &map::Position | {
			let tpos = &relmap.translate(pos);
			if player.knows(tpos) {
				let t = relmap.at(pos);
				let sprite = map::Sprite::for_tile(t, player.sees(tpos));
				view.draw_sprite(screen, tiles, pos, sprite);
			}
		}

		view.draw_sprite(screen, tiles, &map::Position {x:0,y:0}, map::Sprite::human());

		screen.flip();

		match event::wait_event() {
			event::KeyEvent(key, true , _, _) => {
				match key {
					event::EscapeKey => {
						return;
					},
					event::KKey | event::UpKey => {
						player.move_forward();
					},
					event::HKey | event::LeftKey => {
						player.turn_left();
					},
					event::LKey | event::RightKey => {
						player.turn_right();
					},
					event::JKey | event::DownKey => {
						player.move_backwards();
					},
					k => {
						io::print(fmt!("%d\n", k as int));
					}
				}
			},
			event::NoEvent => {},
			_ => {}
		}
	}
}

fn main() {
	do sdl::start {
		sdl_main();
	}
}
