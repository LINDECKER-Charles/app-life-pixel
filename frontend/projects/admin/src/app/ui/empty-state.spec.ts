import { ApiProblem } from 'shared';
import { openPage, seriousViolations, textOf } from '../testing/admin-test-support';
import { EmptyState } from './empty-state';

async function openEmpty(problem: ApiProblem | null): Promise<HTMLElement> {
  const fixture = await openPage(EmptyState, {
    inputs: { nothing: 'admin.logs.empty', problem },
  });
  return fixture.nativeElement.querySelector('[role="status"]') as HTMLElement;
}

describe('EmptyState', () => {
  it('says there is nothing in the range', async () => {
    const empty = await openEmpty(null);

    expect(empty.dataset['reason']).toBe('nothing');
    expect(textOf(empty)).toBe('No log line matches these filters over this period.');
  });

  it('says the source is not configured, and which', async () => {
    const empty = await openEmpty(
      new ApiProblem(503, 'monitoring.not_configured', { source: 'loki' }),
    );

    expect(empty.dataset['reason']).toBe('not_configured');
    expect(textOf(empty)).toContain('This source is not configured on the admin server.');
    expect(textOf(empty)).toContain('loki');
  });

  it('says the source does not answer', async () => {
    const empty = await openEmpty(
      new ApiProblem(503, 'monitoring.unavailable', { source: 'prometheus' }),
    );

    expect(empty.dataset['reason']).toBe('unavailable');
    expect(textOf(empty)).toContain('This source is not answering right now.');
    expect(textOf(empty)).toContain('prometheus');
  });

  it('says the admin server failed', async () => {
    const empty = await openEmpty(new ApiProblem(500, 'internal.error'));

    expect(empty.dataset['reason']).toBe('failed');
    expect(textOf(empty)).toContain('The admin server could not answer.');
  });

  it('passes axe', async () => {
    const fixture = await openPage(EmptyState, { inputs: { nothing: 'admin.users.empty' } });

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
