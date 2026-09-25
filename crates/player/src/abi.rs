//! Player ABI v1: the functions the module exports, each a one-line call into the module's single
//! [`Player`]. Every value is a 32-bit integer; `crates/format/README.md` specifies each export.

use core::cell::UnsafeCell;

use crate::{CallError, Player};

/// The status of a call that completed.
const DONE: u32 = 0;

/// The module's only player.
struct SinglePlayer(UnsafeCell<Player>);

// SAFETY: a `wasm32-unknown-unknown` module built without the `atomics` feature has one thread,
// so the player is never reached from two threads at once.
#[allow(unsafe_code)]
unsafe impl Sync for SinglePlayer {}

static PLAYER: SinglePlayer = SinglePlayer(UnsafeCell::new(Player::new()));

/// Runs `call` on the module's player.
#[allow(unsafe_code)]
fn with_player<T>(call: impl FnOnce(&mut Player) -> T) -> T {
    // SAFETY: the module has one thread, and an export neither calls another export nor anything
    // outside the module, so the reference below is the only one to the player while it lives.
    let player = unsafe { &mut *PLAYER.0.get() };
    call(player)
}

/// A call's result as the status the export returns.
fn status(result: Result<(), CallError>) -> u32 {
    result.map_or_else(|error| error.status(), |()| DONE)
}

/// An address as the export returns it: `wasm32` addresses fit in 32 bits.
fn pointer(address: usize) -> u32 {
    u32::try_from(address).unwrap_or(0)
}

/// The address of a block as the export returns it; `0` without a block.
fn block_pointer(block: Option<&mut [u8]>) -> u32 {
    block.map_or(0, |block| pointer(block.as_ptr().addr()))
}

/// An address the loader passed, as the player takes it.
fn address(pointer: u32) -> usize {
    usize::try_from(pointer).unwrap_or(usize::MAX)
}

/// Exports each function under its own name, the name ABI v1 gives it.
macro_rules! exports {
    ($($(#[doc = $doc:literal])+ fn $name:ident($($argument:ident),*) $body:block)+) => {$(
        $(#[doc = $doc])+
        // SAFETY: `no_mangle` exports the function under its name. The names are those of ABI v1,
        // each defined once, and none is a symbol of `core`, `alloc` or the compiler's builtins,
        // which are mangled or prefixed: no definition is replaced.
        #[allow(unsafe_code)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $name($($argument: u32),*) -> u32 $body
    )+};
}

exports! {
    /// `1`: the ABI this player implements.
    fn abi_version() { with_player(|player| player.abi_version()) }

    /// Reserves `len` bytes for the payload and returns their address; `0` if it cannot, or on a
    /// second call.
    fn alloc(len) { with_player(|player| block_pointer(player.alloc(len))) }

    /// Checks the payload written at `ptr`, then shows the first frame of the initial range.
    fn load(ptr, len) { with_player(|player| status(player.load(address(ptr), len))) }

    /// The canvas width, in pixels.
    fn width() { with_player(|player| player.width()) }

    /// The canvas height, in pixels.
    fn height() { with_player(|player| player.height()) }

    /// The framebuffer: `width × height × 4` bytes of RGBA, rows top to bottom.
    fn frame_ptr() { with_player(|player| pointer(player.frame_ptr())) }

    /// Advances playback; bit 0: the framebuffer changed; bit 1: the range reached its end.
    fn tick(elapsed_ms) { with_player(|player| player.tick(elapsed_ms)) }

    /// The number of tags.
    fn tag_count() { with_player(|player| player.tag_count()) }

    /// The address of tag `index`'s UTF-8 name; `0` for an index out of range.
    fn tag_name_ptr(index) { with_player(|player| pointer(player.tag_name_ptr(index))) }

    /// The length of tag `index`'s name; `0` for an index out of range.
    fn tag_name_len(index) { with_player(|player| player.tag_name_len(index)) }

    /// Plays tag `index`, or the whole animation for `0xFFFFFFFF`, from its first frame.
    fn set_tag(index) { with_player(|player| status(player.set_tag(index))) }

    /// `0` the range's own mode, `1` loop, `2` once.
    fn set_loop(mode) { with_player(|player| status(player.set_loop(mode))) }

    /// Shows frame `frame` of the current range, counted from its first frame.
    fn seek(frame) { with_player(|player| status(player.seek(frame))) }

    /// The animation frame shown.
    fn frame_index() { with_player(|player| player.frame_index()) }

    /// The address of the UTF-8 title.
    fn title_ptr() { with_player(|player| pointer(player.title_ptr())) }

    /// The length of the title, in bytes.
    fn title_len() { with_player(|player| player.title_len()) }
}
