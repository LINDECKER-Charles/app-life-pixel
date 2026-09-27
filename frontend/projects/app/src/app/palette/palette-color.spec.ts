import { isValidHex8, joinHex8, splitHex8 } from './palette-color';

describe('palette-color', () => {
  it('splits a valid #rrggbbaa colour into its rgb and alpha parts', () => {
    expect(splitHex8('#1a2b3cff')).toEqual({ rgb: '#1a2b3c', alpha: 255 });
    expect(splitHex8('#00000000')).toEqual({ rgb: '#000000', alpha: 0 });
  });

  it('falls back to opaque black for an invalid colour', () => {
    expect(splitHex8('not a colour')).toEqual({ rgb: '#000000', alpha: 255 });
    expect(splitHex8('#fff')).toEqual({ rgb: '#000000', alpha: 255 });
  });

  it('joins an rgb colour and an alpha into #rrggbbaa, clamping the alpha', () => {
    expect(joinHex8('#1a2b3c', 255)).toBe('#1a2b3cff');
    expect(joinHex8('#1a2b3c', 0)).toBe('#1a2b3c00');
    expect(joinHex8('#1a2b3c', 300)).toBe('#1a2b3cff');
    expect(joinHex8('#1a2b3c', -10)).toBe('#1a2b3c00');
  });

  it('round-trips split then join', () => {
    expect(joinHex8(splitHex8('#7f7f7fab').rgb, splitHex8('#7f7f7fab').alpha)).toBe('#7f7f7fab');
  });

  it('validates the #rrggbbaa shape', () => {
    expect(isValidHex8('#1a2b3cff')).toBe(true);
    expect(isValidHex8('#1A2B3CFF')).toBe(true);
    expect(isValidHex8('#1a2b3c')).toBe(false);
    expect(isValidHex8('1a2b3cff')).toBe(false);
    expect(isValidHex8('#gggggggg')).toBe(false);
  });
});
