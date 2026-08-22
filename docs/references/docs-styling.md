# Documentation Style Guide

This guide defines how documentation in `ce-ai` is structured and written. It exists so that any contributor — human or AI agent — produces docs that serve both newcomers (who need to *learn*) and senior users (who need to *look things up*).

Governing standards:

- [**Diátaxis**](https://diataxis.fr/): the four-quadrant framework for technical documentation.
- [**Keep a Changelog**](https://keepachangelog.com/en/1.0.0/): already adopted for `CHANGELOG.md`.
- **Cognitive-load principles**: lead with the answer, progressive disclosure, chunking, signposting, recognition over recall.

> Note: ISO 20700 governs management consultancy services, not technical writing. Do not cite it as a documentation standard.

---

## 1. The Four Quadrants (Diátaxis)

Every piece of documentation must have exactly one primary intent:

| Quadrant | Reader's goal | Form | Examples in this repo |
| --- | --- | --- | --- |
| **Tutorial** | Learn by doing | Guided lesson, guaranteed to work | `docs/user-guide/quick-start-workflow-guide.md` |
| **How-to** | Accomplish a task | Steps for a specific goal | `docs/user-guide/sync-and-upgrade-mechanisms.md` |
| **Reference** | Look up a fact | Accurate, complete, dry | Harness Matrix, CLI flags, exit codes |
| **Explanation** | Understand | Discussion of why and how | Architectural guide, FSM masterclass |

Never blend quadrants inside one document section. A reference table does not belong mid-tutorial; an architecture discussion does not belong inside installation steps.

## 2. README Contract

`README.md` is the front door, not the house. Target: **≤ 100 lines**.

Required structure, in order:

1. **Title + what & why** (max 3 lines): what `ce-ai` is, who it helps, why it matters.
2. **Quick Path**: the shortest verified route to value — install binary → first command → verification command (e.g., `install` → `ce-ai install --all --dry-run` → `ce-ai doctor`). Nothing else before this.
3. **Documentation map**: a table linking into `docs/`, each row labeled with audience (`Beginner` / `Senior`) or quadrant.
4. **Minimal pointers**: security/compliance links one line each; no duplicated internals.

Forbidden in the README: deep internal mechanics (manifest hashing, backup directory layouts), full reference tables that live in `docs/`, multi-step walkthroughs longer than ~7 lines.

## 3. Progressive Disclosure

- Start with the happy path. Add edge cases, overrides, and internals only after — or better, behind links.
- Every README section longer than 5 lines should either shrink or link out.
- Use callouts (`> 💡`, `> ⚠️`) sparingly for genuine gotchas, not decoration.

## 4. Chunking & Signposting

- One idea per section. Keep flat lists under ~8 items; convert longer ones to tables.
- Headings must tell readers where they are without reading body text.
- Prefer tables and checklists over prose that must be remembered.

## 5. Dual-Audience Onboarding

| Reader | Path |
| --- | --- |
| **Newbie** | README → Quick Path → Quick Start tutorial → Masterclass guides |
| **Senior** | README → Documentation map → Reference / Architecture directly |

Documentation-map rows must state the intended audience so both personas can self-route in under 10 seconds.

## 6. Writing Rules

- English, neutral professional register.
- Present tense, active voice ("ce-ai restores…", not "the config will be restored…").
- Every command shown must be runnable as written.
- Conventional-commit-friendly headings when documenting features.

## 7. Checklist Before Merging Doc Changes

- [ ] Document has exactly one Diátaxis intent.
- [ ] README still ≤ 100 lines and leads with Quick Path.
- [ ] No content duplicated between README and `docs/`.
- [ ] Audience labels present on doc-map/tutorial entries.
- [ ] All commands verified runnable.
