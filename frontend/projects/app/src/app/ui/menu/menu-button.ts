import { DOCUMENT } from '@angular/common';
import {
  afterNextRender,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  inject,
  Injector,
  input,
  signal,
  viewChild,
} from '@angular/core';
import { Icon } from '../icon/icon';

/** Where a key moves the focus among `count` items from `index` (-1: none focused yet). */
type FocusMove = (index: number, count: number) => number;

const FOCUS_MOVES: Readonly<Record<string, FocusMove>> = {
  ArrowDown: (index, count) => (index + 1) % count,
  ArrowUp: (index, count) => (index <= 0 ? count : index) - 1,
  Home: () => 0,
  End: (_, count) => count - 1,
};

const MENU_ITEM_SELECTOR = '[role="menuitem"]:not([aria-disabled="true"], :disabled)';

let nextMenuId = 0;

/**
 * A button that opens a menu of actions and links (WAI-ARIA APG "Menu Button";
 * design-system/docs/accessibility.md, "Menu"). Enter, Space or Down Arrow opens it on its first
 * item, Up Arrow on its last; arrows, Home and End move among the items; Escape closes it and
 * returns the focus to the button; Tab, an outside click or choosing an item closes it.
 *
 *     <lp-menu-button [label]="'…' | transloco">
 *       <span lpMenuTrigger>…</span>                              the button's content
 *       <p lpMenuHeader class="lp-menu__header">…</p>              optional, outside the menu
 *       <a role="menuitem" class="lp-menu__item" routerLink="…">…</a>
 *       <hr role="separator" class="lp-menu__separator" />
 *       <button role="menuitem" class="lp-menu__item" type="button">…</button>
 *     </lp-menu-button>
 */
@Component({
  selector: 'lp-menu-button',
  imports: [Icon],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    '(keydown)': 'moveFocus($event)',
    '(click)': 'choose($event)',
    '(document:click)': 'closeOnOutsideClick($event)',
    '(focusout)': 'closeOnFocusLeaving($event)',
  },
  template: `
    <button
      #trigger
      type="button"
      [class]="triggerClass()"
      aria-haspopup="menu"
      [attr.aria-expanded]="isOpen()"
      [attr.aria-controls]="menuId"
      (click)="toggle()"
      (keydown)="openFromKeyboard($event)"
    >
      <ng-content select="[lpMenuTrigger]" />
      <lp-icon name="chevron-down" size="small" />
    </button>
    <div class="lp-menu" [hidden]="!isOpen()">
      <ng-content select="[lpMenuHeader]" />
      <div #menu role="menu" class="lp-menu__list" [id]="menuId" [attr.aria-label]="label()">
        <ng-content />
      </div>
    </div>
  `,
  styles: `
    :host {
      position: relative;
      display: inline-block;
    }
  `,
})
export class MenuButton {
  /** The menu's accessible name. */
  readonly label = input.required<string>();
  /** The trigger's shared button classes (`_buttons.scss`). */
  readonly triggerClass = input('lp-button lp-button--quiet lp-button--compact');

  protected readonly menuId = `lp-menu-${nextMenuId++}`;
  protected readonly isOpen = signal(false);

  private readonly host: HTMLElement = inject(ElementRef).nativeElement;
  private readonly document = inject(DOCUMENT);
  private readonly injector = inject(Injector);
  private readonly trigger = viewChild.required<ElementRef<HTMLButtonElement>>('trigger');
  private readonly menu = viewChild.required<ElementRef<HTMLElement>>('menu');

  protected toggle(): void {
    if (this.isOpen()) {
      this.close();
    } else {
      this.openOn('Home');
    }
  }

  protected openFromKeyboard(event: KeyboardEvent): void {
    if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
    event.preventDefault();
    this.openOn(event.key === 'ArrowUp' ? 'End' : 'Home');
  }

  /** The menu's keys, heard on the host: they come from its items. */
  protected moveFocus(event: KeyboardEvent): void {
    if (!this.menu().nativeElement.contains(event.target as Node)) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      this.close();
      this.trigger().nativeElement.focus();
      return;
    }
    if (event.key === 'Tab') {
      this.close();
      return;
    }
    this.moveOrActivate(event);
  }

  protected choose(event: MouseEvent): void {
    if (!(event.target as Element).closest('[role="menuitem"]')) return;
    this.close();
    this.trigger().nativeElement.focus();
  }

  protected closeOnOutsideClick(event: MouseEvent): void {
    if (this.isOpen() && !this.host.contains(event.target as Node)) this.close();
  }

  protected closeOnFocusLeaving(event: FocusEvent): void {
    const next = event.relatedTarget as Node | null;
    if (next && !this.host.contains(next)) this.close();
  }

  private moveOrActivate(event: KeyboardEvent): void {
    const items = this.items();
    const current = items.indexOf(this.document.activeElement as HTMLElement);
    // Space activates a button natively, not a link.
    if (event.key === ' ' && items[current]?.localName === 'a') {
      event.preventDefault();
      items[current].click();
      return;
    }
    const move = FOCUS_MOVES[event.key];
    if (!move || items.length === 0) return;
    event.preventDefault();
    items[move(current, items.length)].focus();
  }

  private openOn(key: 'Home' | 'End'): void {
    this.isOpen.set(true);
    afterNextRender(
      () => {
        const items = this.items();
        items.forEach((item) => (item.tabIndex = -1));
        items[FOCUS_MOVES[key](-1, items.length)]?.focus();
      },
      { injector: this.injector },
    );
  }

  private close(): void {
    this.isOpen.set(false);
  }

  private items(): HTMLElement[] {
    return Array.from(this.menu().nativeElement.querySelectorAll<HTMLElement>(MENU_ITEM_SELECTOR));
  }
}
