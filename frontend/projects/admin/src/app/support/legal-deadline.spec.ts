import { legalDeadline, oneMonthAfter } from './legal-deadline';

const day = (iso: string): string => oneMonthAfter(new Date(iso)).toISOString();

describe('legal deadline', () => {
  it('falls one calendar month after the request', () => {
    expect(day('2026-09-26T10:00:00Z')).toBe('2026-10-26T10:00:00.000Z');
    expect(day('2026-12-15T00:00:00Z')).toBe('2027-01-15T00:00:00.000Z');
  });

  it('falls on the last day of a shorter month', () => {
    expect(day('2026-01-31T09:00:00Z')).toBe('2026-02-28T09:00:00.000Z');
    expect(day('2028-01-31T09:00:00Z')).toBe('2028-02-29T09:00:00.000Z');
    expect(day('2026-08-31T09:00:00Z')).toBe('2026-09-30T09:00:00.000Z');
  });

  it('counts the days left of an open data-protection request', () => {
    const request = {
      category: 'data_protection',
      status: 'new',
      createdAt: '2026-09-01T12:00:00Z',
    } as const;

    expect(legalDeadline(request, new Date('2026-09-26T12:00:00Z'))).toEqual({
      deadline: new Date('2026-10-01T12:00:00Z'),
      daysLeft: 5,
      overdue: false,
      settled: false,
    });
    expect(legalDeadline(request, new Date('2026-10-03T12:00:00Z'))).toMatchObject({
      daysLeft: -2,
      overdue: true,
    });
  });

  it('stops running once the request is resolved or closed', () => {
    const request = {
      category: 'data_protection',
      status: 'resolved',
      createdAt: '2026-08-01T12:00:00Z',
    } as const;

    expect(legalDeadline(request, new Date('2026-09-26T00:00:00Z'))).toMatchObject({
      overdue: false,
      settled: true,
    });
  });

  it('has none for another category', () => {
    expect(
      legalDeadline(
        { category: 'bug', status: 'new', createdAt: '2026-09-01T12:00:00Z' },
        new Date(),
      ),
    ).toBeNull();
  });
});
