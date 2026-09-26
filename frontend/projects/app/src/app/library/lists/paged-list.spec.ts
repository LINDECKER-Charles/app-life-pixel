import { FakeLibraryStore } from '../testing/fake-library-store';
import { PAGE_SIZE, PagedList } from './paged-list';

function libraryOf(count: number): FakeLibraryStore {
  const store = new FakeLibraryStore();
  for (let index = 0; index < count; index += 1) store.addProject(`Project ${index}`);
  return store;
}

describe('PagedList', () => {
  it('reads 50 items at a time, then the next 50 on "Load more"', async () => {
    const store = libraryOf(120);
    const list = new PagedList((page) => store.listProjects(page));

    await list.reload();
    expect(list.items()).toHaveLength(PAGE_SIZE);
    expect(list.hasMore()).toBe(true);

    await list.loadMore();
    expect(list.items()).toHaveLength(2 * PAGE_SIZE);

    await list.loadMore();
    expect(list.items()).toHaveLength(120);
    expect(list.hasMore()).toBe(false);
    expect(list.items().map(({ name }) => name)).toEqual(store.projects.map(({ name }) => name));
  });

  it('starts over on reload', async () => {
    const store = libraryOf(60);
    const list = new PagedList((page) => store.listProjects(page));
    await list.reload();
    await list.loadMore();

    await list.reload();

    expect(list.items()).toHaveLength(PAGE_SIZE);
  });

  it('keeps what it read and records the failure of a page', async () => {
    const store = libraryOf(60);
    const list = new PagedList((page) => store.listProjects(page));
    await list.reload();
    store.failNext('service.unavailable');

    await list.loadMore();

    expect(list.items()).toHaveLength(PAGE_SIZE);
    expect(list.failure()).toEqual({ code: 'service.unavailable', params: {} });
    expect(list.hasMore()).toBe(true);
  });

  it('follows the changes made through the store', async () => {
    const store = libraryOf(2);
    const list = new PagedList((page) => store.listProjects(page));
    await list.reload();
    const [first, second] = store.projects;
    if (!first || !second) throw new Error('no projects');

    list.replace({ ...second, name: 'Renamed' });
    list.remove(first.id);
    list.prepend({ ...first, id: 'new', name: 'New' });

    expect(list.items().map(({ name }) => name)).toEqual(['New', 'Renamed']);
  });
});
