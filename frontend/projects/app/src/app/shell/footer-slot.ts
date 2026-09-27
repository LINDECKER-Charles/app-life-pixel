import { InjectionToken, type Type } from '@angular/core';

/**
 * The component the footer holds, where H16 puts the links to the legal pages:
 * `{ provide: FOOTER_SLOT, useValue: LegalLinks }` in `app.config.ts`. No footer until then.
 */
export const FOOTER_SLOT = new InjectionToken<Type<unknown>>('FOOTER_SLOT');
