import { formatAge, formatBytes, formatSeconds, formatValue } from './format';

describe('format', () => {
  it('writes each unit of a panel in the language’s notation', () => {
    expect(formatValue(1, 'health', 'en')).toEqual({ key: 'admin.value.up', params: {} });
    expect(formatValue(0, 'health', 'en')).toEqual({ key: 'admin.value.down', params: {} });
    expect(formatValue(0.125, 'ratio', 'en').params).toEqual({ value: '12.5%' });
    expect(formatValue(12.345, 'rate', 'en')).toEqual({
      key: 'admin.value.rate',
      params: { value: '12.35' },
    });
    expect(formatValue(1234, 'count', 'en').params).toEqual({ value: '1,234' });
    expect(formatValue(null, 'bytes', 'en')).toEqual({ key: 'admin.value.none', params: {} });
  });

  it('writes bytes in their largest unit', () => {
    expect(formatBytes(512, 'en')).toBe('512 byte');
    expect(formatBytes(1_500_000, 'en')).toBe('1.5 MB');
  });

  it('writes a duration in milliseconds under a second', () => {
    expect(formatSeconds(0.25, 'en')).toBe('250 ms');
    expect(formatSeconds(1.5, 'en')).toBe('1.5 sec');
  });

  it('writes an age in its largest whole unit', () => {
    expect(formatAge(3 * 86_400 + 5, 'en')).toBe('3 days');
    expect(formatAge(7_200, 'en')).toBe('2 hours');
    expect(formatAge(30, 'en')).toBe('0 minutes');
  });
});
