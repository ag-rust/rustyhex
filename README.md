Rustyhex is a work toward implementing a simple roguelike game with hex tiles.

It's written in Rust and it's primary purpose is to learn and practice Rust language.

#### Status

ATM (2013-02-17 02:32 CET) the game looks something like this:

![Rustyhex screenshot](http://i.imgur.com/C1EzHzU.png)

The game uses SDL library and [Rust SDL bindings][rust-sdl].

[rust-sdl]: https://github.com/brson/rust-sdl

Currently creatures are roaming around the map and attack anything right in front of them.

#### Keyboard control

Move using Arrow Keys or `hjkl` keys (Vi-like).

To wait a "tick" press `.` or `,`.

Hold Left Shift to strafe, and hold Left Control to attack melee.
