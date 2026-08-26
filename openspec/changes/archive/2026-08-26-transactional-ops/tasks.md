# Tasks: `transactional-ops`

- [x] **T1.1** `src/state/journal.rs`: schema, begin/rollback, tracked_write
      with fault injection, complete; unit tests (round-trip, corrupt-as-absent,
      reverse rollback incl. delete-created).
- [x] **T1.2** doctor finding for present journal.
- [x] **T2.1** install.rs tracked writes + complete() after final state save.
- [x] **T2.2** sync_with tracked writes + completion.
- [x] **T3.1** Fault-injection CLI tests: early cut, late cut, doctor flag,
      recovery run preserves user content.
- [x] **T4.1** Gates: fmt / clippy -D warnings / cargo test / make e2e.
- [x] **T5.1** Bump v1.22.0 + CHANGELOG.
