# RustTS

A real-time strategy game built in Rust, inspired by Age of Empires II. The name is a play on "RTS" -- because it's an RTS written in Rust.

## About

RustTS aims to recreate the core mechanics of classic AoE2-style gameplay: gather resources, advance through ages, build an army, and defeat your opponent. The game features:

- **Isometric diamond tilemap** with AoE2 DE terrain textures
- **Resource economy** -- food, wood, gold, and stone with villager gather/carry/drop-off cycle
- **Age progression** -- advance from Dark Age through Imperial Age by meeting building and resource requirements
- **Unit types** -- villagers, militia, archers, and knights with distinct stats
- **Building system** -- town centers, barracks, lumber camps, mining camps, farms, and more
- **AI opponent** -- economy-driven AI that builds, trains, and attacks
- **Procedural map generation** -- seed-based RMS-style phased terrain generation with coherent forests, lakes, and dirt patches
- **A\* pathfinding** with terrain-aware navigation
- **Interactive minimap**, resource HUD, unit info panel, and age-up progress bar

## Tech Stack

- **[Bevy 0.18](https://bevyengine.org/)** -- ECS game engine
- **[bevy_ecs_tilemap 0.18](https://github.com/StarArawn/bevy_ecs_tilemap)** -- isometric tilemap rendering
- **[rand 0.9](https://crates.io/crates/rand)** -- seeded RNG for deterministic map generation
- **[image 0.25](https://crates.io/crates/image)** -- terrain texture atlas construction

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 2024 edition)
- macOS, Linux, or Windows

### Build and Run

```bash
cargo run
```

The first build takes a few minutes due to Bevy compilation. Subsequent builds are fast (~5-15s).

### Controls

| Input                | Action                  |
| -------------------- | ----------------------- |
| Left-click           | Select unit             |
| Click-drag           | Box select units        |
| Right-click ground   | Move selected units     |
| Right-click resource | Gather (villagers only) |
| Right-click enemy    | Attack                  |
| B                    | Open build menu         |
| WASD / Arrow keys    | Pan camera              |
| Mouse at screen edge | Pan camera              |
| Scroll wheel         | Zoom in/out             |

## Contributing

Contributions are welcome. Here's how to get involved:

### Areas That Need Work

- **Unit sprites** -- replace placeholder circles with actual isometric unit sprites
- **Building sprites** -- most buildings still use procedural colored rectangles
- **Audio** -- background music and sound effects are not yet implemented
- **More unit types** -- siege weapons, monks, cavalry archers, etc.
- **Tech tree** -- per-civilization research and upgrades
- **Multiplayer** -- networked play with deterministic lockstep
- **Performance** -- flow-field pathfinding for large unit counts
- **Map variety** -- additional map generation scripts (islands, rivers, arena)

### How to Contribute

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make your changes and ensure `cargo build` succeeds with no new warnings
4. Test by running `cargo run` and verifying gameplay works
5. Open a pull request with a description of what you changed and why

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Run `cargo clippy` and address warnings
- Keep ECS systems focused -- one system per behavior
- Use `GridPosition` for all logical tile coordinates; never hardcode world pixel positions
- All new game objects should go through `GridPosition::to_world()` for positioning

## License

MIT
