# Tasks: `transactional-ops`

- [ ] **T1.1** `src/state/journal.rs`: schema, begin/rollback, tracked_write
      with fault injection, complete; unit tests (round-trip, corrupt-as-absent,
      reverse rollback incl. delete-created).
- [ ] **T1.2** doctor finding for present journal.
- [ ] **T2.1** install.rs tracked writes + complete() after final state save.
- [ ] **T2.2** sync_with tracked writes + completion.
- [ ] **T3.1** Fault-injection CLI tests: early cut, late cut, doctor flag,
      recovery run preserves user content.
- [ ] **T4.1** Gates: fmt / clippy -D warnings / cargo test / make e2e.
- [ ] **T5.1** Bump v1.22.0 + CHANGELOG.
