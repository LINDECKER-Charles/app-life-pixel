import { HttpTestingController } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { openAccountPage } from '../account/testing/account-test-support';
import { NotFoundPage } from './not-found-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

describe('NotFoundPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('says the page does not exist, on the page template, and leads back to the editor', async () => {
    const fixture = await openAccountPage(NotFoundPage);
    const root: HTMLElement = fixture.nativeElement;

    expect(root.querySelector('ion-content > .lp-page h1')?.textContent?.trim()).toBe(
      'Page not found',
    );
    expect(root.querySelectorAll('h1')).toHaveLength(1);
    expect(root.querySelector('main')).toBeNull();
    expect(root.querySelector('a')?.getAttribute('href')).toBe('/editor');
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(NotFoundPage);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
