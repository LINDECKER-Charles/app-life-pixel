import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter, Router } from '@angular/router';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { PlatformService } from '../../platform/platform';
import { AppHeader } from './app-header';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

async function render(isDesktop: boolean): Promise<HTMLElement> {
  TestBed.configureTestingModule({
    providers: [
      provideRouter([{ path: '**', children: [] }]),
      importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      { provide: PlatformService, useValue: { desktop: isDesktop } },
    ],
  });
  await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  await TestBed.inject(Router).navigateByUrl('/library');
  const fixture = TestBed.createComponent(AppHeader);
  await fixture.whenStable();
  return fixture.nativeElement;
}

function linkTexts(root: HTMLElement): string[] {
  return Array.from(root.querySelectorAll('nav a'), (link) => link.textContent?.trim() ?? '');
}

describe('AppHeader', () => {
  it('names the app beside its decorative mark, and leads to the editor', async () => {
    const root = await render(false);
    const brand = root.querySelector('.brand');

    expect(brand?.textContent?.trim()).toBe(en['app.name']);
    expect(brand?.getAttribute('href')).toBe('/editor');
    expect(brand?.querySelector('img')?.getAttribute('alt')).toBe('');
  });

  it('links to every area on the web, the current one marked', async () => {
    const root = await render(false);

    expect(root.querySelector('nav')?.getAttribute('aria-label')).toBe(en['shell.nav.label']);
    expect(linkTexts(root)).toEqual([
      en['shell.nav.editor'],
      en['shell.nav.library'],
      en['shell.nav.settings'],
      en['shell.nav.help'],
    ]);
    expect(root.querySelector('[aria-current="page"]')?.textContent?.trim()).toBe(
      en['shell.nav.library'],
    );
  });

  it('leaves Help out on the desktop, where support is not available', async () => {
    const root = await render(true);

    expect(linkTexts(root)).not.toContain(en['shell.nav.help']);
    expect(root.querySelector('a[href="/support"]')).toBeNull();
  });

  it('hides its icons from assistive technology', async () => {
    const root = await render(false);

    const icons = Array.from(root.querySelectorAll('lp-icon'));
    expect(icons.length).toBeGreaterThan(0);
    expect(icons.every((icon) => icon.getAttribute('aria-hidden') === 'true')).toBe(true);
  });
});
