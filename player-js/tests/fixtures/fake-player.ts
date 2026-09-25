import { readFile } from 'node:fs/promises';
import wabt from 'wabt';

const SOURCE = new URL('./fake-player.wat', import.meta.url);
const ABI_VERSION_BODY = '(func (export "abi_version") (result i32)\n    (i32.const 1))';

/** Compiles `fake-player.wat`, answering `abi_version` with the version given. */
export async function compileFakePlayer(abiVersion = 1): Promise<Uint8Array> {
  const source = await readFile(SOURCE, 'utf8');
  if (!source.includes(ABI_VERSION_BODY)) throw Error('fake-player.wat: abi_version not found');
  const text = source.replace(ABI_VERSION_BODY, ABI_VERSION_BODY.replace('1)', `${abiVersion})`));
  const module = (await wabt()).parseWat('fake-player.wat', text);
  try {
    module.validate();
    return module.toBinary({}).buffer;
  } finally {
    module.destroy();
  }
}
