import { ChangeDetectionStrategy, Component, importProvidersFrom, signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { ICON_PATHS } from '../icon/icon-paths';
import { type BannerVariant, StatusBanner } from './status-banner';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

@Component({
  imports: [StatusBanner],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <lp-status-banner [variant]="variant()">
      <p>Could not save. Your work is still open.</p>
      <div lpBannerActions><button type="button">Try again</button></div>
    </lp-status-banner>
  `,
})
class Host {
  readonly variant = signal<BannerVariant>('info');
}

async function render(variant: BannerVariant): Promise<HTMLElement> {
  TestBed.configureTestingModule({
    providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
  });
  await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  const fixture = TestBed.createComponent(Host);
  fixture.componentInstance.variant.set(variant);
  fixture.detectChanges();
  return fixture.nativeElement.querySelector('lp-status-banner');
}

describe('StatusBanner', () => {
  it('announces an error at once, with its icon and the severity in words', async () => {
    const banner = await render('danger');

    expect(banner.getAttribute('role')).toBe('alert');
    expect(banner.classList).toContain('lp-banner--danger');
    expect(banner.querySelector('path')?.getAttribute('d')).toBe(ICON_PATHS.error.outline);
    expect(banner.textContent).toContain(en['common.status.error']);
    expect(banner.textContent).toContain('Could not save. Your work is still open.');
  });

  it('reports the other kinds politely', async () => {
    for (const variant of ['success', 'warning', 'info'] as const) {
      TestBed.resetTestingModule();
      const banner = await render(variant);

      expect(banner.getAttribute('role')).toBe('status');
      expect(banner.querySelector('path')?.getAttribute('d')).toBe(ICON_PATHS[variant].outline);
    }
  });

  it('keeps the projected actions', async () => {
    const banner = await render('warning');

    expect(banner.querySelector('.lp-banner__actions button')?.textContent).toBe('Try again');
  });
});
