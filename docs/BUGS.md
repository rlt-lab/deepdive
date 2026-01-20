# Bug Tracker

Quick bug tracking for issues noticed during development.

## Open Bugs

### BUG-001: Tile flash on level descent
**Reported:** 2026-01-19
**Priority:** Medium
**Status:** Open

**Description:**
Many tiles flash on screen when descending downstairs before the map is correctly rendered.

**Suspected cause:**
Likely related to mapgen logic timing - tiles may be rendered before generation is complete or visibility state is properly initialized.

**Steps to reproduce:**
1. Find down stairs
2. Press X to descend
3. Observe brief flash of tiles before map renders correctly

---

### BUG-002: Map generation not reproducible with same seed
**Reported:** 2026-01-19
**Priority:** Low
**Status:** Open

**Description:**
Map generation produces different results even with the same RNG seed. This prevents save/load from regenerating identical maps and makes debugging harder.

**Suspected cause:**
`CompactOrganicGenerator::generate_organic_boundary()` in `src/map_generation_compact.rs` uses `HashSet` iteration (lines 51, 81) which has non-deterministic ordering in Rust.

**Fix:**
Replace `HashSet` with `BTreeSet` or sort the iterator output before processing.

**Related tests:**
- `tests/map_generation_tests.rs::map_generation_same_seed_identical_output` (ignored)
- `tests/map_generation_tests.rs::map_generation_reproducibility_all_biomes` (ignored)

---

### BUG-003: LOS (Line of Sight) not symmetric
**Reported:** 2026-01-19
**Priority:** Low
**Status:** Open

**Description:**
Bresenham's line algorithm used for LOS calculation is not symmetric. If A can see B, B may not necessarily see A. This is a known limitation of integer-based line-drawing algorithms where the path from A to B can differ from the path from B to A due to integer rounding.

**Suspected cause:**
`has_line_of_sight()` in `src/fov.rs` uses standard Bresenham's algorithm which has inherent asymmetry when stepping diagonally. The algorithm makes different rounding decisions depending on the direction of travel.

**Fix:**
Options:
1. Use a symmetric variant of Bresenham's (e.g., double-check both directions)
2. Use a different LOS algorithm like ray-casting with proper symmetry
3. Accept the asymmetry as a game mechanic (enemies may spot you before you see them)

**Impact:**
Mostly cosmetic/fairness. In rare cases, a player may not see an enemy that can see them (or vice versa). The current implementation is still functional for gameplay.

**Related tests:**
- `tests/property_fov.rs::prop_los_symmetric_with_walls` (ignored)

---

## Closed Bugs

(None yet)

---

## Template

```markdown
### BUG-XXX: Title
**Reported:** YYYY-MM-DD
**Priority:** Low/Medium/High/Critical
**Status:** Open/In Progress/Closed

**Description:**
What happens.

**Suspected cause:**
Any theories.

**Steps to reproduce:**
1. Step one
2. Step two
```
