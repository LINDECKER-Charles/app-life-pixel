import type { EngineError, EngineState } from '../engine-types';
import {
  transferables,
  type EngineArgs,
  type EngineMethod,
  type EngineRequest,
  type EngineValue,
  type WorkerMessage,
} from './protocol';

/** What a request that can no longer be answered rejects with. */
export const INTERNAL_ERROR: EngineError = { code: 'internal.error', params: {} };

/** What a connection tells the adapter. */
export interface ConnectionEvents {
  /** The worker pushed its state, after a command and before the command's answer. */
  state(state: EngineState): void;
  /**
   * The worker failed. `hadAnswered` says whether it ever answered a request: one that never
   * did — its module failed to load, or the snapshot it reopened failed it — would fail again.
   */
  failed(hadAnswered: boolean): void;
}

interface PendingRequest {
  resolve(value: unknown): void;
  reject(error: EngineError): void;
}

/**
 * One worker and the requests it has yet to answer: requests post their buffers transferred and
 * resolve by id. Once the worker fails, or is closed, every pending request rejects with
 * `internal.error` and later events are ignored.
 */
export class EngineConnection {
  private readonly pending = new Map<number, PendingRequest>();
  private nextId = 1;
  private hasAnswered = false;
  private isClosed = false;
  private readonly worker: Worker;
  private readonly events: ConnectionEvents;

  constructor(worker: Worker, events: ConnectionEvents) {
    this.worker = worker;
    this.events = events;
    worker.addEventListener('message', (event: MessageEvent<WorkerMessage>) =>
      this.receive(event.data),
    );
    worker.addEventListener('error', (event) => {
      event.preventDefault();
      this.fail();
    });
    worker.addEventListener('messageerror', () => this.fail());
  }

  request<M extends EngineMethod>(method: M, args: EngineArgs<M>): Promise<EngineValue<M>> {
    if (this.isClosed) return Promise.reject(INTERNAL_ERROR);
    const id = this.nextId++;
    const request = { id, method, args } as EngineRequest;
    return new Promise<EngineValue<M>>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (value: unknown) => void, reject });
      this.worker.postMessage(request, transferables(args));
    });
  }

  close(): void {
    this.isClosed = true;
    this.worker.terminate();
    for (const request of this.pending.values()) request.reject(INTERNAL_ERROR);
    this.pending.clear();
  }

  private receive(message: WorkerMessage): void {
    if (this.isClosed) return;
    if ('type' in message) {
      this.events.state(message.state);
      return;
    }
    const request = this.pending.get(message.id);
    if (!request) return;
    this.pending.delete(message.id);
    this.hasAnswered = true;
    if (message.ok) request.resolve(message.value);
    else request.reject(message.error);
  }

  private fail(): void {
    if (this.isClosed) return;
    this.close();
    this.events.failed(this.hasAnswered);
  }
}
