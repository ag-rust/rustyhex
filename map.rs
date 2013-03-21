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

pub enum Action {
	MOVE_FORWARD,
	MOVE_BACKWARD,
	TURN_LEFT,
	TURN_RIGHT,
	WAIT
}

pub trait MoveController {
	fn get_move(&mut self, &mut Map, cr: @mut Creature) -> Action;
}

pub struct Creature {
	pos : Position,
	dir : Direction,
	action : Option<Action>,
	pre_action_ticks : uint,
	post_action_ticks : uint,
	map_visible : ~[ ~[ bool ] ],
	map_known : ~[ ~[ bool ] ],
	map_height: uint,
	map_width: uint
}

pub enum Tile {
	FLOOR,
	WALL
}

const MAP_WIDTH : uint = 32;
const MAP_HEIGHT : uint = 32;

pub struct Map {
	tiles : ~[ ~[ Tile ] ],
	creatures : ~[ ~[ Option<@mut Creature> ] ],
	width: uint,
	height : uint
}

pub trait MapView {
	pure fn at(&self, pos: &Position) -> Tile;
	pure fn creature_at(&self, pos: &Position) -> Option<@mut Creature>;
	pure fn translate(&self, pos : &Position) -> Position;
}

/**
 * View of the map with rotation (dir) and offset (pos)
 */
pub struct RelativeMap {
	map : &'self mut Map,
	pos : Position,
	dir : Direction
}

pub impl Action {
	pure fn pre_ticks(&self) -> uint {
		match *self {
			MOVE_FORWARD => 10u,
			MOVE_BACKWARD => 15u,
			TURN_RIGHT => 7u,
			TURN_LEFT => 7u,
			WAIT => 1u
		}
	}
	pure fn post_ticks(&self) -> uint {
		match *self {
			MOVE_FORWARD => 10u,
			MOVE_BACKWARD => 10u,
			TURN_RIGHT => 7u,
			TURN_LEFT => 7u,
			WAIT => 0u
		}
	}
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

const PLAYER_VIEW: int = 10;
pub impl Creature {
	static fn new(map : &mut Map, position : &Position, direction : Direction) -> Creature {
		Creature {
			pos : *position, dir : direction,
			action: None, pre_action_ticks: 0, post_action_ticks: 0,
			map_visible: vec::from_elem(map.width, vec::from_elem(map.height, false)),
			map_known: vec::from_elem(map.width, vec::from_elem(map.height, false)),
			map_width: map.width,
			map_height: map.height,
		}
	}

	fn tick<T: MoveController>(@mut self, map : &mut Map, cr : &mut T) -> bool {
		let mut redraw = false;
		if (self.pre_action_ticks > 0) {
			self.pre_action_ticks -= 1;
		} else {
			match (self.action) {
				Some(action) => {
					redraw = true;
					match (action) {
						MOVE_FORWARD => self.move_forward(map),
						MOVE_BACKWARD => self.move_backward(map),
						TURN_RIGHT => self.turn_right(),
						TURN_LEFT => self.turn_left(),
						WAIT => {}
					}
					self.action = None
				}
				None => {
					if (self.post_action_ticks > 0) {
						self.post_action_ticks -= 1;
					} else {
						let action = cr.get_move(map, self);
						self.action = Some(action);
						self.pre_action_ticks = action.pre_ticks();
						self.post_action_ticks = action.post_ticks();
					}
				}
			}
		}
		redraw
	}

	fn turn_right(@mut self) {
		 self.dir = self.dir.right();
	}

	fn turn_left(@mut self) -> () {
		self.dir = self.dir.left();
	}

	fn move_forward(@mut self, map : &mut Map) {
		let new_position = self.pos.neighbor(self.dir);
		if (map.at(&new_position).is_passable()) {
			map.move_creature(self, &new_position);
		}
	}

	fn move_backward(@mut self, map : &mut Map) {
		let new_position = self.pos.neighbor(self.dir.opposite());
		self.mark_known(map, &new_position);
		if (map.at(&new_position).is_passable()) {
			map.move_creature(self, &new_position);
		}
	}

	fn mark_visible(&mut self, map : &Map, pos : &Position) {
		let p = map.wrap_position(pos);

		self.map_visible[p.x][p.y] = true;
	}

	fn mark_known(&mut self, map : &Map,  pos : &Position) {
		let p = map.wrap_position(pos);

		self.map_known[p.x][p.y] = true;
	}

	pure fn sees(&self, map : &Map, pos: &Position) -> bool {
		let p = map.wrap_position(pos);

		self.map_visible[p.x][p.y]
	}

	pure fn knows(&self, map : &Map, pos: &Position) -> bool {
		let p = map.wrap_position(pos);

		self.map_known[p.x][p.y]
	}

	pure fn position(&self) -> Position {
		self.pos
	}

	/* Iterate over a rectangle in front of the Creature */
	fn each_in_view_rect(&self, f : &fn(position : &Position)) {
		Position{x:0,y:0}.each_around(PLAYER_VIEW, 2, PLAYER_VIEW, PLAYER_VIEW, f)
	}

