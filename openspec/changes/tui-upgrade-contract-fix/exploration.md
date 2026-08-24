# Exploration
Sweep of all 9 capture_cli sites found only upgrade drifted; others remain on
valid surfaces. Validation models top-level Cli globals (--dry-run) via a
with_cli_globals helper, since module Args augmented standalone do not carry
them. clap consumes argv[0] as bin name, so verb-stripped vectors are padded
with a dummy program name before matching.
