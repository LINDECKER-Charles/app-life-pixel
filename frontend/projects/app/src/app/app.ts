import { ChangeDetectionStrategy, Component, ElementRef, inject, viewChild } from '@angular/core';
import { IonApp, IonRouterOutlet } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { AppFooter } from './shell/app-footer';
import { EngineNotifications } from './shell/engine-notifications';
import { AppHeader } from './shell/header/app-header';
import { SupportScreen } from './support/support-screen';

/** Draws the focus ring inside the page, which fills the outlet (_page.scss). */
const SKIP_TARGET_CLASS = 'lp-skip-target';

/**
 * The shell: a skip link, the header, the engine's messages, the routed page — the main
 * landmark — and the footer. It starts following the screens a support request names as its
 * context.
 */
@Component({
  imports: [AppFooter, AppHeader, EngineNotifications, IonApp, IonRouterOutlet, TranslocoPipe],
  selector: 'lp-root',
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {
  private readonly main = viewChild.required('main', { read: ElementRef<HTMLElement> });

  constructor() {
    inject(SupportScreen).start();
  }

  /**
   * Moves the focus to the routed page, the last the outlet holds, without changing the URL: a
   * fragment would go through the router, which could run its guards again. Not to the outlet
   * itself: a shadow host with a negative tabindex drops its whole content from the Tab order.
   */
  protected skipToContent(event: Event): void {
    event.preventDefault();
    const outlet = this.main().nativeElement;
    const page = (outlet.lastElementChild as HTMLElement | null) ?? outlet;
    page.classList.add(SKIP_TARGET_CLASS);
    page.tabIndex = -1;
    page.focus();
  }
}
