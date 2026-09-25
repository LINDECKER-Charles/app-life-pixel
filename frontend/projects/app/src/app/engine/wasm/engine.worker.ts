/*
 * The engine's Web Worker (editor.md, W1): it loads editor-wasm, owns one `EngineCore`, answers
 * requests in order, and pushes the state after each command. A refusal is an answer; anything
 * else thrown — a trap, a request it cannot read — is left uncaught, so that the page sees the
 * worker fail and starts another one.
 */
import { isEngineError, type EngineState, type RenderedFrame } from '../engine-types';
import init, { EngineCore } from './generated/editor_engine';
import {
  COMMANDS,
  transferables,
  type EngineArgs,
  type EngineMethod,
  type EngineRequest,
  type WorkerMessage,
} from './protocol';

/** Where the app serves the module `cargo xtask build-editor` writes to public/engine/. */
const MODULE_URL = '/engine/editor_engine_bg.wasm';

/** The part of a dedicated worker's scope used here: the app compiles against the DOM's lib. */
interface WorkerScope {
  addEventListener(type: 'message', listener: (event: MessageEvent<EngineRequest>) => void): void;
  postMessage(message: WorkerMessage, transfer: Transferable[]): void;
}

type Handlers = {
  readonly [M in EngineMethod]: (core: EngineCore, ...args: EngineArgs<M>) => unknown;
};

/** One call of `EngineCore` per method: the bindings type what they take and give as unknown. */
const HANDLERS: Handlers = {
  create: (core, options) => core.create(options),
  open: (core, document) => core.open(document),
  restore: (core, document) => core.restore(document),
  apply: (core, operation) => core.apply(operation),
  undo: (core) => core.undo(),
  redo: (core) => core.redo(),
  markSaved: (core) => core.markSaved(),
  state: (core) => core.state(),
  render: (core, request) => clamped(core.render(request) as RawFrame),
  serialize: (core) => core.serialize(),
  export: (core, request) => core.export(request),
  snippet: (core, request) => core.snippet(request),
};

/** A frame as the bindings give it: bytes are a `Uint8Array`. */
type RawFrame = Omit<RenderedFrame, 'pixels'> & { readonly pixels: Uint8Array };

const scope = self as unknown as WorkerScope;
const waiting: EngineRequest[] = [];
let engine: EngineCore | undefined;
/** Set by the first failure: the page is replacing this worker, which answers nothing more. */
let hasFailed = false;

scope.addEventListener('message', ({ data }) => {
  if (hasFailed) return;
  if (engine) answer(engine, data);
  else waiting.push(data);
});
load().catch(reportError);

/** Loads the module, then answers the requests that came before it. */
async function load(): Promise<void> {
  await init({ module_or_path: MODULE_URL });
  const core = new EngineCore();
  engine = core;
  for (const request of waiting.splice(0)) answer(core, request);
}

function answer(core: EngineCore, request: EngineRequest): void {
  let value: unknown;
  try {
    value = call(core, request);
  } catch (error) {
    if (!isEngineError(error)) {
      hasFailed = true;
      throw error;
    }
    post({ id: request.id, ok: false, error });
    return;
  }
  if (COMMANDS.has(request.method)) post({ type: 'state', state: core.state() as EngineState });
  post({ id: request.id, ok: true, value });
}

function call(core: EngineCore, request: EngineRequest): unknown {
  if (!Object.hasOwn(HANDLERS, request.method)) {
    throw new Error(`the engine has no method ${String(request.method)}`);
  }
  const handler = HANDLERS[request.method] as (core: EngineCore, ...args: unknown[]) => unknown;
  return handler(core, ...request.args);
}

/** The pixels as `RenderedFrame` types them, over the same bytes. */
function clamped(frame: RawFrame): RenderedFrame {
  const { buffer, byteOffset, byteLength } = frame.pixels;
  return { ...frame, pixels: new Uint8ClampedArray(buffer, byteOffset, byteLength) };
}

function post(message: WorkerMessage): void {
  scope.postMessage(message, transferables(message));
}
