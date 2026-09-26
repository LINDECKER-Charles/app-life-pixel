import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { vi } from 'vitest';
import { FileSaver } from '../core/file-saver';
import { USER } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { UsersApi } from './users-api';
import { UsersPage } from './users-page';

async function openUsers(
  search: ReturnType<typeof vi.fn>,
  inputs: Record<string, unknown> = {},
): Promise<ComponentFixture<UsersPage>> {
  const fixture = await openPage(UsersPage, {
    url: '/users?env=production',
    inputs,
    providers: [{ provide: UsersApi, useValue: { search } }],
  });
  await settled(() => expect(textOf(fixture.nativeElement)).not.toContain('Loading'));
  return fixture;
}

describe('UsersPage', () => {
  it('lists the users found, each opening its detail in the same view', async () => {
    const search = vi.fn().mockResolvedValue({ items: [USER], nextCursor: null });
    const fixture = await openUsers(search, { q: 'lee', status: 'active' });
    const link = fixture.nativeElement.querySelector('tbody a') as HTMLAnchorElement;

    expect(search).toHaveBeenCalledWith({ q: 'lee', status: 'active', cursor: undefined });
    expect(textOf(link)).toBe('lee@example.com');
    expect(link.getAttribute('href')).toBe('/users/u1?env=production&from=now-24h&to=now');
    expect(textOf(fixture.nativeElement.querySelector('tbody'))).toContain('1.5 MB');
  });

  it('searches through the URL', async () => {
    const fixture = await openUsers(vi.fn().mockResolvedValue({ items: [], nextCursor: null }));
    const form = fixture.nativeElement.querySelector('form') as HTMLFormElement;
    (form.querySelector('input[name="q"]') as HTMLInputElement).value = ' kim ';

    form.dispatchEvent(new Event('submit'));

    await settled(() => expect(TestBed.inject(Router).url).toBe('/users?env=production&q=kim'));
  });

  it('sorts by a column from its header, and says which', async () => {
    const other = { ...USER, id: 'u2', email: 'abe@example.com' };
    const search = vi.fn().mockResolvedValue({ items: [USER, other], nextCursor: null });
    const fixture = await openUsers(search);
    const emailHeader = fixture.nativeElement.querySelector('th[scope="col"]') as HTMLElement;

    (emailHeader.querySelector('button') as HTMLButtonElement).click();
    await fixture.whenStable();

    expect(emailHeader.getAttribute('aria-sort')).toBe('ascending');
    expect(textOf(fixture.nativeElement.querySelector('tbody a'))).toBe('abe@example.com');
  });

  it('exports the table as CSV', async () => {
    const fixture = await openUsers(vi.fn().mockResolvedValue({ items: [USER], nextCursor: null }));

    Array.from(fixture.nativeElement.querySelectorAll('button') as NodeListOf<HTMLButtonElement>)
      .find((button) => textOf(button) === 'Export as CSV')
      ?.click();

    const saver = TestBed.inject(FileSaver) as unknown as { saveCsv: ReturnType<typeof vi.fn> };
    expect(saver.saveCsv).toHaveBeenCalledWith([USER], expect.any(Array), {
      table: 'users',
      environment: 'staging',
    });
  });

  it('says nothing matches a search', async () => {
    const fixture = await openUsers(vi.fn().mockResolvedValue({ items: [], nextCursor: null }), {
      q: 'nobody',
    });

    expect(textOf(fixture.nativeElement.querySelector('[data-reason="nothing"]'))).toBe(
      'No account matches this search.',
    );
  });

  it('passes axe', async () => {
    const fixture = await openUsers(vi.fn().mockResolvedValue({ items: [USER], nextCursor: 'c' }));

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
