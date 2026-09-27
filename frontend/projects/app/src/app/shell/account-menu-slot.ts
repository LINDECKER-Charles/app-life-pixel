import { InjectionToken, type Type } from '@angular/core';

/**
 * The component shown at the end of the header, where H7 puts the account menu:
 * `{ provide: ACCOUNT_MENU_SLOT, useValue: AccountMenu }` in `app.config.ts`. Empty until then.
 */
export const ACCOUNT_MENU_SLOT = new InjectionToken<Type<unknown>>('ACCOUNT_MENU_SLOT');
