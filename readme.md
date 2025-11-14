# Rustaclysm

[![GitHub issues](https://img.shields.io/github/issues/papow65/rustaclysm)](https://github.com/papow65/rustaclysm/issues)
[![Code license](https://img.shields.io/badge/license-AGPL(code)-blue.svg)](license.md)
[![Rust version](https://img.shields.io/badge/Rust-latest-orange.svg)](https://www.rust-lang.org/)
[![Bevy engine version](https://img.shields.io/badge/Bevy-0.16-purple.svg)](https://bevyengine.org/)

**Rustaclysm** is a 3D reimagining of the beloved roguelike **[Cataclysm: Dark Days Ahead](https://cataclysmdda.org/)** (C:DDA) built with **[Rust](https://www.rust-lang.org/)** and the **[Bevy](https://bevyengine.org/)** engine.

## Call for contributions

Whether you're a seasoned Rust developer, a Bevy enthusiast, a C:DDA veteran, or just passionate about game development, there are multiple ways to contribute:

*   **Testing:** Ensuring a stable and enjoyable gameplay experience. 🔍
*   **Bug reporting:** Reporting any bugs you encounter. 🐛
*   **Programming:** Implementing various game mechanics. ⌨️
*   **Data parsing:** Improving the loading and parsing of C:DDA json data files. ⚙
*   **Documentation:** Helping us create clear and comprehensive documentation for the project. 💖
*   **Advice:** Suggesting new features or improvements. 💡
*   **UI/UX:** Creating a user-friendly and intuitive interface. 👌

See [contributing.md](contributing.md) for more details

## Goals

1.  **Spirit:** Bring the depth and complexity of C:DDA into an engaging 3D world.
2.  **Data-compatibility:** Utilize the existing C:DDA data files, and save files.
2.  **User-friendly UI:** Create an intuitive and accessible interface, making C:DDA more approachable for new players.

### Long-term expansion goals

4.  **Stability**
5.  **Compatilbility** with multiple C:DDA versions, and C:DDA forks
6.  **Extensible architecture:** Design a modular and well-documented codebase that welcomes community contributions and future expansion.
7.  **3D immersion:** Deliver a visually stunning and immersive 3D experience that captures the atmosphere of C:DDA.

## Supported features

Currently, these features are (partially) supported:

*   Loading existing saves from Cataclysm: Dark Days Ahead 💾
*   Walking, crouching, and running around the world 🏃‍♂️
*   Inventory management 🎒
*   Crafting system 🛠
*   Health and stamina tracking ❤️
*   Combat with zombies and wildlife 🧟
*   Opening and closing doors and windows 🚪
*   Examining items and tiles 🔍
*   Sleeping mechanic 😴
*   Day-night cycle ☀️

## Getting started

### Using a release

1.  Download and install version [0.G Gaiman](https://cataclysmdda.org/releases/) of Cataclysm: Dark Days Ahead.
2.  Create a world and save a character in C:DDA.
3.  Download the [latest release](https://github.com/papow65/rustaclysm/releases) of Rustaclysm.
4.  Run Rustaclysm and then follow the instructions to set up the necessary symlinks or directories.

### Development setup

1.  Install version [0.G Gaiman](https://cataclysmdda.org/releases/) of Cataclysm: Dark Days Ahead.
2.  Create a world and save a character in C:DDA.
3.  Install [git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git), [Rust, and Cargo](https://forge.rust-lang.org/infra/other-installation-methods.html#which-installer-should-you-use)
4.  Clone the Rustaclysm git repository: `git clone https://github.com/papow65/rustaclysm.git`
5.  Navigate to the Rustaclysm directory: `cd rustaclysm`
6.  Launch it by running the command `cargo run --profile dev-opt` on the command line.
7.  Follow the instructions to set up the necessary symlinks or directories in the `assets` directory.

## Environment configuration

These environment variables can be set for development purposes:

| Variable           | Set to         | Description                                            |
|--------------------|----------------|--------------------------------------------------------|
| `DUMP_ENRICHED`    | Directory path | Dump the enriched C:DDA data to the given directory.   |
| `EXIT_AFTER_INFOS` | `1`            | Exit the application when all C:DDA data is processed. |
| `FPS_OVERLAY`      | `1`            | Show the FPS overlay                                   |
| `LOG_ARCHETYPES`   | `1`            | Log the Bevy archetypes used within the application.   |
| `UI_OUTLINES`      | `1`            | Show outlines around every UI node                     |

This project uses environment variables instead of Cargo features for easier maintenance.

## License

See [license.md](license.md)

## Screenshots

[<img src="screenshots/field.png" alt="Field" width="600"/>](screenshots/field.png)

[<img src="screenshots/tower.png" alt="Tower" width="600"/>](screenshots/tower.png)

## Code of conduct

Please follow the [code of conduct](code_of_conduct.md)
