/**
 * The environment and the time range every view shares, kept in the URL —
 * `?env=production&from=now-24h&to=now` — so that any view is a link (admin-console.md).
 */
export interface ViewState {
  readonly env: string;
  readonly from: string;
  readonly to: string;
}

/** The query parameters of a view state, as the router takes them. */
export type ViewParams = Readonly<Record<'env' | 'from' | 'to', string>>;

export const DEFAULT_FROM = 'now-24h';
export const DEFAULT_TO = 'now';

/** The ranges the selector offers, each ending now. */
export const RANGE_PRESETS: readonly string[] = [
  'now-1h',
  'now-6h',
  'now-24h',
  'now-7d',
  'now-30d',
];

/**
 * `now`, `now-<n><unit>`, Unix seconds or an RFC 3339 date: the forms the admin server takes
 * (support-admin.md, H11). Anything else falls back to the default range.
 */
const TIME_PATTERN = /^(now(-\d+[smhdw])?|\d{1,12}|\d{4}-\d{2}-\d{2}T[\d:.]+(Z|[+-]\d{2}:\d{2}))$/;

export function isTime(value: string | null | undefined): value is string {
  return typeof value === 'string' && TIME_PATTERN.test(value);
}

/**
 * The environment the views start on: the one this console administers when its monitoring
 * shows it, otherwise the first it shows.
 */
export function defaultEnvironment(monitored: readonly string[], own: string): string {
  return monitored.includes(own) ? own : (monitored[0] ?? own);
}

/** Reads the state from the URL's query, each part falling back to its default. */
export function readViewState(
  query: Readonly<Record<string, string | null | undefined>>,
  monitored: readonly string[],
  own: string,
): ViewState {
  const env = query['env'];
  return {
    env: env && monitored.includes(env) ? env : defaultEnvironment(monitored, own),
    from: isTime(query['from']) ? query['from'] : DEFAULT_FROM,
    to: isTime(query['to']) ? query['to'] : DEFAULT_TO,
  };
}

export function viewParams(state: ViewState): ViewParams {
  return { env: state.env, from: state.from, to: state.to };
}
