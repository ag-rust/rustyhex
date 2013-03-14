use core::int::*;
use core::cast;
use core::cmp::Eq;
use core::ops::{Add, Sub};
use core::vec;

#[deriving_eq]
pub enum Direction {
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
	FLOOR,
	WALL
}

const MAP_WIDTH : uint = 32;
const MAP_HEIGHT : uint = 32;

pub struct Map {
	map : ~[ ~[ Tile ] ],
	creatures : ~[ ~[ Option<@Creature> ] ],
	width: uint,
	height : uint
}

pub trait MapView {
	fn at(&self, pos: &Position) -> Tile;
	fn creature_at(&self, pos: &Position) -> Option<@Creature>;
	fn translate(&self, pos : &Position) -> Position;
}

/**
 * View of the map with rotation (dir) and offset (pos)
 */
pub struct RelativeMap {
	map : @mut Map,
	pos : Position,
	dir : Direction
}

pub impl Direction {
	pure fn right(&self) -> Direction {
		unsafe {
			cast::reinterpret_cast(&modulo((*self as int + 1), 6))
		}
	}

	pure fn left(&self) -> Direction {
		unsafe {
			cast::reinterpret_cast(&modulo((*self as int + 5), 6))
		}
	}

	pure fn turn(&self, i : int) -> Direction {
		unsafe {
			cast::reinterpret_cast(&modulo((*self as int + i), 6))
		}
	}

	pure fn opposite(&self) -> Direction {
		unsafe {
			cast::reinterpret_cast(&modulo((*self as int + 3), 6))
		}
	}

	pure fn relative_to(&self, dir : Direction) -> Direction {
		unsafe {
			cast::reinterpret_cast(&modulo(((*self as int)  - (dir as int)), 6))
		}
	}

	pure fn to_uint(&self) -> uint {
		unsafe {
			cast::reinterpret_cast(&(*self as int))
		}
	}
}

impl Eq for Position {

	pure fn eq(&self, p : &Position) -> bool {
		self.x == p.x && self.y == p.y
	}

	pure fn ne(&self, p : &Position) -> bool {
		!(self == p)
	}
}

impl Add<Position, Position> for Position {
	pure fn add(&self, pos : &Position) -> Position {
		Position {x: self.x + pos.x, y: self.y + pos.y }
	}
}

impl Sub<Position, Position> for Position {
	pure fn sub(&self, pos : &Position) -> Position {
		Position {x: self.x - pos.x, y: self.y - pos.y }
	}
}

pub impl Position {

	pure fn relative_to(&self, pos : &Position) -> ~Position {
		~Position{ x: self.x - pos.x, y: self.y - pos.y}
	}

	/* Iterate over every neighbor */
	fn each_around(&self, up : int, down : int, left : int, right : int, f : &fn(position : &Position)) {
		for range(self.y - up, self.y + down + 1) |vy| {
			for range(self.x - left, self.x + right + 1) |vx| {
				let x = vx;
				let y = vy + ((vx - self.x) >> 1);
				f (&Position {x: x, y: y});
			}
		}
	}

	pure fn is_neighbor(&self, position : Position) -> bool {
		let rx = self.x - position.x;
		let ry = self.y - position.y;
		match (rx, ry) {
			(0, -1)  => true,
			(0, 1)   => true,
			(-1, 0)  => true,
			(1, 0)   => true,
			(-1, -1) => true,
			(1, 1)  => true,
			_ => {
				false
			}
		}
	}

	pure fn neighbor(&self, direction : Direction) -> Position {
		match direction {
			N => Position { x: self.x, y: self.y - 1 },
			S => Position { x: self.x, y: self.y + 1 },
			SW => Position { x: self.x - 1, y: self.y },
			NE => Position { x: self.x + 1, y: self.y },
			NW => Position { x: self.x - 1, y: self.y - 1 },
			SE => Position { x: self.x + 1, y: self.y + 1 }
		}
	}
}

pure fn modulo(x :int, m : int) -> int {
	let r = x%m;
	if r < 0 { r+m } else { r }
}

/*
 * Creature helper to deal with
 * optional map.
 */
macro_rules! if_map(
		(|$v:ident| $inexp:expr ) => (
		{
			let maybe_map = self.map;
			match (maybe_map) {
				Some($v) => $inexp,
				_ => {}
			};
		}
		)
	)

const PLAYER_VIEW: int = 10;
pub impl Creature {
	static fn new(position : Position, direction : Direction) -> Creature {
		Creature {
			position: position, direction: direction, map: None,
			map_visible: None, map_known: None,
			map_width: 0, map_height: 0
		}
	}

	fn set_map(&mut self, map : @mut Map) {
		self.map = Some(map);
		self.map_width = map.width;
		self.map_height = map.height;

		self.map_visible = Some(vec::from_elem(map.width, vec::from_elem(map.height, false)));
		self.map_known = Some(vec::from_elem(map.width, vec::from_elem(map.height, false)));
	}


	fn turn_right(&mut self) {
		 self.direction = self.direction.right();
	}

	fn turn_left(&mut self) -> () {
		self.direction = self.direction.left();
	}

