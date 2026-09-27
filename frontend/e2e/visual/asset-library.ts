import type { Page } from '@playwright/test';
import type { components } from '../../projects/shared/src/lib/api/schema';
import type { Language } from './visual-matrix';

const CREATED_AT = '2026-01-01T12:00:00Z';
const PROJECT = {
  id: '11111111-1111-4111-8111-111111111111',
  name: 'Pixel studies',
  animationCount: 2,
  createdAt: CREATED_AT,
  updatedAt: CREATED_AT,
} satisfies components['schemas']['Project'];
const ANIMATIONS = ['Walk cycle', 'Idle pose'].map((title, index) => ({
  id: `22222222-2222-4222-8222-22222222222${index}`,
  projectId: PROJECT.id,
  title,
  width: 32,
  height: 32,
  frameCount: index === 0 ? 8 : 4,
  documentBytes: 4096,
  version: 1,
  createdAt: CREATED_AT,
  updatedAt: CREATED_AT,
})) satisfies components['schemas']['Animation'][];

/** Synthetic, read-only API responses for reviewing library assets without a real account. */
export class AssetLibrary {
  readonly projectName = PROJECT.name;
  readonly projectPath = `/library/projects/${PROJECT.id}`;
  readonly animationName = ANIMATIONS[0].title;
  private projects: components['schemas']['Project'][] = [];
  private animations: components['schemas']['Animation'][] = [];

  constructor(
    private readonly page: Page,
    private readonly language: Language,
  ) {}

  async install(): Promise<void> {
    await this.page.route('**/api/v1/auth/session', (route) =>
      route.fulfill({ json: { account: this.account(), csrfToken: 'visual-fixture-only' } }),
    );
    await this.page.route('**/api/v1/account', (route) => route.fulfill({ json: this.account() }));
    await this.page.route(
      (url) => url.pathname === '/api/v1/projects',
      (route) => route.fulfill({ json: { items: this.projects, nextCursor: null } }),
    );
    await this.page.route(
      (url) => url.pathname === '/api/v1/animations',
      (route) => route.fulfill({ json: { items: this.animations, nextCursor: null } }),
    );
  }

  populate(): void {
    this.projects = [PROJECT];
    this.animations = ANIMATIONS;
  }

  emptyProject(): void {
    this.projects = [{ ...PROJECT, animationCount: 0 }];
    this.animations = [];
  }

  private account(): components['schemas']['Account'] {
    return {
      id: '33333333-3333-4333-8333-333333333333',
      email: 'artist@example.invalid',
      emailVerified: true,
      language: this.language,
      plan: 'free',
      storage: { usedBytes: this.animations.length * 4096, limitBytes: 10485760 },
      createdAt: CREATED_AT,
    };
  }
}
