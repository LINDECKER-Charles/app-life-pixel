import { FunctionArg, Select, SelectCase, Token } from '@messageformat/parser';

/** The values an ICU message reads its arguments from. */
export type IcuParams = Readonly<Record<string, unknown>>;

interface Scope {
  readonly locale: string;
  readonly params: IcuParams;
  /** The value `#` stands for, inside a plural. */
  readonly count?: number;
}

type DateStyle = Intl.DateTimeFormatOptions['dateStyle'];

const NUMBER_STYLES: Readonly<Record<string, Intl.NumberFormatOptions>> = {
  integer: { maximumFractionDigits: 0 },
  percent: { style: 'percent' },
};
const DATE_STYLES: readonly string[] = ['full', 'long', 'medium', 'short'];

/**
 * Formats a parsed ICU message by walking its tokens with `Intl`: unlike
 * `@messageformat/core`, which compiles each message into a function, it never evaluates code,
 * so it runs under the desktop app's CSP, which forbids `eval`. It covers what the catalogues
 * use — arguments, `number`, `date` and `time`, `plural`, `selectordinal` and `select`.
 */
export function formatIcu(tokens: readonly Token[], params: IcuParams, locale: string): string {
  return formatTokens(tokens, { locale, params });
}

function formatTokens(tokens: readonly Token[], scope: Scope): string {
  return tokens.map((token) => formatToken(token, scope)).join('');
}

function formatToken(token: Token, scope: Scope): string {
  switch (token.type) {
    case 'content':
      return token.value;
    case 'argument':
      return String(scope.params[token.arg] ?? '');
    case 'function':
      return formatFunction(token, scope);
    case 'octothorpe':
      return scope.count === undefined ? '#' : formatNumber(scope.count, scope.locale);
    case 'select':
      return formatTokens(pickCase(token.cases, String(scope.params[token.arg])), scope);
    default:
      return formatPlural(token, scope);
  }
}

function formatPlural(token: Select, scope: Scope): string {
  const value = Number(scope.params[token.arg]);
  const count = value - (token.pluralOffset ?? 0);
  const type = token.type === 'selectordinal' ? 'ordinal' : 'cardinal';
  const exact = token.cases.find((option) => option.key === `=${value}`);
  const category = new Intl.PluralRules(scope.locale, { type }).select(count);
  return formatTokens(exact?.tokens ?? pickCase(token.cases, category), { ...scope, count });
}

function pickCase(cases: readonly SelectCase[], key: string): readonly Token[] {
  const option =
    cases.find((each) => each.key === key) ?? cases.find((each) => each.key === 'other');
  return option?.tokens ?? [];
}

function formatFunction(token: FunctionArg, scope: Scope): string {
  const value = scope.params[token.arg];
  const style = (token.param ?? [])
    .map((part) => (part.type === 'content' ? part.value : ''))
    .join('')
    .trim();
  switch (token.key) {
    case 'number':
      return formatNumber(Number(value), scope.locale, NUMBER_STYLES[style]);
    case 'date':
      return new Intl.DateTimeFormat(scope.locale, { dateStyle: dateStyle(style) }).format(
        toDate(value),
      );
    case 'time':
      return new Intl.DateTimeFormat(scope.locale, { timeStyle: dateStyle(style) }).format(
        toDate(value),
      );
    default:
      return String(value ?? '');
  }
}

function formatNumber(value: number, locale: string, options?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(locale, options).format(value);
}

function dateStyle(style: string): DateStyle {
  return DATE_STYLES.includes(style) ? (style as DateStyle) : 'medium';
}

function toDate(value: unknown): Date {
  return value instanceof Date ? value : new Date(value as string | number);
}
