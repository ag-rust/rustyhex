extern mod sdl;
mod map;

use map;
use map::*;

use core::str;
use core::io;
use core::libc::{c_char};
use sdl::sdl;
use sdl::ll;
use sdl::video;
use sdl::img;
use sdl::keyboard::*;
use sdl::event;
use sdl::video::{DoubleBuf, HWSurface, AsyncBlit};

use sdl::util::Rect;

const SCREEN_WIDTH: uint = 800;
const SCREEN_HEIGHT: uint = 600;
const SCREEN_BPP: uint = 32;

fn load_or_die(file : ~str) -> ~video::Surface {
	match img::load_img(str::concat(&[~"data/", copy file, ~".png"])) {
		result::Ok(image) => {
			image
		},
		result::Err(str) => {
			die!(str);
		}
	}
}

fn main() {
	sdl::sdl::init(&[sdl::sdl::InitEverything]);

	let screen = match video::set_video_mode(
			SCREEN_WIDTH as int, SCREEN_HEIGHT as int, SCREEN_BPP as int,
			&[],&[DoubleBuf]
			) {
		result::Ok(image) => {
			image
		},
		result::Err(str) => {
			io::print(str);
			return;
		}
	};

	let floor = load_or_die(~"floor");
	let floor_shadow = load_or_die(~"floor-shadow");
	let wall = load_or_die(~"wall");
	let wall_shadow = load_or_die(~"wall-shadow");
	let human = load_or_die(~"human");

	let view = ~View {
		x_offset: (SCREEN_WIDTH - HEX_FULL_WIDTH) as int / 2,
		y_offset: (SCREEN_HEIGHT - HEX_FULL_HEIGHT) as int * 5 / 6
		};
	let map = map::Map::new();

	let player = Creature::new(Position {x: 0, y: 0}, N);

	player.set_map(map);

	loop {
		screen.fill(0);

		player.update_visibility();

		let relmap = RelativeMap::new(map, &player.position, player.direction);

		do player.each_in_front() | pos : &map::Position | {
			let tpos = &relmap.translate(pos);
			if player.sees(tpos) {
				let t = relmap.at(pos);
				if !t.is_wall() {
					view.draw(screen, pos, floor)
				}
			}
		}
		do player.each_in_front() | pos : &map::Position | {
			let tpos = &relmap.translate(pos);
			if player.sees(tpos) {
				let t = relmap.at(pos);
				if t.is_wall() {
					view.draw(screen, pos, wall)
				}
			}
		}
		do player.each_in_front | pos : &map::Position | {
			let tpos = &relmap.translate(pos);
			if player.knows(tpos) && !player.sees(tpos) {
				let t = relmap.at(pos);
				if !t.is_wall() {
					view.draw(screen, pos, floor_shadow)
				}
			}
		}

		do player.each_in_front() | pos : &map::Position | {
			let tpos = &relmap.translate(pos);
			if player.knows(tpos) && !player.sees(tpos) {
				let t = relmap.at(pos);
				if t.is_wall() {
					view.draw(screen, pos, wall_shadow)
				}
			}
		}

		view.draw(screen, &Position {x:0,y:0}, human);

		screen.flip();

		match event::poll_event() {
			event::KeyDownEvent(ref key_event) => {
				match key_event.keycode {
					SDLKEscape => {
						return;
					},
					SDLKk | SDLKUp => {
						player.move_forward();
					},
					SDLKh | SDLKLeft => {
						player.turn_left();
					},
					SDLKl | SDLKRight => {
						player.turn_right();
					},
					SDLKj | SDLKDown => {
						player.move_backwards();
					},
					k => {
						io::print(fmt!("%d\n", k as int));
					}
				};
			},
			event::NoEvent => {},
			_ => {}
		}
	}
}
