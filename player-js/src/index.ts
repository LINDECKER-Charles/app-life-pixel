import { LifePixelElement } from './life-pixel-element';

const TAG_NAME = 'life-pixel';

// Two copies of the loader on one page share the first definition.
if (!customElements.get(TAG_NAME)) customElements.define(TAG_NAME, LifePixelElement);

export { LifePixelElement };
