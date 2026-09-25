# `life-pixel-player`

The player ABI v1 module. `crates/format/README.md` (owned by F2) is the canonical
specification of payload v1 and player ABI v1; this file tracks this crate's own departures
from, and additions to, that specification, until F2 folds them into the canonical document.

## Contract change: `tick`'s bit 2, `RANGE_STOPPED`

Added by the fix for wave 5's Group 2 (`.sillage/reports/wave-5.md`): `player-js`'s loader
inferred whether a range that just ended had stopped (played once) or looped, by comparing
`frame_index()` after the tick to the range's first frame. That heuristic is unsound in two
cases, both reachable by a real compiled export and reproduced by
`player-js/tests/golden-exports.spec.ts`:

- A single-frame range played once: `frame_index()` never leaves the range's only frame, so it
  never differs from the range's first frame, even once stopped.
- A looping range whose elapsed time is reduced modulo its total duration (`Playback::tick`, for
  a `tick` call far longer than one cycle): the frame it lands on afterwards can be anywhere in
  the range, including its last frame, without the range having stopped.

Both cases make it impossible for a caller to tell "stopped" from "looping" by frame position
alone, from any information ABI v1 exposed before this change: no export gave the range's own
loop mode, its last frame, or its stopped state.

`tick` now sets bit 2 (`RANGE_STOPPED`, value `4`) together with bit 1 (`RANGE_ENDED`) exactly
when the range stopped on its last frame rather than looping back to its first — the same
`is_stopped` transition `Playback::leave_frame` already tracked internally. Bit 2 is never set
without bit 1. `player-js/src/player-instance.ts` uses it to answer `hasStopped()` and masks it
out of the flags it hands the element, so `FRAME_CHANGED` (bit 0) and `RANGE_ENDED` (bit 1) keep
their existing meaning for callers.

F2 owns folding this bit into `crates/format/README.md`'s `tick` row and Rust API notes; this
crate's own doc comments (`src/playback.rs`, `src/abi.rs`, `src/player.rs`) already document it.