	/* Very hacky, recursive LoS algorithm */
	fn do_view(&mut self, map : &mut Map, pos: &Position,
		main_dir : Direction, dir : Option<Direction>, pdir : Option<Direction>, depth: uint) {
		if (depth == 0) {
			return;
		}

		self.mark_visible(map, pos);
		self.mark_known(map, pos);

		let neighbors = match (dir, pdir) {
			(Some(dir), Some(pdir)) => {
				if dir == pdir {
					~[dir]
				} else {
					~[dir, pdir]
				}
			},
			(Some(dir), None) => {
				if main_dir == dir {
					~[dir, dir.left(), dir.right()]
				} else {
					~[dir, main_dir]
				}
			},
			_ => {
				~[main_dir, main_dir.left(), main_dir.right()]
			}
		};

		if map.at(pos).can_see_through() {
			for neighbors.each |&d| {
				let n = pos.neighbor(d);
				match dir {
					Some(_) => {
						self.do_view(map, &n, d, Some(d), dir, depth - 1);
					},
					None => {
						self.do_view(map, &n, main_dir, Some(d), dir, depth - 1);
					}
				};
			}
		}
	}

	fn update_visibility(&mut self, map : &mut Map) {
		self.map_visible = vec::from_elem(map.width, vec::from_elem(map.height, false));

		let position = copy self.pos;
		let direction = copy self.dir;

		self.do_view(map, &position, direction, None, None, PLAYER_VIEW as uint);
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
	pure fn at(&self, pos: &Position) -> Tile {
		let p = self.wrap_position(pos);
		self.tiles[p.x][p.y]
	}
	pure fn creature_at(&self, pos: &Position) -> Option<@mut Creature> {
		let p = self.wrap_position(pos);
		self.creatures[p.x][p.y]
	}
	pure fn translate(&self, pos : &Position) -> Position {
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

		let creatures = vec::from_fn(MAP_WIDTH, |_| {
			vec::from_fn(MAP_HEIGHT, |_| {
				None
			})
		});

		Map {
			tiles: map, creatures: creatures,
			width: MAP_WIDTH, height: MAP_HEIGHT
		}
	}

	pure fn wrap_position(&self, pos : &Position) -> Position {
		Position {
			x: modulo(pos.x, self.width as int),
			y: modulo(pos.y, self.height as int)
		}
	}

	fn for_each_tile(&mut self, f : &fn(Position, &mut Tile)) {
		for range(0, self.width as int) |x| {
			for range(0, self.height as int) |y| {
				f(Position {x: x as int, y: y as int}, &mut self.tiles[x][y]);
			}
		}
	}

	fn for_each_creature(&mut self, f : &fn(@mut Creature)) {
		for range(0, self.width as int) |x| {
			for range(0, self.height as int) |y| {
				match (self.creatures[x][y]) {
					Some(creature) => f(creature),
					None => {}
				}
			}
		}
	}

	fn spawn_creature(&mut self, pos : &Position, dir : Direction) -> Option<@mut Creature> {
		match (self.creatures[pos.x][pos.y]) {
			Some(_) => None,
			None => {
				let mut c = @mut Creature::new(self, pos, dir);
				self.creatures[pos.x][pos.y] = Some(c);
				Some(c)
			}
		}
	}

	fn spawn_random_creature(&mut self) -> @mut Creature {
		let rng = rand::Rng();
		let pos = &Position{
			x: rng.gen_int_range(0, self.width as int),
			y: rng.gen_int_range(0, self.height as int)
		};

		let dir = N.turn(rng.gen_int_range(0, 6));

		match (self.spawn_creature(pos, dir)) {
			None => self.spawn_random_creature(),
			Some(creature) => creature
		}
	}

	fn move_creature(&mut self, cr : @mut Creature, pos : &Position) {
		let pos = &self.wrap_position(pos);
		match (self.creatures[pos.x][pos.y]) {
			Some(_) => {},
			None => {
				self.creatures[cr.pos.x][cr.pos.y] = None;
				cr.pos = *pos;
				self.creatures[pos.x][pos.y] = Some(cr);
			}
		}
	}
}

pub impl<'self> RelativeMap<'self> {
	static fn new(map: &'r mut Map, pos : &Position, dir : Direction) -> RelativeMap/&r {
		RelativeMap{ map: map, pos: *pos, dir: dir }
	}

	/* Underlying map. */
	fn base(&self) -> &self/mut Map {
		&mut *self.map
	}
}

impl MapView for RelativeMap<'self> {
	pure fn at(&self, pos: &Position) -> Tile {
		self.map.at(&self.translate(pos))
	}

	pure fn creature_at(&self, pos: &Position) -> Option<@mut Creature> {
		self.map.creature_at(&self.translate(pos))
	}

	pure fn translate(&self, pos : &Position) -> Position {
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
