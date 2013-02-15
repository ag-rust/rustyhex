pub use sdl::util::Rect;

use core::uint::range;
use core::cast;
use core::cmp::Eq;
use core::vec;
use core::vec::{filtered};
use core::num::float;
use core::int::*;

#[deriving_eq]
enum Direction {
	N = 0,
	NE,
	SE,
	S,
	SW,
	NW
}

pub struct Position {
	x : int,
	y : int
}

pub struct Creature {
	position :Position,
	direction : Direction,
	map: Option<@mut Map>,
	map_width: uint,
	map_height: uint,
	map_visible : Option<~[ ~[ bool ] ]>,
	map_known : Option<~[ ~[ bool ] ]>
}

pub enum Tile {
	WALL,
	FLOOR
}

const MAP_WIDTH : uint = 16;
const MAP_HEIGHT : uint = 16;

pub struct Map {
	map : ~[ ~[ Tile ] ],
	width: uint,
	height : uint
}

pub const HEX_BASE_HEIGHT: uint = 35;
pub const HEX_BASE_WIDTH: uint = 20;
pub const HEX_SIDE_WIDTH: uint = 10;
pub const HEX_BORDER_HEIGHT: uint = 2;
pub const HEX_BORDER_WIDTH: uint = 2;

pub const HEX_FULL_WIDTH: uint = HEX_BASE_WIDTH + 2 * HEX_SIDE_WIDTH + HEX_BORDER_WIDTH;
pub const HEX_FULL_HEIGHT: uint = HEX_BASE_HEIGHT + HEX_BORDER_HEIGHT;

pub impl Direction {

	pure fn right(&self) -> Direction {
		unsafe {
			cast::reinterpret_cast(&mymod((*self as int + 1), 6))
		}
	}
	pure fn left(&self) -> Direction {
		unsafe {
			cast::reinterpret_cast(&mymod((*self as int + 5), 6))
		}
	}
	pure fn turn(&self, i : int) -> Direction {
		unsafe {
			cast::reinterpret_cast(&mymod((*self as int + i), 6))
		}
	}
	pure fn opposite(&self) -> Direction {
		unsafe {
			cast::reinterpret_cast(&mymod((*self as int + 3), 6))
		}
	}

	pure fn to_uint(&self) -> uint {
		unsafe {
			cast::reinterpret_cast(&(*self as int))
		}
	}
}

pub impl Position : Eq {

	pure fn eq(&self, p : &Position) -> bool {
		self.x == p.x && self.y == p.y
	}

	pure fn ne(&self, p : &Position) -> bool {
		!(self == p)
	}
}

pub impl Position {

	pure fn to_pix_x(&self) -> uint {
		self.x as uint * (HEX_BASE_WIDTH + HEX_SIDE_WIDTH)
	}

	pure fn to_pix_y(&self) -> uint {
		self.y as uint * HEX_BASE_HEIGHT +
			if mymod(self.x, 2) != 0 { HEX_BASE_HEIGHT / 2 } else { 0 }
	}

	pure fn to_pix_cx(&self) -> uint {
		self.to_pix_x() + HEX_FULL_WIDTH / 2
	}

	pure fn to_pix_cy(&self) -> uint {
		self.to_pix_y() + HEX_FULL_HEIGHT / 2
	}

	pure fn to_rect(&self) -> Rect {
		Rect {
			x: self.to_pix_x() as i16, y: self.to_pix_y() as i16,
			w: (HEX_BASE_WIDTH + 2 * HEX_SIDE_WIDTH + HEX_BORDER_WIDTH) as u16,
			h: (HEX_BASE_HEIGHT + HEX_BORDER_HEIGHT) as u16
		}
	}

	pure fn is_neighbor(&self, position : Position) -> bool {
		let rx = self.x - position.x;
		let ry = self.y - position.y;
		if mymod(self.x, 2) != 0 {
			match (rx, ry) {
				(0, -1)  => true,
				(0, 1)   => true,
				(-1, 0)  => true,
				(1, 0)   => true,
				(-1, -1) => true,
				(1, -1)  => true,
				_ => {
					false
				}
			}
		} else {
			match (rx, ry) {
				(0, -1)  => true,
				(0, 1)   => true,
				(-1, 0)  => true,
				(1, 0)   => true,
				(-1, 1)  => true,
				(1, 1)   => true,
				_ => {
						false
				}
			}
		}
	}

	pure fn neighbor(&self, direction : Direction) -> Position {
		match direction {
			N => Position { x: self.x, y: self.y - 1 },
			S => Position { x: self.x, y: self.y + 1 },
			NE => Position { x: self.x + 1 , y: self.y - mymod((self.x + 1), 2)},
			SE => Position { x: self.x + 1, y: self.y + mymod(self.x, 2) },
			NW => Position { x: self.x - 1 , y: self.y - mymod((self.x + 1), 2)},
			SW => Position { x: self.x - 1, y: self.y + mymod(self.x, 2) }
		}
	}
}

/*
pure fn mymod(x :int, m : int) -> int {
	let r = x%m;
	if r < 0 { r+m } else { r }
}*/

