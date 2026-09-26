import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { SupportApi, SupportRequestThread } from './support-api';

function setUp(): { api: SupportApi; http: HttpTestingController } {
  TestBed.configureTestingModule({ providers: [provideHttpClient(), provideHttpClientTesting()] });
  return { api: TestBed.inject(SupportApi), http: TestBed.inject(HttpTestingController) };
}

const CONTEXT = { appVersion: '0.1.0', platform: 'web', language: 'en', screen: '/editor' };
const MESSAGE = {
  id: 'm1',
  author: 'user' as const,
  body: 'The canvas stays blank.',
  createdAt: '2026-09-26T10:00:00Z',
};
const THREAD: SupportRequestThread = {
  id: 'r1',
  category: 'bug',
  status: 'new',
  hasScreenshot: true,
  createdAt: '2026-09-26T10:00:00Z',
  updatedAt: '2026-09-26T10:00:00Z',
  messages: [MESSAGE],
};

describe('SupportApi', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('sends a request as a form, its context as JSON and its screenshot as a file', async () => {
    const { api, http } = setUp();
    const screenshot = new Blob([new Uint8Array([0x89, 0x50])], { type: 'image/png' });
    const answer = api.create({
      category: 'bug',
      message: 'The canvas stays blank.',
      context: CONTEXT,
      screenshot,
    });

    const request = http.expectOne('/api/v1/support-requests');
    expect(request.request.method).toBe('POST');
    const form = request.request.body as FormData;
    expect(form.get('category')).toBe('bug');
    expect(form.get('message')).toBe('The canvas stays blank.');
    expect(JSON.parse(form.get('context') as string)).toEqual(CONTEXT);
    expect((form.get('screenshot') as File).type).toBe('image/png');
    request.flush(THREAD, { status: 201, statusText: 'Created' });
    await expect(answer).resolves.toEqual(THREAD);
  });

  it('leaves the screenshot out when there is none', async () => {
    const { api, http } = setUp();
    const answer = api.create({ category: 'other', message: 'Hello', context: CONTEXT });

    const request = http.expectOne('/api/v1/support-requests');
    expect((request.request.body as FormData).has('screenshot')).toBe(false);
    request.flush({ ...THREAD, hasScreenshot: false }, { status: 201, statusText: 'Created' });
    await answer;
  });

  it('lists the requests from a cursor', async () => {
    const { api, http } = setUp();
    const page = { items: [], nextCursor: null };
    const answer = api.list('c1');

    const request = http.expectOne('/api/v1/support-requests?cursor=c1');
    expect(request.request.method).toBe('GET');
    request.flush(page);
    await expect(answer).resolves.toEqual(page);
  });

  it('reads a thread and replies to it', async () => {
    const { api, http } = setUp();
    const thread = api.get('r 1');
    http.expectOne('/api/v1/support-requests/r%201').flush(THREAD);
    await expect(thread).resolves.toEqual(THREAD);

    const reply = api.reply('r1', 'Still blank');
    const request = http.expectOne('/api/v1/support-requests/r1/messages');
    expect(request.request.method).toBe('POST');
    expect(request.request.body).toEqual({ body: 'Still blank' });
    request.flush(MESSAGE, { status: 201, statusText: 'Created' });
    await expect(reply).resolves.toEqual(MESSAGE);
  });
});
