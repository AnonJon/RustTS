# RustTS -- TODO

## Stub / No-op Implementations

- [ ] Audio -- `play_selection_sound` and `play_attack_sound` are empty; no sounds play
- [ ] Animation -- `AnimationConfig` exists but no units use sprite sheets; facing_system works
- [x] Death state -- units now enter `UnitState::Dead` with 3s corpse fade before despawn

---

## Core Game Loop

### Technologies / Upgrades

- [x] Loom, Wheelbarrow, Hand Cart -- researched at Town Center, affects villager HP/armor/speed/carry
- [ ] `Blacksmith` building: upgrades attack and armor
- [ ] Unit upgrades (e.g. Militia -> Man-at-Arms, Archer -> Crossbowman)
- [x] Research costs resources and takes time
- [x] Upgraded stats apply to existing and future units

### AI Improvements

- [ ] AI difficulty levels
- [ ] AI target priority: units -> buildings -> Town Center
- [ ] AI rebuilds destroyed buildings
- [ ] AI sends coordinated attack waves instead of trickle

---

## UI

- [ ] Training queue visual (show icons in bottom panel)
- [x] Unit group hotkeys (ctrl+1 to assign, 1 to select, double-tap to center camera)

---

## Phase 1 Complete

- [x] Population cap -- Houses (+5 pop), TC (+5), Castle (+20), max 200, enforced on training
- [x] Fog of war -- wired into MapPlugin, units have LineOfSight, enemy entities hidden in fog
- [x] Building repair -- villagers right-click damaged buildings to repair at 50% build cost
- [x] TC arrows -- Town Center fires arrows (base 5 pierce, +garrison archer bonus)
- [x] Control groups -- Ctrl+1..0 assign, 1..0 recall, double-tap centers camera

---

## Polish / Future

- [ ] Actual sprite sheet art (units are colored circles, buildings mostly procedural)
- [ ] Background music
- [ ] SFX: selection acknowledgment, attack, gathering, death
- [ ] Multiple terrain layers (elevation, forests blocking LOS)
- [ ] Relics / map control objectives

---

## Art Assets

### Building sprites -- PARTIAL

- [x] Lumber Camp
- [x] Mining Camp
- [ ] Town Center
- [ ] Barracks
- [ ] Archery Range
- [ ] Stable
- [ ] Blacksmith
- [ ] Farm, Mill

### Minimum art needed per unit

- [ ] 8 directional walking animations (N, NE, E, SE, S, SW, W, NW) x ~6 frames
- [ ] Idle animation per direction
- [ ] Attack animation per direction
- [ ] Death animation (can be direction-agnostic)
