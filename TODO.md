# RustTS -- TODO

## Stub / No-op Implementations

- [ ] Audio -- `play_selection_sound` and `play_attack_sound` are empty; no sounds play
- [ ] Animation -- `AnimationConfig` exists but no units use sprite sheets; facing_system works
- [ ] Death state -- `UnitState::Dead` is never used; units despawn instantly

---

## Core Game Loop

### Technologies / Upgrades

- [ ] `Blacksmith` building: upgrades attack and armor
- [ ] Unit upgrades (e.g. Militia -> Man-at-Arms, Archer -> Crossbowman)
- [ ] Research costs resources and takes time
- [ ] Upgraded stats apply to existing and future units

### AI Improvements

- [ ] AI difficulty levels
- [ ] AI target priority: units -> buildings -> Town Center
- [ ] AI rebuilds destroyed buildings
- [ ] AI sends coordinated attack waves instead of trickle

---

## UI

- [ ] Training queue visual (show icons in bottom panel)
- [ ] Unit group hotkeys (ctrl+1 to assign, 1 to select)

---

## Polish / Future

- [ ] Actual sprite sheet art (units are colored circles, buildings mostly procedural)
- [ ] Background music
- [ ] SFX: selection acknowledgment, attack, gathering, death
- [ ] Fog of war
- [ ] Multiple terrain layers (elevation, forests blocking LOS)
- [ ] Population cap (houses increase cap)
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
