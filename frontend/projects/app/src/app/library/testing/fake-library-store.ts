import type { LibraryStore } from '../library-store';
import type {
  AnimationFilter,
  AnimationSummary,
  LibraryFailure,
  Page,
  PageQuery,
  Project,
} from '../library-types';

const DEFAULT_LIMIT = 50;
const TIMESTAMP = '2024-01-01T00:00:00.000Z';

function notFound(kind: 'animation' | 'project'): LibraryFailure {
  return { code: `library.${kind}_not_found`, params: {} };
}

/** `items` from `page.cursor` — an index here —, `page.limit` of them. */
function pageOf<T>(items: readonly T[], page: PageQuery = {}): Page<T> {
  const start = Number(page.cursor ?? 0);
  const end = start + (page.limit ?? DEFAULT_LIMIT);
  return { items: items.slice(start, end), nextCursor: end < items.length ? String(end) : null };
}

/**
 * An in-memory `LibraryStore` for the tests: projects and animations in insertion order, cursors
 * as indexes, and a queue of failures the next calls reject with. Its documents are kept as
 * given, and `saveDocument` checks the version as the server does.
 */
export class FakeLibraryStore implements LibraryStore {
  readonly projects: Project[] = [];
  readonly animations: AnimationSummary[] = [];
  readonly documents = new Map<string, Uint8Array>();
  usedBytes = 0;
  limitBytes: number | null = 1_000_000;
  private readonly failures: LibraryFailure[] = [];
  private nextId = 1;

  /** Makes the next call reject with `failure`. */
  failNext(code: string, params: Record<string, unknown> = {}): void {
    this.failures.push({ code, params });
  }

  addProject(name: string): Project {
    const project = { id: this.newId('p'), name, animationCount: 0, ...this.times() };
    this.projects.push(project);
    return project;
  }

  addAnimation(projectId: string, title: string, document: Uint8Array = new Uint8Array([1])) {
    const animation: AnimationSummary = {
      id: this.newId('a'),
      projectId,
      title,
      width: 8,
      height: 8,
      frameCount: 1,
      documentBytes: document.byteLength,
      version: 1,
      ...this.times(),
    };
    this.animations.push(animation);
    this.documents.set(animation.id, document);
    return animation;
  }

  async listProjects(page?: PageQuery): Promise<Page<Project>> {
    this.failIfAsked();
    return pageOf(this.projects, page);
  }

  async createProject(name: string): Promise<Project> {
    this.failIfAsked();
    return this.addProject(name);
  }

  async renameProject(id: string, name: string): Promise<Project> {
    return this.updateProject(id, { name });
  }

  async duplicateProject(id: string, name: string): Promise<Project> {
    this.findProject(id);
    return this.addProject(name);
  }

  async deleteProject(id: string): Promise<void> {
    this.projects.splice(this.projects.indexOf(this.findProject(id)), 1);
  }

  async listAnimations(filter: AnimationFilter, page?: PageQuery): Promise<Page<AnimationSummary>> {
    this.failIfAsked();
    const matching = this.animations.filter(
      (animation) =>
        (filter.projectId === undefined || animation.projectId === filter.projectId) &&
        animation.title.toLowerCase().includes((filter.query ?? '').toLowerCase()),
    );
    return pageOf(matching, page);
  }

  async createAnimation(projectId: string, document: Uint8Array): Promise<AnimationSummary> {
    this.findProject(projectId);
    return this.addAnimation(projectId, 'Saved', document);
  }

  async openDocument(id: string): Promise<{ summary: AnimationSummary; document: Uint8Array }> {
    const summary = this.findAnimation(id);
    return { summary, document: this.documents.get(id) ?? new Uint8Array() };
  }

  async saveDocument(id: string, document: Uint8Array, version: number): Promise<AnimationSummary> {
    const current = this.findAnimation(id);
    if (current.version !== version) {
      throw { code: 'document.version_conflict', params: { current: current.version } };
    }
    this.documents.set(id, document);
    return this.updateAnimation(id, { version: version + 1 });
  }

  async renameAnimation(id: string, title: string, version: number): Promise<AnimationSummary> {
    const current = this.findAnimation(id);
    if (current.version !== version) {
      throw { code: 'document.version_conflict', params: { current: current.version } };
    }
    return this.updateAnimation(id, { title, version: version + 1 });
  }

  async moveAnimation(id: string, projectId: string): Promise<AnimationSummary> {
    this.findProject(projectId);
    return this.updateAnimation(id, { projectId });
  }

  async duplicateAnimation(
    id: string,
    title: string,
    projectId?: string,
  ): Promise<AnimationSummary> {
    const original = this.findAnimation(id);
    return this.addAnimation(projectId ?? original.projectId, title, this.documents.get(id));
  }

  async deleteAnimation(id: string): Promise<void> {
    this.animations.splice(this.animations.indexOf(this.findAnimation(id)), 1);
  }

  async usage(): Promise<{ usedBytes: number; limitBytes: number | null }> {
    this.failIfAsked();
    return { usedBytes: this.usedBytes, limitBytes: this.limitBytes };
  }

  private updateProject(id: string, change: Partial<Project>): Project {
    const index = this.projects.indexOf(this.findProject(id));
    const updated = { ...this.projects[index], ...change } as Project;
    this.projects[index] = updated;
    return updated;
  }

  private updateAnimation(id: string, change: Partial<AnimationSummary>): AnimationSummary {
    const index = this.animations.indexOf(this.findAnimation(id));
    const updated = { ...this.animations[index], ...change } as AnimationSummary;
    this.animations[index] = updated;
    return updated;
  }

  private findProject(id: string): Project {
    this.failIfAsked();
    const project = this.projects.find((each) => each.id === id);
    if (!project) throw notFound('project');
    return project;
  }

  private findAnimation(id: string): AnimationSummary {
    this.failIfAsked();
    const animation = this.animations.find((each) => each.id === id);
    if (!animation) throw notFound('animation');
    return animation;
  }

  private failIfAsked(): void {
    const failure = this.failures.shift();
    if (failure) throw failure;
  }

  private newId(prefix: string): string {
    return `${prefix}${this.nextId++}`;
  }

  private times(): { createdAt: string; updatedAt: string } {
    return { createdAt: TIMESTAMP, updatedAt: TIMESTAMP };
  }
}
