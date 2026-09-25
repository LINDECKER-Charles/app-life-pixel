;; A hand-written player implementing ABI v1 (crates/format/README.md) for the loader's tests.
;;
;; It reads a simplified payload, not payload v1: every frame is a single colour, so a test tells
;; the frame shown from any pixel. Little-endian, no padding:
;;
;;   0   u32  magic "LPIX" — status 1 otherwise
;;   4   u16  width         6  u16  height
;;   8   u16  frame count  10  u16  tag count
;;  12   u16  title length 14       title bytes
;;   then per frame: u16 duration in ms, u8 red, green, blue, alpha
;;   then per tag:   u16 first frame, u16 last frame, u8 once, u8 name length, name bytes
;;
;; The payload goes at 1024; `load` grows the memory by a page and puts the framebuffer there, so
;; that a loader keeping a stale `memory.buffer` fails. The playback follows ABI v1's rules,
;; without the modulo on long pauses.
(module
  (memory (export "memory") 1)

  (global $payload i32 (i32.const 1024))
  (global $payload_capacity i32 (i32.const 63488))
  (global $framebuffer i32 (i32.const 65536))
  (global $framebuffer_capacity i32 (i32.const 65536))
  (global $magic i32 (i32.const 0x5849504C))
  (global $header_size i32 (i32.const 14))
  (global $frame_size i32 (i32.const 6))
  (global $whole_animation i32 (i32.const -1))

  (global $is_allocated (mut i32) (i32.const 0))
  (global $is_loaded (mut i32) (i32.const 0))
  (global $width (mut i32) (i32.const 0))
  (global $height (mut i32) (i32.const 0))
  (global $frame_count (mut i32) (i32.const 0))
  (global $tag_count (mut i32) (i32.const 0))
  (global $title_ptr (mut i32) (i32.const 0))
  (global $title_len (mut i32) (i32.const 0))
  (global $frames (mut i32) (i32.const 0))
  (global $tags (mut i32) (i32.const 0))
  (global $first (mut i32) (i32.const 0))
  (global $last (mut i32) (i32.const 0))
  (global $is_range_once (mut i32) (i32.const 0))
  (global $loop_mode (mut i32) (i32.const 0))
  (global $current (mut i32) (i32.const 0))
  (global $elapsed (mut i32) (i32.const 0))
  (global $is_stopped (mut i32) (i32.const 0))

  (func (export "abi_version") (result i32)
    (i32.const 1))

  (func (export "alloc") (param $len i32) (result i32)
    (if (global.get $is_allocated) (then (return (i32.const 0))))
    (global.set $is_allocated (i32.const 1))
    (if (i32.gt_u (local.get $len) (global.get $payload_capacity))
      (then (return (i32.const 0))))
    (global.get $payload))

  (func (export "load") (param $ptr i32) (param $len i32) (result i32)
    (if (i32.or (global.get $is_loaded) (i32.eqz (global.get $is_allocated)))
      (then (return (i32.const 7))))
    (if (i32.lt_u (local.get $len) (global.get $header_size)) (then (return (i32.const 4))))
    (if (i32.ne (i32.load (local.get $ptr)) (global.get $magic)) (then (return (i32.const 1))))
    (global.set $width (i32.load16_u offset=4 (local.get $ptr)))
    (global.set $height (i32.load16_u offset=6 (local.get $ptr)))
    (global.set $frame_count (i32.load16_u offset=8 (local.get $ptr)))
    (global.set $tag_count (i32.load16_u offset=10 (local.get $ptr)))
    (global.set $title_len (i32.load16_u offset=12 (local.get $ptr)))
    (global.set $title_ptr (i32.add (local.get $ptr) (global.get $header_size)))
    (global.set $frames (i32.add (global.get $title_ptr) (global.get $title_len)))
    (global.set $tags
      (i32.add (global.get $frames) (i32.mul (global.get $frame_count) (global.get $frame_size))))
    (if (i32.eqz (global.get $frame_count)) (then (return (i32.const 4))))
    (if (i32.gt_u (call $framebuffer_len) (global.get $framebuffer_capacity))
      (then (return (i32.const 5))))
    (if (i32.eq (memory.grow (i32.const 1)) (i32.const -1)) (then (return (i32.const 6))))
    (global.set $is_loaded (i32.const 1))
    (drop (call $set_tag
      (select (i32.const 0) (global.get $whole_animation) (global.get $tag_count))))
    (i32.const 0))

  (func (export "width") (result i32) (global.get $width))
  (func (export "height") (result i32) (global.get $height))
  (func (export "frame_ptr") (result i32) (global.get $framebuffer))
  (func (export "tag_count") (result i32) (global.get $tag_count))
  (func (export "frame_index") (result i32) (global.get $current))
  (func (export "title_ptr") (result i32) (global.get $title_ptr))
  (func (export "title_len") (result i32) (global.get $title_len))

  (func (export "tag_name_ptr") (param $index i32) (result i32)
    (if (i32.ge_u (local.get $index) (global.get $tag_count)) (then (return (i32.const 0))))
    (i32.add (call $tag_at (local.get $index)) (i32.const 6)))

  (func (export "tag_name_len") (param $index i32) (result i32)
    (if (i32.ge_u (local.get $index) (global.get $tag_count)) (then (return (i32.const 0))))
    (i32.load8_u offset=5 (call $tag_at (local.get $index))))

  (func $set_tag (export "set_tag") (param $index i32) (result i32)
    (local $tag i32)
    (if (i32.eqz (global.get $is_loaded)) (then (return (i32.const 7))))
    (if (i32.eq (local.get $index) (global.get $whole_animation))
      (then
        (global.set $first (i32.const 0))
        (global.set $last (i32.sub (global.get $frame_count) (i32.const 1)))
        (global.set $is_range_once (i32.const 0)))
      (else
        (if (i32.ge_u (local.get $index) (global.get $tag_count)) (then (return (i32.const 8))))
        (local.set $tag (call $tag_at (local.get $index)))
        (global.set $first (i32.load16_u (local.get $tag)))
        (global.set $last (i32.load16_u offset=2 (local.get $tag)))
        (global.set $is_range_once (i32.load8_u offset=4 (local.get $tag)))))
    (call $show (global.get $first))
    (i32.const 0))

  (func (export "set_loop") (param $mode i32) (result i32)
    (if (i32.eqz (global.get $is_loaded)) (then (return (i32.const 7))))
    (if (i32.gt_u (local.get $mode) (i32.const 2)) (then (return (i32.const 8))))
    (global.set $loop_mode (local.get $mode))
    (i32.const 0))

  (func (export "seek") (param $frame i32) (result i32)
    (if (i32.eqz (global.get $is_loaded)) (then (return (i32.const 7))))
    (if (i32.gt_u (local.get $frame) (i32.sub (global.get $last) (global.get $first)))
      (then (return (i32.const 8))))
    (call $show (i32.add (global.get $first) (local.get $frame)))
    (i32.const 0))

  (func (export "tick") (param $elapsed_ms i32) (result i32)
    (local $flags i32)
    (if (i32.or (i32.eqz (global.get $is_loaded)) (global.get $is_stopped))
      (then (return (i32.const 0))))
    (global.set $elapsed (i32.add (global.get $elapsed) (local.get $elapsed_ms)))
    (block $done
      (loop $next
        (br_if $done (i32.lt_u (global.get $elapsed) (call $duration (global.get $current))))
        (global.set $elapsed (i32.sub (global.get $elapsed) (call $duration (global.get $current))))
        (local.set $flags (i32.or (local.get $flags) (call $advance)))
        (br_if $next (i32.eqz (global.get $is_stopped)))))
    (if (i32.and (local.get $flags) (i32.const 1)) (then (call $draw)))
    (local.get $flags))

  ;; Leaves the current frame: the next one, the range's first when it loops, or a stop.
  (func $advance (result i32)
    (if (i32.lt_u (global.get $current) (global.get $last))
      (then
        (global.set $current (i32.add (global.get $current) (i32.const 1)))
        (return (i32.const 1))))
    (if (call $is_looping)
      (then
        (global.set $current (global.get $first))
        (return (i32.const 3))))
    (global.set $is_stopped (i32.const 1))
    (global.set $elapsed (i32.const 0))
    (i32.const 2))

  (func $is_looping (result i32)
    (if (result i32) (global.get $loop_mode)
      (then (i32.eq (global.get $loop_mode) (i32.const 1)))
      (else (i32.eqz (global.get $is_range_once)))))

  (func $show (param $frame i32)
    (global.set $current (local.get $frame))
    (global.set $elapsed (i32.const 0))
    (global.set $is_stopped (i32.const 0))
    (call $draw))

  ;; Fills the framebuffer with the current frame's colour.
  (func $draw
    (local $at i32)
    (local $end i32)
    (local $color i32)
    (local.set $color (i32.load offset=2 (call $frame_at (global.get $current))))
    (local.set $at (global.get $framebuffer))
    (local.set $end (i32.add (local.get $at) (call $framebuffer_len)))
    (block $done
      (loop $next
        (br_if $done (i32.ge_u (local.get $at) (local.get $end)))
        (i32.store (local.get $at) (local.get $color))
        (local.set $at (i32.add (local.get $at) (i32.const 4)))
        (br $next))))

  (func $framebuffer_len (result i32)
    (i32.mul (i32.mul (global.get $width) (global.get $height)) (i32.const 4)))

  (func $frame_at (param $frame i32) (result i32)
    (i32.add (global.get $frames) (i32.mul (local.get $frame) (global.get $frame_size))))

  (func $duration (param $frame i32) (result i32)
    (i32.load16_u (call $frame_at (local.get $frame))))

  ;; The address of tag `index`: tags have names of their own lengths, so it walks them.
  (func $tag_at (param $index i32) (result i32)
    (local $at i32)
    (local.set $at (global.get $tags))
    (block $done
      (loop $next
        (br_if $done (i32.eqz (local.get $index)))
        (local.set $at
          (i32.add (local.get $at) (i32.add (i32.const 6) (i32.load8_u offset=5 (local.get $at)))))
        (local.set $index (i32.sub (local.get $index) (i32.const 1)))
        (br $next)))
    (local.get $at))
)
