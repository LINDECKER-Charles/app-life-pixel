import { TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { AdminClient } from '../core/admin-client';

/** An admin client whose every call is a mock: a service's test checks what it sends. */
export interface FakeAdminClient {
  readonly request: ReturnType<typeof vi.fn>;
  readonly download: ReturnType<typeof vi.fn>;
  readonly downloadPost: ReturnType<typeof vi.fn>;
}

/** Provides a fake admin client, and returns it. */
export function useFakeAdminClient(): FakeAdminClient {
  const client: FakeAdminClient = { request: vi.fn(), download: vi.fn(), downloadPost: vi.fn() };
  TestBed.configureTestingModule({ providers: [{ provide: AdminClient, useValue: client }] });
  return client;
}
