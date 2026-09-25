import { transferables } from './protocol';

describe('transferables', () => {
  it('finds the buffer of every byte array, however deep, once', () => {
    const png = Uint8Array.from([1, 2]);
    const pixels = new Uint8ClampedArray(4);
    const message = {
      id: 1,
      value: { files: [{ bytes: png }, { bytes: png.subarray(1) }], frame: { pixels } },
    };

    const buffers = transferables(message);
    expect(buffers).toHaveLength(2);
    expect(buffers[0]).toBe(png.buffer);
    expect(buffers[1]).toBe(pixels.buffer);
  });

  it('finds nothing in a request without bytes', () => {
    expect(transferables([{ kind: 'setTitle', title: 'Plain' }])).toEqual([]);
  });
});
