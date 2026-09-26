import { expect, type APIResponse, type Page } from '@playwright/test';
import { HOSTED_APP_URL } from './stack/ports';

/** The media type of an animation document (server.md, H3). */
const DOCUMENT_TYPE = 'application/vnd.life-pixel.animation+json';

interface Project {
  readonly id: string;
  readonly name: string;
}

async function json<T>(response: APIResponse): Promise<T> {
  expect(response.ok(), `${response.url()} answered ${response.status()}`).toBe(true);
  return (await response.json()) as T;
}

/**
 * The public API as the signed-in person's page calls it, sharing its cookies: for what a
 * journey sets up rather than tests, such as a library filled close to its quota.
 */
export class LibraryApi {
  constructor(private readonly page: Page) {}

  /** The bytes the account's library uses. */
  async usedBytes(): Promise<number> {
    const account = await json<{ storage: { usedBytes: number } }>(
      await this.page.request.get('/api/v1/account'),
    );
    return account.storage.usedBytes;
  }

  async projectId(name: string): Promise<string> {
    const page = await json<{ items: Project[] }>(await this.page.request.get('/api/v1/projects'));
    const project = page.items.find((item) => item.name === name);
    if (!project) throw new Error(`No project is named ${name}.`);
    return project.id;
  }

  /** Creates an animation of `document` in the project `projectId`, as the editor's save does. */
  async createAnimation(projectId: string, document: string): Promise<void> {
    const response = await this.page.request.post(`/api/v1/projects/${projectId}/animations`, {
      headers: { ...(await this.unsafeHeaders()), 'Content-Type': DOCUMENT_TYPE },
      data: document,
    });
    await json(response);
  }

  /** What a change needs (accounts.md, H5): the app's origin, and the session's CSRF token. */
  private async unsafeHeaders(): Promise<Record<string, string>> {
    const session = await json<{ csrfToken: string }>(
      await this.page.request.get('/api/v1/auth/session'),
    );
    return { Origin: HOSTED_APP_URL, 'X-CSRF-Token': session.csrfToken };
  }
}
