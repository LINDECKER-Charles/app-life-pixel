import { InjectionToken } from '@angular/core';
import type { AnimationFilter, AnimationSummary, Page, PageQuery, Project } from './library-types';

/**
 * The library, wherever it lives (accounts.md, H8): `HttpLibraryStore` over the hosted account's
 * API, the desktop's local folder in T2. Every failure rejects with a `LibraryFailure`
 * (`{ code, params }`), never a sentence.
 */
export interface LibraryStore {
  listProjects(page?: PageQuery): Promise<Page<Project>>;
  createProject(name: string): Promise<Project>;
  renameProject(id: string, name: string): Promise<Project>;
  duplicateProject(id: string, name: string): Promise<Project>;
  deleteProject(id: string): Promise<void>;
  listAnimations(filter: AnimationFilter, page?: PageQuery): Promise<Page<AnimationSummary>>;
  createAnimation(projectId: string, document: Uint8Array): Promise<AnimationSummary>;
  openDocument(id: string): Promise<{ summary: AnimationSummary; document: Uint8Array }>;
  saveDocument(id: string, document: Uint8Array, version: number): Promise<AnimationSummary>;
  renameAnimation(id: string, title: string, version: number): Promise<AnimationSummary>;
  moveAnimation(id: string, projectId: string): Promise<AnimationSummary>;
  duplicateAnimation(id: string, title: string, projectId?: string): Promise<AnimationSummary>;
  deleteAnimation(id: string): Promise<void>;
  usage(): Promise<{ usedBytes: number; limitBytes: number | null }>;
}

/** The library of the platform: provided once in `app.config.ts`, the desktop's by T2. */
export const LIBRARY_STORE = new InjectionToken<LibraryStore>('LibraryStore');
