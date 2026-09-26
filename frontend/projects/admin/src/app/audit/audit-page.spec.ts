import { ComponentFixture } from '@angular/core/testing';
import { vi } from 'vitest';
import type { AuditEntry } from '../core/admin-types';
import { AUDIT_ENTRY } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { AuditApi } from './audit-api';
import { AuditPage } from './audit-page';

async function openAudit(
  search: ReturnType<typeof vi.fn>,
  inputs: Record<string, unknown> = {},
): Promise<ComponentFixture<AuditPage>> {
  const fixture = await openPage(AuditPage, {
    url: '/audit',
    inputs,
    providers: [{ provide: AuditApi, useValue: { search } }],
  });
  await settled(() => expect(textOf(fixture.nativeElement)).not.toContain('Loading'));
  return fixture;
}

describe('AuditPage', () => {
  it('searches the audit log by action and target', async () => {
    const search = vi.fn().mockResolvedValue({ items: [AUDIT_ENTRY], nextCursor: null });
    const fixture = await openAudit(search, { action: 'user.suspend', targetId: ' u1 ' });

    expect(search).toHaveBeenCalledWith(
      { adminId: undefined, action: 'user.suspend', targetId: 'u1' },
      undefined,
    );
    const row = textOf(fixture.nativeElement.querySelector('tbody tr'));
    expect(row).toContain('ops@example.com');
    expect(row).toContain('Spam');
  });

  it('says nothing matches', async () => {
    const fixture = await openAudit(vi.fn().mockResolvedValue({ items: [], nextCursor: null }));

    expect(textOf(fixture.nativeElement.querySelector('[data-reason="nothing"]'))).toBe(
      'No admin action matches this search.',
    );
  });

  it('passes axe', async () => {
    // The schema types a snapshot as an empty object: any JSON object, as the log keeps it.
    const snapshot = (status: string): AuditEntry['before'] =>
      ({ status }) as unknown as AuditEntry['before'];
    const entry = { ...AUDIT_ENTRY, before: snapshot('active'), after: snapshot('suspended') };
    const fixture = await openAudit(
      vi.fn().mockResolvedValue({ items: [entry], nextCursor: null }),
    );

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
