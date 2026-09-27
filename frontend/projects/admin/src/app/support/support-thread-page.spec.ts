import { ComponentFixture } from '@angular/core/testing';
import { vi } from 'vitest';
import { THREAD } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { SupportApi } from './support-api';
import { SupportThreadPage } from './support-thread-page';

type FakeSupportApi = Record<'thread' | 'update' | 'post' | 'screenshot', ReturnType<typeof vi.fn>>;

function fakeApi(): FakeSupportApi {
  return {
    thread: vi.fn().mockResolvedValue(THREAD),
    update: vi.fn().mockResolvedValue(THREAD.request),
    post: vi.fn().mockResolvedValue(THREAD.messages[0]),
    screenshot: vi.fn().mockResolvedValue(new Blob(['png'], { type: 'image/png' })),
  };
}

async function openThread(api: FakeSupportApi): Promise<ComponentFixture<SupportThreadPage>> {
  const fixture = await openPage(SupportThreadPage, {
    url: '/support/r1',
    inputs: { id: 'r1' },
    providers: [{ provide: SupportApi, useValue: api }],
  });
  await settled(() => expect(fixture.nativeElement.querySelector('h1')).not.toBeNull());
  return fixture;
}

function button(fixture: ComponentFixture<SupportThreadPage>, label: string): HTMLButtonElement {
  const buttons = fixture.nativeElement.querySelectorAll('button') as NodeListOf<HTMLButtonElement>;
  const found = Array.from(buttons).find((candidate) => textOf(candidate) === label);
  if (!found) {
    throw new Error(`No button "${label}"`);
  }
  return found;
}

describe('SupportThreadPage', () => {
  it('shows the thread, its internal notes marked, the context and the legal deadline', async () => {
    const fixture = await openThread(fakeApi());
    const page = fixture.nativeElement as HTMLElement;

    expect(textOf(page.querySelector('h1'))).toBe('Data protection · New');
    expect(textOf(page.querySelector('li.internal'))).toContain('Internal note');
    expect(textOf(page.querySelector('li[data-author="user"]'))).toContain(
      'Please send me all my data.',
    );
    expect(textOf(page.querySelector('.deadline'))).toMatch(/^Legal deadline: .+ days left$/);
    expect(textOf(page.querySelector('aside'))).toContain('1.2.0');
  });

  it('assigns the request to the admin', async () => {
    const api = fakeApi();
    const fixture = await openThread(api);

    button(fixture, 'Assign to me').click();

    await settled(() => expect(api.update).toHaveBeenCalledWith('r1', { assignedTo: 'adm-1' }));
    await settled(() =>
      expect(textOf(fixture.nativeElement.querySelector('.notice'))).toBe(
        'The request is assigned to you.',
      ),
    );
  });

  it('changes the status', async () => {
    const api = fakeApi();
    const fixture = await openThread(api);
    const form = fixture.nativeElement.querySelector('form.status') as HTMLFormElement;
    (form.querySelector('select') as HTMLSelectElement).value = 'resolved';

    form.dispatchEvent(new Event('submit'));

    await settled(() => expect(api.update).toHaveBeenCalledWith('r1', { status: 'resolved' }));
  });

  it('adds an internal note, chosen explicitly', async () => {
    const api = fakeApi();
    const fixture = await openThread(api);
    const compose = fixture.nativeElement.querySelector('form.compose') as HTMLFormElement;
    const note = compose.querySelector('input[value="note"]') as HTMLInputElement;
    note.checked = true;
    note.dispatchEvent(new Event('change'));
    const body = compose.querySelector('textarea') as HTMLTextAreaElement;
    body.value = ' Identity checked by phone. ';
    body.dispatchEvent(new Event('input'));
    await fixture.whenStable();

    button(fixture, 'Add the note').click();

    await settled(() =>
      expect(api.post).toHaveBeenCalledWith('r1', 'Identity checked by phone.', true),
    );
  });

  it('shows the screenshot on demand', async () => {
    const createObjectURL = vi.fn().mockReturnValue('blob:shot');
    vi.stubGlobal('URL', Object.assign(URL, { createObjectURL, revokeObjectURL: vi.fn() }));
    const fixture = await openThread(fakeApi());

    button(fixture, 'Show the screenshot').click();

    await settled(() =>
      expect(fixture.nativeElement.querySelector('img')?.getAttribute('src')).toBe('blob:shot'),
    );
    expect(fixture.nativeElement.querySelector('img')?.getAttribute('alt')).toBe(
      'The screenshot the user sent with the request',
    );
    vi.unstubAllGlobals();
  });

  it('passes axe', async () => {
    const fixture = await openThread(fakeApi());

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
