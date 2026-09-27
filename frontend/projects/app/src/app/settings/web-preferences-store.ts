import { inject, Injectable, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoService } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import type { MotionPreference, ThemePreference } from 'shared';
import { PreferencesStore } from './preferences-store';

/**
 * The web's preferences: kept for the page only, since nothing is kept in the browser (D37). The
 * language starts as `provideI18n()` chose it — from the browser, or a preference known earlier.
 */
@Injectable()
export class WebPreferencesStore extends PreferencesStore {
  private readonly transloco = inject(TranslocoService);
  private readonly themeSignal = signal<ThemePreference>('system');
  private readonly motionSignal = signal<MotionPreference>('system');

  readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });
  readonly theme = this.themeSignal.asReadonly();
  readonly motion = this.motionSignal.asReadonly();

  async setLanguage(code: string): Promise<void> {
    await firstValueFrom(this.transloco.load(code));
    this.transloco.setActiveLang(code);
  }

  async setTheme(theme: ThemePreference): Promise<void> {
    this.themeSignal.set(theme);
  }

  async setMotion(motion: MotionPreference): Promise<void> {
    this.motionSignal.set(motion);
  }
}
