# Design
- CeError gains State(String), Network(String); exit_code(): Usage 2,
  Runtime|Json 1, State 3, Io 4, Network 5, Verification 6.
- State::load maps parse+non-NotFound io to State(path context); save maps
  write_atomic failures to State. NotFound still yields default State.
- install/upgrade tarball send/bytes failures map to Network.
- Makefile: `bench` target; benches derive version via env!("CARGO_PKG_VERSION").
