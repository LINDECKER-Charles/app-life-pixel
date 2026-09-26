import { expect } from '@playwright/test';

/** Mailpit's web and API port on the local stack (docs/v1/README.md, "Ports"). */
const MAILPIT_URL = process.env['LP_TEST_MAILPIT_URL'] ?? 'http://127.0.0.1:5463';
/** How long an email gets to reach Mailpit: the server sends it in the background. */
const DELIVERY_TIMEOUT_MS = 30_000;

/** An email as Mailpit's API gives it. */
export interface MailMessage {
  readonly ID: string;
  readonly Subject: string;
  readonly Text: string;
}

interface Search {
  readonly messages: readonly { readonly ID: string; readonly Subject: string }[];
}

async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(`${MAILPIT_URL}${path}`);
  if (!response.ok) throw new Error(`Mailpit answered ${response.status} to ${path}`);
  return (await response.json()) as T;
}

/**
 * The last message sent to `recipient` whose subject is `subject`, waiting for it to arrive;
 * Mailpit lists a search's messages newest first.
 */
export async function lastMessage(recipient: string, subject: string): Promise<MailMessage> {
  const query = encodeURIComponent(`to:"${recipient}" subject:"${subject}"`);
  let id: string | undefined;
  await expect
    .poll(
      async () => {
        const search = await getJson<Search>(`/api/v1/search?query=${query}`);
        id = search.messages.find((message) => message.Subject === subject)?.ID;
        return id;
      },
      { message: `an email "${subject}" to ${recipient}`, timeout: DELIVERY_TIMEOUT_MS },
    )
    .toBeDefined();
  return getJson<MailMessage>(`/api/v1/message/${id}`);
}

/** The link of `message`'s text that starts with `origin`: the one a person would open. */
export function linkIn(message: MailMessage, origin: string): URL {
  const link = message.Text.split(/\s+/).find((word) => word.startsWith(`${origin}/`));
  if (!link) throw new Error(`"${message.Subject}" holds no link to ${origin}.`);
  return new URL(link);
}
