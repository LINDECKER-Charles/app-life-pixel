import { openPage, seriousViolations, textOf } from '../testing/admin-test-support';
import { bannerTone, EnvironmentBanner } from './environment-banner';

describe('EnvironmentBanner', () => {
  it('is amber for staging, red for production and neutral otherwise', () => {
    expect(bannerTone('staging')).toBe('staging');
    expect(bannerTone('production')).toBe('production');
    expect(bannerTone('local')).toBe('neutral');
    expect(bannerTone('')).toBe('neutral');
  });

  it('names the environment every action changes', async () => {
    const fixture = await openPage(EnvironmentBanner, { inputs: { environment: 'staging' } });
    const banner = fixture.nativeElement.querySelector('.banner') as HTMLElement;

    expect(banner.dataset['tone']).toBe('staging');
    expect(textOf(banner)).toBe('Staging Every action here changes this environment’s data.');
    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });

  it('names an environment it has no name for as it is', async () => {
    const fixture = await openPage(EnvironmentBanner, { inputs: { environment: 'qa-2' } });

    expect(textOf(fixture.nativeElement.querySelector('strong'))).toBe('qa-2');
  });
});
