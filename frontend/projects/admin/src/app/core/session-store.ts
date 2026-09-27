import { computed, inject, Injectable } from '@angular/core';
import { ApiProblem } from 'shared';
import { AdminClient } from './admin-client';
import type { SignInRequest } from './admin-types';
import { SessionState } from './session-state';

/**
 * Who is signed in to the console, the environment it administers and those its monitoring
 * shows: read once at start, then set by signing in and out (support-admin.md, H11).
 */
@Injectable({ providedIn: 'root' })
export class SessionStore {
  private readonly client = inject(AdminClient);
  private readonly state = inject(SessionState);

  readonly session = this.state.session;
  readonly signedIn = computed(() => !!this.state.session());
  readonly admin = computed(() => this.state.session()?.admin ?? null);
  /** `LPA_ENVIRONMENT`: the environment whose data every action changes. */
  readonly environment = computed(() => this.state.session()?.environment ?? '');
  /** `LPA_ENVIRONMENTS`: the environments the monitoring views show. */
  readonly monitoredEnvironments = computed(
    () => this.state.session()?.monitoredEnvironments ?? [],
  );

  /** Reads the live session; none is `admin.unauthenticated`, and leaves the console signed out. */
  async load(): Promise<void> {
    try {
      this.state.set(await this.client.request('get', '/api/admin/v1/auth/session'));
    } catch (error: unknown) {
      this.state.set(null);
      if (!(error instanceof ApiProblem)) {
        console.error('The admin session could not be read.', error);
      }
    }
  }

  /** Signs in: a wrong address, password or code throws `admin.invalid_credentials`. */
  async signIn(request: SignInRequest): Promise<void> {
    const session = await this.client.request('post', '/api/admin/v1/auth/sign-in', {
      body: request,
    });
    this.state.set(session);
  }

  async signOut(): Promise<void> {
    try {
      await this.client.request('post', '/api/admin/v1/auth/sign-out');
    } finally {
      this.state.set(null);
    }
  }
}
