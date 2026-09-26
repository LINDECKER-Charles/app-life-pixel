import { decodeBase64, encodeBase64 } from './base64';

/** The bytes of an ASCII text, in this realm's `Uint8Array` (`TextEncoder`'s is a different one). */
function bytes(text: string): Uint8Array {
  return Uint8Array.from(text, (character) => character.charCodeAt(0));
}

describe('base64', () => {
  it('round-trips arbitrary bytes, of every length modulo three', () => {
    for (const length of [0, 1, 2, 3, 4, 5, 6, 7]) {
      const data = Uint8Array.from({ length }, (_, index) => (index * 37 + 11) % 256);
      expect(decodeBase64(encodeBase64(data))).toEqual(data);
    }
  });

  it('encodes a known document as base64', () => {
    const text = bytes('Life Pixel');
    expect(encodeBase64(text)).toBe('TGlmZSBQaXhlbA==');
    expect(decodeBase64('TGlmZSBQaXhlbA==')).toEqual(text);
  });
});
