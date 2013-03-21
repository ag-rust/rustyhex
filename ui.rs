use core::str;
use core::io;

use sdl;
use sdl::video;
use sdl::img;
use sdl::event;
use sdl::Rect;

use map;
use map::MapView;

/* replace with something more Rusty
 * in the future */
use core::libc::{c_int};
pub extern {
	fn usleep(n : c_int) -> c_int;
}

const SCREEN_WIDTH: uint = 800;
const SCREEN_HEIGHT: uint = 600;
const SCREEN_BPP: uint = 32;

const HEX_WIDTH: uint = 66;
const HEX_HEIGHT: uint = 56;
const HEX_SIDE_WIDTH: uint = 16;
const HEX_BORDER_WIDTH:  uint = 5;
const HEX_BORDER_HEIGHT: uint = 5;

const HEX_FULL_WIDTH: uint = HEX_WIDTH + 2 * HEX_BORDER_WIDTH;
const HEX_FULL_HEIGHT: uint = HEX_HEIGHT + 2 * HEX_BORDER_HEIGHT;

struct Sprite {
	x : uint,
	y : uint
}

struct View {
	x_offset : int,
	y_offset : int
}

pub struct UI {
	screen : ~video::Surface,
	tiles : ~video::Surface,
	view : ~View,
	exit : bool
}

impl map::Position {
	pure fn to_pix_x(&self) -> int {
		self.x * ((HEX_WIDTH - HEX_SIDE_WIDTH) as int) + HEX_BORDER_WIDTH as int
	}

	pure fn to_pix_y(&self) -> int {
		self.y * (HEX_HEIGHT  as int)
		- (self.x  * (HEX_HEIGHT as int)) / 2 + HEX_BORDER_HEIGHT as int
	}

	pure fn to_pix_cx(&self) -> int {
		self.to_pix_x() + (HEX_FULL_WIDTH as int) / 2
	}

	pure fn to_pix_cy(&self) -> int {
		self.to_pix_y() + (HEX_FULL_HEIGHT as int) / 2
	}

	pure fn to_rect(&self) -> Rect {
		Rect {
			x: self.to_pix_x() as i16, y: self.to_pix_y() as i16,
			w: (HEX_WIDTH + 2 * HEX_BORDER_WIDTH) as u16,
			h: (HEX_HEIGHT + 2 * HEX_BORDER_HEIGHT) as u16
		}
	}
}

impl Sprite {
	static fn for_tile(tile : map::Tile, visible : bool) -> Sprite {
		let mut spr = match tile {
				map::FLOOR => Sprite{ x: 0, y: 1 },
				map::WALL => Sprite{ x: 0, y: 2 }
			};

		if (!visible) {
			spr.x += 1;
		}
		spr
	}
	static fn for_creature(dir : map::Direction) -> Sprite {
		Sprite{ x: dir.to_uint(), y: 3 }
	}

	static fn human() -> Sprite {
		Sprite{ x: 1, y: 0 }
	}

	fn to_rect(&self) -> Rect {
		Rect {
			x: (HEX_FULL_WIDTH * self.x) as i16,
			y: (HEX_FULL_HEIGHT * self.y) as i16,
			w: HEX_FULL_WIDTH as u16,
			h: HEX_FULL_HEIGHT as u16
		}
	}
}

fn load_or_die(file : ~str) -> ~video::Surface {
	match img::load(&Path(str::concat(&[~"data/", copy file, ~".png"]))) {
		result::Ok(image) => {
			image
		},
		result::Err(str) => {
			fail!(str);
		}
	}
}

pub impl View {
	static fn new(x : int, y : int) -> View {
		View{ x_offset: x, y_offset: y }
	}

	fn draw(&self, screen: &video::Surface, pos : &map::Position, surface : &video::Surface) {
		let mut drect = pos.to_rect();
		drect.x += self.x_offset as i16;
		drect.y += self.y_offset as i16;
		if !screen.blit_rect(
				surface,
				Some(Rect {
					x: 0, y: 0,
					w: HEX_FULL_WIDTH as u16,
					h: HEX_FULL_HEIGHT as u16
				}),
				Some(drect)
		) { fail!(~"Failed blit_surface_rect") }
	}

	fn draw_sprite(&self, dsurf: &video::Surface, ssurf: &video::Surface,
		pos : &map::Position, sprite : Sprite) {
		let mut drect = pos.to_rect();
		let mut srect = sprite.to_rect();

		drect.x += self.x_offset as i16;
		drect.y += self.y_offset as i16;

		if !dsurf.blit_rect(
				ssurf,
				Some(srect),
				Some(drect)
		) { fail!(~"Failed blit_surface_rect") }
	}
}

pub impl UI {
	static fn new() -> UI {
		sdl::init(&[sdl::InitEverything]);
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
					fail!(str);
				}
			};

		let tiles = load_or_die(~"tiles");

		UI {
			screen: screen,
			exit: false,
			view: ~View {
			  x_offset: (SCREEN_WIDTH - HEX_FULL_WIDTH) as int / 2,
			  y_offset: (SCREEN_HEIGHT - HEX_FULL_HEIGHT) as int * 7 / 8
			},
			tiles: tiles
		}
	}

	fn update(&self, player : &mut map::Creature, m : &mut map::Map) {
		self.screen.fill(video::RGB(0, 0, 0));

		let mut rm = map::RelativeMap::new(m, &player.pos, player.dir);

		do player.each_in_view_rect() | pos : &map::Position | {
			let tpos = &rm.translate(pos);
			let mut base = rm.base();
			if player.knows(base, tpos) {
				let t = base.at(tpos);
				let sprite = Sprite::for_tile(t, player.sees(base, tpos));
				self.view.draw_sprite(self.screen, self.tiles, pos, sprite);

				if player.sees(base, tpos) {
					match base.creature_at(tpos) {
						Some(creature) => {
							let sprite = Sprite::for_creature(
								creature.dir.relative_to(player.dir)
							);
							self.view.draw_sprite(self.screen, self.tiles, pos, sprite);
						},
						None => {}
					};
				}
			}
		}

		self.view.draw_sprite(self.screen, self.tiles, &map::Position {x:0, y:0}, Sprite::human());

		self.screen.flip();

		unsafe {
			usleep(50000);
		}
	}

	fn get_input(&mut self) -> map::Action {
		loop {
			match event::wait_event() {
				event::KeyEvent(key, true , m, _) => {
					let ctrl = m.contains(&event::LCtrlMod);

					if ctrl {
						match key {
							event::HKey | event::LeftKey => {
								return map::MELEE_LEFT;
							},
							event::LKey | event::RightKey => {
								return map::MELEE_RIGHT;
							},
							event::KKey | event::UpKey => {
								return map::MELEE_FORWARD;
							},
							_ => {}
						}
					} else {
						match key {
							event::EscapeKey => {
								self.exit = true;
								return map::WAIT;
							},
							event::KKey | event::UpKey => {
								if (m.contains(&event::RCtrlMod)) {
									return map::MELEE_FORWARD;
								} else {
									return map::MOVE_FORWARD;
								}
							},
							event::HKey | event::LeftKey => {
								return map::TURN_LEFT;
							},
							event::LKey | event::RightKey => {
								return map::TURN_RIGHT;
							},
							event::JKey | event::DownKey => {
								return map::MOVE_BACKWARD;
							},
							event::CommaKey | event::SpaceKey => {
								return map::WAIT;
							},
							k => {
								io::print(fmt!("%d\n", k as int));
							}
						}
					}
				},
				event::NoEvent => {},
				_ => {}
			}
		}
	}
}


