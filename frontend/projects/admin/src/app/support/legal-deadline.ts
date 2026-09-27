import type { SupportRequest } from '../core/admin-types';

const DAY_MS = 86_400_000;

/**
 * One calendar month after `created`, in UTC: 26 September gives 26 October, and a day the next
 * month lacks gives its last day — 31 January gives 28 or 29 February.
 */
export function oneMonthAfter(created: Date): Date {
  const deadline = new Date(created.getTime());
  const day = created.getUTCDate();
  deadline.setUTCDate(1);
  deadline.setUTCMonth(deadline.getUTCMonth() + 1);
  const lastDay = new Date(
    Date.UTC(deadline.getUTCFullYear(), deadline.getUTCMonth() + 1, 0),
  ).getUTCDate();
  deadline.setUTCDate(Math.min(day, lastDay));
  return deadline;
}

/** Where a data-protection request stands against its legal deadline. */
export interface LegalDeadline {
  readonly deadline: Date;
  /** Whole days left, negative once passed. */
  readonly daysLeft: number;
  /** Passed while the request is still open. */
  readonly overdue: boolean;
  /** Resolved or closed: the deadline no longer runs. */
  readonly settled: boolean;
}

/**
 * The legal deadline of a `data_protection` request — one month after its creation — or `null`
 * for any other category.
 */
export function legalDeadline(
  request: Pick<SupportRequest, 'category' | 'createdAt' | 'status'>,
  now: Date,
): LegalDeadline | null {
  if (request.category !== 'data_protection') {
    return null;
  }
  const settled = request.status === 'resolved' || request.status === 'closed';
  const deadline = oneMonthAfter(new Date(request.createdAt));
  const daysLeft = Math.floor((deadline.getTime() - now.getTime()) / DAY_MS);
  return { deadline, daysLeft, overdue: !settled && daysLeft < 0, settled };
}
