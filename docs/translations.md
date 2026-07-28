# README Translations

Tracks which commit of `README.md` each translation was produced from, so it's
easy to tell when a translation has drifted out of sync with the source.

| File | Language | Source commit | Notes |
|------|----------|----------------|-------|
| [`README.es.md`](../README.es.md) | Español | `c89f7186` | Predates the "VS Code Setup" and "Creating a New Contract" sections and the Directory Tree / Contract API Reference / Upgrade Guide resource links added to `README.md` since — **out of sync**, needs a refresh pass. |
| [`README.fr.md`](../README.fr.md) | Français | `bb87f4ad` | Full translation of `README.md` at this commit. |

## Updating a translation

1. Diff the translation's source commit against `README.md`'s current `HEAD`
   to see what changed: `git diff <source-commit> HEAD -- README.md`.
2. Apply the equivalent changes to the translated file.
3. Update this table's `Source commit` column to the new `HEAD` short hash
   (`git rev-parse --short HEAD`).
