import { ApiProblem } from 'shared';
import { vi } from 'vitest';
import { Page, PagedList } from './paged-list';

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((settle) => (resolve = settle));
  return { promise, resolve };
}

describe('PagedList', () => {
  it('reads the first page, then appends the next by cursor', async () => {
    const read = vi
      .fn()
      .mockResolvedValueOnce({ items: [1, 2], nextCursor: 'c1' })
      .mockResolvedValueOnce({ items: [3], nextCursor: null });
    const list = new PagedList<number, string>(read);

    await list.reload('lee');
    expect(list.items()).toEqual([1, 2]);
    expect(list.hasMore()).toBe(true);

    await list.more();
    expect(read).toHaveBeenLastCalledWith('lee', 'c1');
    expect(list.items()).toEqual([1, 2, 3]);
    expect(list.hasMore()).toBe(false);
  });

  it('drops the answer to an older query', async () => {
    const first = deferred<Page<number>>();
    const read = vi
      .fn()
      .mockReturnValueOnce(first.promise)
      .mockResolvedValueOnce({ items: [9], nextCursor: null });
    const list = new PagedList<number, string>(read);

    const older = list.reload('a');
    await list.reload('b');
    first.resolve({ items: [1], nextCursor: 'x' });
    await older;

    expect(list.items()).toEqual([9]);
    expect(list.hasMore()).toBe(false);
  });

  it('keeps the problem of a failed read, and empties the list', async () => {
    const read = vi
      .fn()
      .mockResolvedValueOnce({ items: [1], nextCursor: null })
      .mockRejectedValueOnce(new ApiProblem(503, 'service.unavailable'));
    const list = new PagedList<number, string>(read);
    await list.reload('a');

    await list.reload('b');

    expect(list.items()).toEqual([]);
    expect(list.problem()?.code).toBe('service.unavailable');
    expect(list.loading()).toBe(false);
  });
});
