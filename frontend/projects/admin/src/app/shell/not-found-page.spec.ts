import { openPage, seriousViolations, textOf } from '../testing/admin-test-support';
import { NotFoundPage } from './not-found-page';

describe('NotFoundPage', () => {
  it('leads back to the overview, in the same view', async () => {
    const fixture = await openPage(NotFoundPage, { url: '/nowhere?env=production' });
    const link = fixture.nativeElement.querySelector('a') as HTMLAnchorElement;

    expect(textOf(fixture.nativeElement.querySelector('h1'))).toBe('This page does not exist');
    expect(link.getAttribute('href')).toBe('/?env=production&from=now-24h&to=now');
    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
