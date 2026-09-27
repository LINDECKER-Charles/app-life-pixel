import { idScope } from './id-scope';

describe('idScope', () => {
  it('names each id after its component and part', () => {
    const id = idScope('new-animation');
    expect(id('width')).toMatch(/^new-animation-\d+-width$/);
  });

  it('gives each instance ids of its own, and the same id for the same part', () => {
    const first = idScope('animation-title');
    const second = idScope('animation-title');
    expect(first('field')).toBe(first('field'));
    expect(first('field')).not.toBe(second('field'));
  });
});
