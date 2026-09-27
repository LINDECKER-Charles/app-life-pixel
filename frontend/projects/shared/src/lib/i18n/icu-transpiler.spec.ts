import { TestBed } from '@angular/core/testing';
import { IcuTranspiler } from './icu-transpiler';

function transpiler(lang = 'en'): IcuTranspiler {
  TestBed.configureTestingModule({ providers: [IcuTranspiler] });
  const icu = TestBed.inject(IcuTranspiler);
  icu.onLangChanged(lang);
  return icu;
}

function format(icu: IcuTranspiler, value: string, params: Record<string, unknown> = {}): unknown {
  return icu.transpile({ value, params, translation: {}, key: 'test.key' });
}

describe('IcuTranspiler', () => {
  const frames = '{count, plural, =0 {No frame} one {# frame} other {# frames}}';

  it('formats plurals, exact values and # for the active language', () => {
    const icu = transpiler();
    expect(format(icu, frames, { count: 0 })).toBe('No frame');
    expect(format(icu, frames, { count: 1 })).toBe('1 frame');
    expect(format(icu, frames, { count: 1234 })).toBe('1,234 frames');

    icu.onLangChanged('fr');
    const images = '{count, plural, one {# image} other {# images}}';
    expect(format(icu, images, { count: 0 })).toBe('0 image');
    expect(format(icu, images, { count: 1234 })).toBe('1 234 images');
  });

  it('formats arguments, numbers, selects and ordinals', () => {
    const icu = transpiler();
    expect(format(icu, 'Hello {name}, {size, number} px', { name: 'Ada', size: 2048 })).toBe(
      'Hello Ada, 2,048 px',
    );
    expect(format(icu, '{ratio, number, percent}', { ratio: 0.5 })).toBe('50%');
    const who = '{kind, select, owner {Yours} other {Shared}}';
    expect(format(icu, who, { kind: 'owner' })).toBe('Yours');
    expect(format(icu, who, { kind: 'guest' })).toBe('Shared');
    const place = '{n, selectordinal, one {#st} two {#nd} few {#rd} other {#th}}';
    expect(format(icu, place, { n: 22 })).toBe('22nd');
  });

  it('keeps quoted text, and Transloco interpolation, as messageformat did', () => {
    const icu = transpiler();
    expect(format(icu, "Use '{braces}' and #")).toBe('Use {braces} and #');
    expect(
      format(icu, '{{ name }} has {n, plural, one {# layer} other {# layers}}', {
        name: 'Ada',
        n: 2,
      }),
    ).toBe('Ada has 2 layers');
  });

  it('returns a message that does not parse as it is', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
    expect(format(transpiler(), '{count, plural, one {x}')).toBe('{count, plural, one {x}');
    expect(console.error).toHaveBeenCalled();
  });
});
