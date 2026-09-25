/** The name of the custom section that carries an export's payload. */
const PAYLOAD_SECTION = 'life-pixel';

/** A compiled export: the player module and the payload appended to it. */
export interface CompiledExport {
  readonly module: WebAssembly.Module;
  readonly payload: ArrayBuffer;
}

const compiledExports = new Map<string, Promise<CompiledExport>>();

/** Compiles the export at an absolute URL once, however many elements play it. */
export function loadExport(url: string): Promise<CompiledExport> {
  let compiled = compiledExports.get(url);
  if (!compiled) {
    compiled = compile(url).then(withPayload);
    compiledExports.set(url, compiled);
    // A failure is not kept: the next element that asks for the URL tries again.
    compiled.catch(() => compiledExports.delete(url));
  }
  return compiled;
}

async function compile(url: string): Promise<WebAssembly.Module> {
  try {
    return await WebAssembly.compileStreaming(fetch(url));
  } catch (error) {
    // A server that does not send `application/wasm` makes streaming fail with a TypeError.
    if (!(error instanceof TypeError)) throw error;
    const response = await fetch(url);
    return WebAssembly.compile(await response.arrayBuffer());
  }
}

function withPayload(module: WebAssembly.Module): CompiledExport {
  const payload = WebAssembly.Module.customSections(module, PAYLOAD_SECTION)[0];
  if (!payload) throw Error('no ' + PAYLOAD_SECTION + ' section');
  return { module, payload };
}
