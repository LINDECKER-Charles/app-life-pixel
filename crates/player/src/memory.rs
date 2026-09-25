//! Vectors reserved once and filled without growing: `load` reserves everything playback needs,
//! so memory never grows while it plays.
//!
//! They are filled through their spare capacity rather than with `extend` or `resize`, whose
//! growth path — dead here — would still bring the allocation-failure panic and its formatting
//! code into the module: about a sixth of the player's size budget.

use alloc::vec::Vec;
use core::iter;

use crate::CallError;

/// A vector of the first `capacity` items of `items` — fewer if `items` ends first.
#[allow(unsafe_code)]
pub(crate) fn collected<T>(
    capacity: usize,
    items: impl IntoIterator<Item = T>,
) -> Result<Vec<T>, CallError> {
    let mut vector = Vec::new();
    vector
        .try_reserve_exact(capacity)
        .map_err(|_| CallError::OutOfMemory)?;
    let mut len = 0;
    let slots = vector.spare_capacity_mut().iter_mut().take(capacity);
    for (slot, item) in slots.zip(items) {
        slot.write(item);
        len += 1;
    }
    // SAFETY: the vector was empty, and the loop initialised its first `len` spare slots, one
    // per count: its first `len` items are initialised.
    unsafe { vector.set_len(len) };
    Ok(vector)
}

/// `len` zero bytes.
pub(crate) fn zeroed(len: usize) -> Result<Vec<u8>, CallError> {
    collected(len, iter::repeat(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collected_keeps_the_first_items_up_to_its_capacity() {
        assert_eq!(collected(3, 1..=5), Ok(alloc::vec![1, 2, 3]));
        assert_eq!(collected(3, [7]), Ok(alloc::vec![7]));
    }

    #[test]
    fn zeroed_is_len_zero_bytes() {
        assert_eq!(zeroed(4), Ok(alloc::vec![0; 4]));
    }
}
