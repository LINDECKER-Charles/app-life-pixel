import { computed, inject, Injectable, signal } from '@angular/core';
import { EDITOR_ENGINE } from './editor-engine';
import {
  isEngineError,
  type EditOperation,
  type EngineState,
  type NewAnimationOptions,
} from './engine-types';

const INITIAL_STATE: EngineState = {
  status: 'starting',
  document: null,
  canUndo: false,
  canRedo: false,
  hasUnsavedWork: false,
  limits: null,
};

/** A rejected `EngineError`, turned into something a page can show (editor.md, W0). */
export interface EngineNotification {
  readonly id: number;
  readonly key: string;
  readonly params: Readonly<Record<string, unknown>>;
}

/**
 * The engine's state as a signal, and its commands. It lives for the whole page (`providedIn:
 * 'root'`), so moving between routes never loses the work (editor.md, W0).
 */
@Injectable({ providedIn: 'root' })
export class EngineStore {
  private readonly engine = inject(EDITOR_ENGINE);
  private readonly stateSignal = signal<EngineState>(INITIAL_STATE);
  private readonly notificationsSignal = signal<readonly EngineNotification[]>([]);
  private nextNotificationId = 1;

  readonly state = this.stateSignal.asReadonly();
  readonly document = computed(() => this.stateSignal().document);
  readonly limits = computed(() => this.stateSignal().limits);
  readonly hasUnsavedWork = computed(() => this.stateSignal().hasUnsavedWork);
  readonly notifications = this.notificationsSignal.asReadonly();

  constructor() {
    this.engine.subscribe((state) => this.stateSignal.set(state));
  }

  async create(options: NewAnimationOptions): Promise<void> {
    await this.run(() => this.engine.create(options));
  }

  async open(document: Uint8Array): Promise<void> {
    await this.run(() => this.engine.open(document));
  }

  async apply(operation: EditOperation): Promise<void> {
    await this.run(() => this.engine.apply(operation));
  }

  async undo(): Promise<void> {
    await this.run(() => this.engine.undo());
  }

  async redo(): Promise<void> {
    await this.run(() => this.engine.redo());
  }

  async markSaved(): Promise<void> {
    await this.run(() => this.engine.markSaved());
  }

  dismissNotification(id: number): void {
    this.notificationsSignal.update((list) =>
      list.filter((notification) => notification.id !== id),
    );
  }

  private async run(command: () => Promise<void>): Promise<void> {
    try {
      await command();
    } catch (error) {
      this.notify(error);
    }
  }

  private notify(error: unknown): void {
    if (!isEngineError(error)) throw error;
    const notification = {
      id: this.nextNotificationId++,
      key: `errors.${error.code}`,
      params: error.params,
    };
    this.notificationsSignal.update((list) => [...list, notification]);
  }
}
