/** T1's commands carry documents and files as base64 strings (desktop.md, T1). */
const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';

/** `bytes`, base64-encoded. */
export function encodeBase64(bytes: Uint8Array): string {
  let text = '';
  for (let index = 0; index < bytes.length; index += 3) {
    const first = bytes[index];
    const second = bytes[index + 1];
    const third = bytes[index + 2];
    text += ALPHABET[first >> 2];
    text += ALPHABET[((first & 0x03) << 4) | (second === undefined ? 0 : second >> 4)];
    text +=
      second === undefined
        ? '='
        : ALPHABET[((second & 0x0f) << 2) | (third === undefined ? 0 : third >> 6)];
    text += third === undefined ? '=' : ALPHABET[third & 0x3f];
  }
  return text;
}

/** The bytes `text` (base64) encodes. */
export function decodeBase64(text: string): Uint8Array {
  const bytes: number[] = [];
  let buffer = 0;
  let bits = 0;
  for (const character of text) {
    if (character === '=') break;
    const value = ALPHABET.indexOf(character);
    if (value === -1) continue;
    buffer = (buffer << 6) | value;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes.push((buffer >> bits) & 0xff);
    }
  }
  return Uint8Array.from(bytes);
}
