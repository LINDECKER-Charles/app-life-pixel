/**
 * The ports of the server and the admin server (docs/v1/README.md, "Ports"): the suite starts
 * both on them, after checking that they are free.
 */
export const PORTS = {
  app: 8460,
  metrics: 8461,
  adminApi: 8462,
  console: 8463,
  consoleMetrics: 8464,
} as const;

/** The app and its API, as the server serves them and its emails link to them. */
export const HOSTED_APP_URL = `http://localhost:${PORTS.app}`;
/** The admin console, as the admin server serves it. */
export const ADMIN_CONSOLE_URL = `http://localhost:${PORTS.console}`;