pure fn mymod(x :int, m : int) -> int {
	((x % m) + m) % m
}

pub impl Creature {
	static fn new(position : Position, direction : Direction) -> ~mut Creature {
		~mut Creature {
			position: position, direction: direction, map: None,
			map_visible: None, map_known: None,
			map_width: 0, map_height: 0
		}
	}

	fn set_map(&mut self, map : @mut Map) {
		self.map = Some(map);
		self.map_width= map.width;
		self.map_height = map.width;

		self.map_visible = Some(vec::from_elem(map.width, vec::from_elem(map.height, false)));
		self.map_known = Some(vec::from_elem(map.width, vec::from_elem(map.height, false)));
	}

	fn with_map(&mut self, f : &fn (map : &mut Map)) {
		match self.map {
			Some(map) => f(map),
			None => {}
		}
	}

	fn do_view(&mut self, pos: &Position, dir : Direction,
		pdir : Option<Direction>, depth: uint) {
		if (depth == 0) {
			return;
		}

		self.mark_visible(pos);
		self.mark_known(pos);

		let neighbors = match pdir {
			Some(pdir) => {
				if pdir == dir {
					~[dir]
				} else {
					~[dir, pdir]
				}
			},
			None => {
				~[dir, dir.left(), dir.right()]
			}
		};

		do self.with_map |map| {
			if map.at(pos).can_see_through() {
				for neighbors.each |&d| {
					let n = pos.neighbor(d);
					self.do_view(&n, d, Some(dir), depth - 1);
				}
			}
		}
	}

	fn update_visibility(&mut self) {

		do self.with_map |map| {
			self.map_visible = Some(vec::from_elem(map.width, vec::from_elem(map.height, false)));
		}

		self.do_view(&self.position, self.direction, None, 15);
	}

	fn turn_right(&mut self) {
		 self.direction = self.direction.right();
	}

	fn turn_left(&mut self) -> () {
		self.direction = self.direction.left();
	}

	fn move_forward(&mut self) {
		do self.with_map() | map | {
			let new_position = self.position.neighbor(self.direction);
			if (map.at(&new_position).is_passable()) {
				self.position = new_position;
			}
		}
	}

	fn move_backwards(&mut self) {
		do self.with_map() | map | {
			let new_position = self.position.neighbor(self.direction.opposite());
			if (map.at(&new_position).is_passable()) {
				self.position = new_position;
			}
		}
	}

	pure fn wrap_position(&self, pos : &Position) -> Position {
		Position {
			x: mymod(pos.x, self.map_width as int),
			y: mymod(pos.y, self.map_height as int)
		}
	}

	fn mark_visible(&mut self, pos : &Position) {
		let p = self.wrap_position(pos);

		match self.map_visible {
			Some(ref mut visible) => {
				visible[p.x][p.y] = true
			},
			None => {}
		}
	}


	fn mark_known(&mut self, pos : &Position) {
		let p = self.wrap_position(pos);

		match self.map_known {
			Some(ref mut known) => {
				known[p.x][p.y] = true
			},
			None => {}
		}
	}

	pure fn sees(&self, pos: &Position) -> bool {
		let p = self.wrap_position(pos);

		match self.map_visible {
			Some(ref visible) => {
				visible[p.x][p.y]
			},
			None => false
		}
	}

	pure fn knowns(&self, pos: &Position) -> bool {
		let p = self.wrap_position(pos);

		match self.map_known {
			Some(ref knows) => {
				knows[p.x][p.y]
			},
			None => false
		}
	}

	pure fn position(&self) -> Position {
		self.position
	}
}

pub impl Tile {

	fn is_wall(&self) -> bool {
		match *self {
			WALL => true,
			_ => false
		}
	}

	fn can_see_through(&self) -> bool {
		match *self {
			WALL => false,
			_ => true
		}
	}

	fn is_passable(&self) -> bool {
		match *self {
			WALL => false,
			_ => true
		}
	}
}

pub impl Map {
	static fn new() -> @mut Map {
		let rng = rand::Rng();

		let map = vec::from_fn(MAP_WIDTH, |_| {
			vec::from_fn(MAP_HEIGHT, |_| {
				if (rng.gen_int_range(0, 3) == 0) {
					WALL
				} else {
					FLOOR
				}
			})
		});
		@mut Map {
			map: map, width: MAP_WIDTH, height: MAP_HEIGHT
		}
	}

	pure fn wrap_position(&self, pos : &Position) -> Position {
		Position {
			x: mymod(pos.x, self.width as int),
			y: mymod(pos.y, self.height as int)
		}
	}

	fn at(&self, position : &Position) -> Tile {
		let p = self.wrap_position(position);
		self.map[p.x][p.y]
	}

	fn each(&mut self, f : &fn(position : Position, &mut Tile)) {
		for range(0, self.width as int) |x| {
			for range(0, self.height as int) |y| {
				f(Position {x: x as int, y: y as int}, &mut self.map[x][y]);
			}
		}
	}
}

