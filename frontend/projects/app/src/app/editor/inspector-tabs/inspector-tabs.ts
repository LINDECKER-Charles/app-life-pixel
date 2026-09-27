import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  input,
  model,
  viewChildren,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import type { InspectorTab } from './inspector-tab';

/** Where a key moves the selection among `count` tabs from `index`, wrapping around. */
type TabMove = (index: number, count: number) => number;

const TAB_MOVES: Readonly<Record<string, TabMove>> = {
  ArrowRight: (index, count) => (index + 1) % count,
  ArrowLeft: (index, count) => (index + count - 1) % count,
  Home: () => 0,
  End: (_, count) => count - 1,
};

/**
 * The tabs that swap the inspector's panels on narrower screens (plan C11; tablist pattern of
 * design-system/docs/components.md, "Navigation, tabs and settings"). One tab is selected and
 * alone in the Tab sequence (roving tabindex); Left and Right Arrow move to the previous and next
 * tab, Home and End to the first and last, and the focused tab is selected at once, since its
 * panel shows without delay. The owner draws the panels and hides all but the selected one:
 *
 *     <lp-inspector-tabs [tabs]="tabs" [label]="…" [(selected)]="panelId" />
 *     <section role="tabpanel" [id]="tab.panelId" [attr.aria-labelledby]="tab.panelId + '-tab'"
 *       [hidden]="panelId() !== tab.panelId">…</section>
 */
@Component({
  selector: 'lp-inspector-tabs',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="lp-tabs" role="tablist" [attr.aria-label]="label()">
      @for (tab of tabs(); track tab.panelId) {
        @let isSelected = tab.panelId === selected();
        <button
          #tab
          type="button"
          class="lp-tab"
          role="tab"
          [id]="tab.panelId + '-tab'"
          [attr.aria-selected]="isSelected"
          [attr.aria-controls]="tab.panelId"
          [tabIndex]="isSelected ? 0 : -1"
          (click)="selected.set(tab.panelId)"
          (keydown)="onKeydown($event)"
        >
          {{ tab.labelKey | transloco }}
        </button>
      }
    </div>
  `,
  styles: `
    :host {
      display: block;
    }

    // The tabs share the inspector's width, so that each label stays whole.
    .lp-tab {
      flex: 1 1 auto;
      justify-content: center;
    }
  `,
})
export class InspectorTabs {
  readonly tabs = input.required<readonly InspectorTab[]>();
  /** The tab list's accessible name. */
  readonly label = input.required<string>();
  /** The id of the panel shown. */
  readonly selected = model.required<string>();

  private readonly buttons = viewChildren<ElementRef<HTMLButtonElement>>('tab');

  protected onKeydown(event: KeyboardEvent): void {
    const move = TAB_MOVES[event.key];
    const tabs = this.tabs();
    if (!move || tabs.length === 0) return;
    event.preventDefault();
    const current = tabs.findIndex((tab) => tab.panelId === this.selected());
    const next = move(Math.max(current, 0), tabs.length);
    this.selected.set(tabs[next].panelId);
    this.buttons()[next]?.nativeElement.focus();
  }
}
