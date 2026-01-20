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
