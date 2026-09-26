import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { vi } from 'vitest';
import { FileSaver } from '../core/file-saver';
import { USER_DETAIL } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { UserPage } from './user-page';
import { UsersApi } from './users-api';

type FakeUsersApi = Record<
  'get' | 'suspend' | 'reactivate' | 'export' | 'delete',
  ReturnType<typeof vi.fn>
>;

function fakeApi(): FakeUsersApi {
  return {
    get: vi.fn().mockResolvedValue(USER_DETAIL),
    suspend: vi.fn().mockResolvedValue(undefined),
    reactivate: vi.fn().mockResolvedValue(undefined),
    export: vi.fn().mockResolvedValue({ blob: new Blob(['zip']), fileName: null }),
    delete: vi.fn().mockResolvedValue(undefined),
  };
}

async function openUser(api: FakeUsersApi): Promise<ComponentFixture<UserPage>> {
  const fixture = await openPage(UserPage, {
    url: '/users/u1?env=production',
    inputs: { id: 'u1' },
    providers: [{ provide: UsersApi, useValue: api }],
  });
  await settled(() => expect(fixture.nativeElement.querySelector('h1')).not.toBeNull());
  return fixture;
}

async function ask(fixture: ComponentFixture<UserPage>, label: string): Promise<HTMLDialogElement> {
  Array.from(fixture.nativeElement.querySelectorAll('.actions button') as NodeListOf<HTMLElement>)
    .find((button) => textOf(button) === label)
    ?.click();
  await fixture.whenStable();
  return fixture.nativeElement.querySelector('dialog');
}

async function confirmWith(fixture: ComponentFixture<UserPage>, reason: string): Promise<void> {
  const dialog = fixture.nativeElement.querySelector('dialog') as HTMLDialogElement;
  const textarea = dialog.querySelector('textarea');
  if (textarea) {
    textarea.value = reason;
    textarea.dispatchEvent(new Event('input'));
    await fixture.whenStable();
  }
  (dialog.querySelector('button[type="submit"]') as HTMLButtonElement).click();
}

describe('UserPage', () => {
  it('shows the account', async () => {
    const fixture = await openUser(fakeApi());
    const page = textOf(fixture.nativeElement);

    expect(textOf(fixture.nativeElement.querySelector('h1'))).toBe('lee@example.com');
    expect(page).toContain('3 projects, 7 animations');
    expect(page).toContain('export.completed');
  });

  it('suspends only once confirmed, naming this console’s environment and the account', async () => {
    const api = fakeApi();
    const fixture = await openUser(api);

    const dialog = await ask(fixture, 'Suspend');
    expect(textOf(dialog.querySelector('.environment'))).toBe('Staging');
    expect(textOf(dialog.querySelector('.target'))).toBe('lee@example.com');
    expect(api.suspend).not.toHaveBeenCalled();

    await confirmWith(fixture, 'Spam, reported twice');

    await settled(() => expect(api.suspend).toHaveBeenCalledWith('u1', 'Spam, reported twice'));
    await settled(() =>
      expect(textOf(fixture.nativeElement.querySelector('.notice'))).toBe(
        'The account is suspended.',
      ),
    );
    expect(fixture.nativeElement.querySelector('dialog')).toBeNull();
  });

  it('changes nothing when the confirmation is cancelled', async () => {
    const api = fakeApi();
    const fixture = await openUser(api);
    const dialog = await ask(fixture, 'Delete the account');

    (dialog.querySelector('button[type="button"]') as HTMLButtonElement).click();
    await fixture.whenStable();

    expect(fixture.nativeElement.querySelector('dialog')).toBeNull();
    expect(api.delete).not.toHaveBeenCalled();
  });

  it('deletes with a reason, then goes back to the users', async () => {
    const api = fakeApi();
    const fixture = await openUser(api);
    await ask(fixture, 'Delete the account');

    await confirmWith(fixture, 'Erasure request r1');

    await settled(() => expect(TestBed.inject(Router).url).toMatch(/^\/users\?env=production/));
    expect(api.delete).toHaveBeenCalledWith('u1', 'Erasure request r1');
  });

  it('downloads the export once confirmed', async () => {
    const api = fakeApi();
    const fixture = await openUser(api);
    const dialog = await ask(fixture, 'Export the data');
    expect(textOf(dialog.querySelector('.target'))).toBe('lee@example.com');

    await confirmWith(fixture, '');

    const saver = TestBed.inject(FileSaver) as unknown as { save: ReturnType<typeof vi.fn> };
    await settled(() =>
      expect(saver.save).toHaveBeenCalledWith(expect.any(Blob), 'life-pixel-export-u1.zip'),
    );
  });

  it('passes axe, with its confirmation open', async () => {
    const fixture = await openUser(fakeApi());
    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);

    await ask(fixture, 'Suspend');

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
