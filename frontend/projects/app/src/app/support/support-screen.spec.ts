import { provideLocationMocks } from '@angular/common/testing';
import { Component } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter, Router } from '@angular/router';
import { TranslocoService } from '@jsverse/transloco';
import { LIFE_PIXEL_CLIENT } from 'shared';
import { SupportScreen } from './support-screen';

@Component({ selector: 'lp-blank', template: '' })
class Blank {}

function configure(): void {
  TestBed.configureTestingModule({
    providers: [
      provideRouter([
        { path: 'editor', component: Blank },
        { path: 'editor/:animationId', component: Blank },
        { path: 'legal/:page', component: Blank },
        { path: 'support', component: Blank },
        { path: 'support/:requestId', component: Blank },
      ]),
      provideLocationMocks(),
      { provide: LIFE_PIXEL_CLIENT, useValue: 'web/1.2.3' },
      { provide: TranslocoService, useValue: { getActiveLang: () => 'fr' } },
    ],
  });
}

function setUp(): { screen: SupportScreen; router: Router } {
  configure();
  const screen = TestBed.inject(SupportScreen);
  screen.start();
  return { screen, router: TestBed.inject(Router) };
}

describe('SupportScreen', () => {
  it('starts on the editor, with the client and the language', () => {
    const { screen } = setUp();

    expect(screen.context()).toEqual({
      appVersion: '1.2.3',
      platform: 'web',
      language: 'fr',
      screen: '/editor',
    });
  });

  it('remembers the route template of the last screen before support, never an id', async () => {
    const { screen, router } = setUp();

    await router.navigateByUrl('/editor/0190f6a2-7c1e-7d3a-9b4e-5f6a7b8c9d0e');
    expect(screen.context().screen).toBe('/editor/:animationId');
    await router.navigateByUrl('/legal/privacy');
    await router.navigateByUrl('/support');
    await router.navigateByUrl('/support/r1');

    expect(screen.context().screen).toBe('/legal/:page');
  });

  it('counts the screen already shown when it starts', async () => {
    configure();
    await TestBed.inject(Router).navigateByUrl('/editor/a1');
    const screen = TestBed.inject(SupportScreen);

    screen.start();

    expect(screen.context().screen).toBe('/editor/:animationId');
  });
});
