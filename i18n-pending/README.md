# Pending translations

Agents that work in parallel, each in its own worktree, never edit `i18n/*.json` in a commit:
two branches that both touch a catalogue would conflict on every merge. They write their keys
here instead, and the integration merges them.

| File | Holds |
|---|---|
| `i18n-pending/<lot>/en.json`, `fr.json` | the keys a lot adds or changes: flat, sorted, both languages |
| `i18n-pending/<lot>/removed.txt` | the keys a lot removes, one per line (`#` starts a comment) |

`<lot>` names the work, such as `lot3`. Reuse an existing key when the meaning is the same
(`design-system/docs/content.md`).

```shell
node frontend/tools/merge-i18n-fragments.mjs         # integration: merge, then delete the fragments
node frontend/tools/merge-i18n-fragments.mjs --keep  # a lot checking its work: merge, keep them
```

The tool applies the lots in name order: each lot's keys are added or replace existing ones,
then its `removed.txt` drops keys from every catalogue. It writes the catalogues sorted, in the
format `npm run i18n:check --prefix frontend` expects, and warns about a key two lots set
differently or a key missing from one language. Without any fragment it changes nothing.

After a `--keep` run, restore the catalogues before committing:
`git checkout -- i18n/en.json i18n/fr.json`.