	fn move_forward(&mut self) {
		if_map!(|map| {
			let new_position = self.position.neighbor(self.direction);
			if (map.at(&new_position).is_passable()) {
				self.position = new_position;
			}
		});
	}

	fn move_backward(&mut self) {
		if_map!(|map| {
			let new_position = self.position.neighbor(self.direction.opposite());
			if (map.at(&new_position).is_passable()) {
				self.position = new_position;
			}
		})
	}

	pure fn wrap_position(&self, pos : &Position) -> Position {
		Position {
			x: modulo(pos.x, self.map_width as int),
			y: modulo(pos.y, self.map_height as int)
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

	pure fn knows(&self, pos: &Position) -> bool {
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

	/* Iterate over a rectangle in front of the Creature */
	fn each_in_view_rect(&self, f : &fn(position : &Position)) {
		Position{x:0,y:0}.each_around(PLAYER_VIEW, 2, PLAYER_VIEW, PLAYER_VIEW, f)
	}

	/* Recursive LoS algorithm */
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

		if_map!(|map| { {
			if map.at(pos).can_see_through() {
				for neighbors.each |&d| {
					let n = pos.neighbor(d);
					self.do_view(&n, d, Some(dir), depth - 1);
				}
			} }
		});
	}

	fn update_visibility(&mut self) {
		if_map!(|map| {
			self.map_visible = Some(vec::from_elem(map.width, vec::from_elem(map.height, false)))
		});

		let position = copy self.position;
		let direction = copy self.direction;

		self.do_view(&position, direction, None, PLAYER_VIEW as uint);
	}
}

pub impl Tile {
	fn is_wall(&self) -> bool {
		match *self {
			WALL => true,
			_ => false
		}
	}

	fn is_floor(&self) -> bool {
		match *self {
			FLOOR => true,
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

impl MapView for Map {
	fn at(&self, pos: &Position) -> Tile {
		let p = self.wrap_position(pos);
		self.map[p.x][p.y]
	}
	fn creature_at(&self, pos: &Position) -> Option<@Creature> {
		let p = self.wrap_position(pos);
		self.creatures[p.x][p.y]
	}
	pub fn translate(&self, pos : &Position) -> Position {
		*pos
	}
}

fn each_in_vrect<T: MapView>(s: &T, cp : &Position, rx : int, ry : int, f : &fn(position : Position, t: Tile)) {
	for range(-rx, rx + 1) |vx| {
		for range(-ry, ry + 1) |vy| {
			let x = cp.x + vx;
			let y = cp.y + vy + (vx >> 1);
			let p = Position {x: x as int, y: y as int};
			f(p, s.at(&p));
		}
	}
}

pub impl Map {
	static fn new() -> Map {
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
		let creatures = vec::from_fn(MAP_WIDTH, |x| {
			vec::from_fn(MAP_HEIGHT, |y| {
				if (rng.gen_int_range(0, 15) == 0) {
					let mut c = @Creature::new(
						Position { x: x as int, y: y as int},
						N.turn(rng.gen_int_range(0, 5))
						);
					Some(c)
				} else {
					None
				}
			})
		});

		Map {
			map: map, creatures: creatures,
			width: MAP_WIDTH, height: MAP_HEIGHT
		}
	}

	pure fn wrap_position(&self, pos : &Position) -> Position {
		Position {
			x: modulo(pos.x, self.width as int),
			y: modulo(pos.y, self.height as int)
		}
	}

	fn each(&mut self, f : &fn(position : Position, &mut Tile)) {
		for range(0, self.width as int) |x| {
			for range(0, self.height as int) |y| {
				f(Position {x: x as int, y: y as int}, &mut self.map[x][y]);
			}
		}
	}
	
}

pub impl RelativeMap {
	static fn new(map: @mut Map, pos : &Position, dir : Direction) -> RelativeMap {
		RelativeMap{ map: map, pos: *pos, dir: dir }
	}

	/* Underlying map. */
	fn base(&mut self) -> @mut Map {
		self.map
	}
}

impl MapView for RelativeMap {
	fn at(&self, pos: &Position) -> Tile {
		self.map.at(&self.translate(pos))
	}

	fn creature_at(&self, pos: &Position) -> Option<@Creature> {
		self.map.creature_at(&self.translate(pos))
	}

	fn translate(&self, pos : &Position) -> Position {
		match self.dir {
			N => Position {
				x: self.pos.x + pos.x,
				y: self.pos.y + pos.y
			},
			S => Position {
				x: self.pos.x - pos.x,
				y: self.pos.y - pos.y
			},
			NW => Position {
				x: self.pos.x + pos.y ,
				y: self.pos.y + pos.y - pos.x
			},
			SE => Position {
				x: self.pos.x - pos.y,
				y: self.pos.y - pos.y + pos.x
			},
			NE => Position {
				x: self.pos.x + pos.x - pos.y,
				y: self.pos.y + pos.x
			},
			SW => Position {
				x: self.pos.x - pos.x + pos.y,
				y: self.pos.y - pos.x
			}
		}
	}

}
