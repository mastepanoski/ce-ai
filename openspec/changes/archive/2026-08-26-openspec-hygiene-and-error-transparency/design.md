# Design: `openspec-hygiene-and-error-transparency`

## Ledger

```
openspec/changes/
├── archive/
│   ├── README.md            # convention + triage table of active folders
│   └── <change_name>/       # moved via git mv; history preserved
└── <active folders>         # only those with unverified open work
```

Entry criteria to archive: (a) zero open tasks, or (b) STATUS header on
`tasks.md` citing feature-level ship evidence; open boxes under (b) are
declared unaudited, never silently ticked.

## Reporters (`src/state/mod.rs`, next to `write_atomic`)

```rust
pub(crate) fn report_best_effort_remove(
    path: impl AsRef<Path>, res: std::io::Result<()>
) -> bool   // true = warned

pub(crate) fn report_best_effort_write(
    path: impl AsRef<Path>, res: Result<(), CeError>
) -> bool
```

| Outcome | remove | write |
| --- | --- | --- |
| Ok | silent | silent |
| NotFound | silent | n/a (warns like any error) |
| other Err | stderr warning naming path | stderr warning naming path |

Both honor no quiet flag deliberately: they are warnings, not output;
commands that own a quiet policy wrap them if ever needed.

## Call-site map

| Site | Conversion |
| --- | --- |
| deinit_prj ×18, init_prj ×3 | reporters |
| install tmp ×2, upgrade tmp ×1 | `report_best_effort_remove` |
| install/sync registry sync | explicit `warning:` honoring `ctx.quiet` |
| sync ctrl-c handler | explicit `warning:` then continue |
| uninstall custom roots ×3 | keep silent + justification comment (`DirectoryNotEmpty` expected) |
| uninstall `let _ = parse()?` | drop discard |

## Testing

One unit test per helper asserting the returned bool for ok/notfound/error;
CLI suites must remain green unchanged (warnings never change exit codes).
