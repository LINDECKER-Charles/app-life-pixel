/**
 * Whether every finite animation of the page has ended — run in the page, with
 * `page.waitForFunction`. Ionic's overlays animate inside their shadow roots, which
 * `document.getAnimations()` does not see: each shadow root is asked too.
 */
export function finiteAnimationsEnded(): boolean {
  const roots: (Document | ShadowRoot)[] = [document];
  // An array's iterator reads its length at each step: nested shadow roots are visited too.
  for (const root of roots) {
    for (const element of root.querySelectorAll('*')) {
      if (element.shadowRoot) roots.push(element.shadowRoot);
    }
  }
  return roots.every((root) =>
    root
      .getAnimations()
      .every(
        (animation) =>
          animation.playState !== 'running' ||
          animation.effect?.getTiming().iterations === Infinity,
      ),
  );
}
