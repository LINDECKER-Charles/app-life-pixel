/** The closest ancestor of `element` that scrolls horizontally, if any. */
function horizontalScroller(element: HTMLElement): HTMLElement | null {
  let candidate = element.parentElement;
  while (candidate && candidate.scrollWidth <= candidate.clientWidth) {
    candidate = candidate.parentElement;
  }
  return candidate;
}

/**
 * Scrolls `element`'s horizontal scroller just enough for `element` to show, clear of the
 * `startInset` pixels a sticky head covers. Unlike `scrollIntoView`, it never scrolls vertically,
 * so that stepping through frames never moves the page away from the canvas.
 */
export function revealInline(element: HTMLElement, startInset: number): void {
  const scroller = horizontalScroller(element);
  if (!scroller) return;
  const box = element.getBoundingClientRect();
  const view = scroller.getBoundingClientRect();
  const start = view.left + startInset;
  if (box.left < start) scroller.scrollLeft -= start - box.left;
  else if (box.right > view.right) scroller.scrollLeft += box.right - view.right;
}
