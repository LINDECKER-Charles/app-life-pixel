import { ComponentFixture } from '@angular/core/testing';
import { vi } from 'vitest';
import { REQUEST } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { SupportApi } from './support-api';
import { SupportQueuePage } from './support-queue-page';

async function openQueue(
  queue: ReturnType<typeof vi.fn>,
  inputs: Record<string, unknown> = {},
): Promise<ComponentFixture<SupportQueuePage>> {
  const fixture = await openPage(SupportQueuePage, {
    url: '/support',
    inputs,
    providers: [{ provide: SupportApi, useValue: { queue } }],
  });
  await settled(() => expect(textOf(fixture.nativeElement)).not.toContain('Loading'));
  return fixture;
}

describe('SupportQueuePage', () => {
  it('filters the queue by status, category and the admin’s own requests', async () => {
    const queue = vi.fn().mockResolvedValue({ items: [REQUEST], nextCursor: null });
    await openQueue(queue, { status: 'new', category: 'data_protection', assignee: 'me' });

    expect(queue).toHaveBeenCalledWith(
      { status: 'new', category: 'data_protection', assignee: 'adm-1' },
      undefined,
    );
  });

  it('ignores a filter it does not know', async () => {
    const queue = vi.fn().mockResolvedValue({ items: [], nextCursor: null });
    await openQueue(queue, { status: 'lost', category: 'spam' });

    expect(queue).toHaveBeenCalledWith(
      { status: undefined, category: undefined, assignee: undefined },
      undefined,
    );
  });

  it('shows the legal deadline of a data-protection request', async () => {
    const fixture = await openQueue(
      vi.fn().mockResolvedValue({ items: [REQUEST], nextCursor: null }),
    );
    const row = fixture.nativeElement.querySelector('tbody tr') as HTMLElement;

    expect(textOf(row)).toContain('Data protection');
    expect(textOf(row.querySelector('.deadline'))).toMatch(/^Legal deadline: .+, \d+ days left$/);
    expect(row.querySelector('a')?.getAttribute('href')).toBe(
      '/support/r1?env=staging&from=now-24h&to=now',
    );
  });

  it('says the queue is empty', async () => {
    const fixture = await openQueue(vi.fn().mockResolvedValue({ items: [], nextCursor: null }));

    expect(textOf(fixture.nativeElement.querySelector('[data-reason="nothing"]'))).toBe(
      'No support request matches these filters.',
    );
  });

  it('passes axe', async () => {
    const fixture = await openQueue(
      vi.fn().mockResolvedValue({ items: [REQUEST], nextCursor: null }),
    );

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
