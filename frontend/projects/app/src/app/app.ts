import { ChangeDetectionStrategy, Component, ElementRef, inject, viewChild } from '@angular/core';
import { IonApp, IonRouterOutlet } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { AppFooter } from './shell/app-footer';
import { EngineNotifications } from './shell/engine-notifications';
import { AppHeader } from './shell/header/app-header';
import { SupportScreen } from './support/support-screen';

/** Draws the focus ring inside the element the skip link focuses (_page.scss). */
const SKIP_TARGET_CLASS = 'lp-skip-target';

/**
 * Where the skip link sends the focus in the routed page, the last the outlet holds: the page
 * itself when it is the main landmark (the editor), else the container `ion-content` — the main
 * landmark of the other pages — slots. Never a shadow host such as `ion-content` or the outlet:
 * one with a negative tabindex drops its whole content from the Tab order.
 */
function skipTarget(outlet: HTMLElement): HTMLElement {
  const page = (outlet.lastElementChild as HTMLElement | null) ?? outlet;
  const content = page.querySelector(':scope > ion-content');
  return (content?.firstElementChild as HTMLElement | null) ?? page;
}

/**
 * The shell: a skip link, the header, the engine's messages, the routed page — which holds the
 * one main landmark — and the footer. It starts following the screens a support request names
 * as its context.
 */
@Component({
  imports: [AppFooter, AppHeader, EngineNotifications, IonApp, IonRouterOutlet, TranslocoPipe],
  selector: 'lp-root',
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {
  private readonly outlet = viewChild.required('outlet', { read: ElementRef<HTMLElement> });

  constructor() {
    inject(SupportScreen).start();
  }

  /**
   * Moves the focus to the page's main content without changing the URL: a fragment would go
   * through the router, which could run its guards again.
   */
  protected skipToContent(event: Event): void {
    event.preventDefault();
    const target = skipTarget(this.outlet().nativeElement);
    target.classList.add(SKIP_TARGET_CLASS);
    target.tabIndex = -1;
    target.focus();
  }
}
