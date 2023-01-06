# Rustaclysm

A 3D reimplementation of [Cataclysm: Dark Days Ahead](https://cataclysmdda.org/) using [Rust](https://www.rust-lang.org/) and [Bevy](https://bevyengine.org/)

## Goals

1. Data-compatibility with Cataclysm: DDA
2. User-friendly UI

## Development stage

Unfinished pre-alpha - Currently you can walk around an existing save from Cataclysm: Dark Days Ahead. You can kill and be killed by zombies. You can open and close doors and pick up and drop items.

## Installation

1. Install the experimental version of Cataclysm: Dark Days Ahead.
2. Create a world in it and save a character in that world. An existing world and character will work as well. We will load this save later.
3. [Install git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
4. [Install Rust and Cargo](https://forge.rust-lang.org/infra/other-installation-methods.html#which-installer-should-you-use)
5. Clone the Rustaclism git repository. When using the command line: `git clone https://github.com/papow65/rustaclysm.git`
6. Navigate to the Rustaclism directory. When using the command line: `cd rustaclysm`
7. Download all dependencies, compile the game, and launch it by running the command `cargo run` on the command line.
8. Follow the instructions to set up the necessary symlinks or directories in the `assets` directory.
9. Run `cargo run` to launch Rustaclysm again. This time the command will be much quicker.

## Screenshots

[<img src="screenshots/field.png" alt="Field" width="400"/>](screenshots/field.png)

[<img src="screenshots/tower.png" alt="Tower" width="400"/>](screenshots/tower.png)

## License

See [license.md](license.md)
