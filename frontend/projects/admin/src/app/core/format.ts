import type { PanelUnit } from '../monitoring/panels';
import type { Message } from './problem';

const BYTE_UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;
const KILO = 1000;
const MILLIS = 1000;
const NO_VALUE: Message = { key: 'admin.value.none', params: {} };

/** `1500000` bytes: "1.5 MB", in the language's own notation. */
export function formatBytes(bytes: number, language: string): string {
  let value = bytes;
  let unit = 0;
  while (Math.abs(value) >= KILO && unit < BYTE_UNITS.length - 1) {
    value /= KILO;
    unit += 1;
  }
  return new Intl.NumberFormat(language, {
    style: 'unit',
    unit: BYTE_UNITS[unit],
    maximumFractionDigits: 1,
  }).format(value);
}

/** A duration in seconds: milliseconds under a second, seconds above. */
export function formatSeconds(seconds: number, language: string): string {
  const inMillis = Math.abs(seconds) < 1;
  return new Intl.NumberFormat(language, {
    style: 'unit',
    unit: inMillis ? 'millisecond' : 'second',
    maximumFractionDigits: inMillis ? 0 : 2,
  }).format(inMillis ? seconds * MILLIS : seconds);
}

export function formatNumber(value: number, language: string, fractionDigits = 2): string {
  return new Intl.NumberFormat(language, { maximumFractionDigits: fractionDigits }).format(value);
}

function formatRatio(value: number, language: string): string {
  return new Intl.NumberFormat(language, { style: 'percent', maximumFractionDigits: 2 }).format(
    value,
  );
}

function plain(value: string): Message {
  return { key: 'admin.value.plain', params: { value } };
}

type Formatter = (value: number, language: string) => Message;

const FORMATTERS: Readonly<Record<PanelUnit, Formatter>> = {
  health: (value) => ({ key: value >= 1 ? 'admin.value.up' : 'admin.value.down', params: {} }),
  rate: (value, language) => ({
    key: 'admin.value.rate',
    params: { value: formatNumber(value, language) },
  }),
  cores: (value, language) => ({
    key: 'admin.value.cores',
    params: { value: formatNumber(value, language) },
  }),
  ratio: (value, language) => plain(formatRatio(value, language)),
  seconds: (value, language) => plain(formatSeconds(value, language)),
  bytes: (value, language) => plain(formatBytes(value, language)),
  count: (value, language) => plain(formatNumber(value, language, 0)),
};

/** A panel's value as a message: its unit's key, and the number in the language's notation. */
export function formatValue(
  value: number | null | undefined,
  unit: PanelUnit,
  language: string,
): Message {
  return value === null || value === undefined ? NO_VALUE : FORMATTERS[unit](value, language);
}

/** A date and time, as the language writes them. */
export function formatDateTime(iso: string | number, language: string): string {
  const date = typeof iso === 'number' ? new Date(iso * MILLIS) : new Date(iso);
  return new Intl.DateTimeFormat(language, { dateStyle: 'medium', timeStyle: 'short' }).format(
    date,
  );
}

/** A date, as the language writes it; in UTC for a date that is a whole day there. */
export function formatDate(iso: string, language: string, timeZone?: string): string {
  return new Intl.DateTimeFormat(language, { dateStyle: 'medium', timeZone }).format(new Date(iso));
}

/** An age in seconds, in its largest whole unit: "3 days", "5 hours", "12 minutes". */
export function formatAge(seconds: number, language: string): string {
  const units: readonly [Intl.RelativeTimeFormatUnit, number][] = [
    ['day', 86_400],
    ['hour', 3_600],
    ['minute', 60],
  ];
  const [unit, size] = units.find(([, length]) => seconds >= length) ?? ['minute', 60];
  return new Intl.NumberFormat(language, { style: 'unit', unit, unitDisplay: 'long' }).format(
    Math.floor(seconds / size),
  );
}
