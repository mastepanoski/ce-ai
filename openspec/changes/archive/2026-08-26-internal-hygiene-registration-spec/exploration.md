# Exploration

- Cursor regression found while unifying the tables: install never copied
  skills for cursor (correct), but the v1.19.2 sync table said skills="skills".
  Verification matrix excluded cursor from hashing, so the pollution was
  invisible to gates — fixed by making the spec truthful (None) instead of
  widening verification.
- Dead items verified against src+tests+benches references before deletion;
  clippy --all-targets is the oracle.
- .ce-ai.json override loading existed (load_with_workspace_overrides +
  merge_overrides) with zero production callers despite CHANGELOG/CONCEPTS
  documentation — wired at the reader level rather than deleted, because the
  feature is documented product behavior.
