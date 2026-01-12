# StreamTUI Implementation Task

## Instructions

1. Read specs/readme.md to understand the project
2. Read specs/implementation_plan.md for current tasks
3. Pick the FIRST uncompleted task (marked [ ])
4. For TDD tasks:
   - If task says "tests": Write tests ONLY, they should fail
   - If task says "implementation": Make the tests pass
5. Mark the task as done [x] in implementation_plan.md
6. Stop and report what you did

## TDD Rules

### When writing tests:
- Tests go in tests/ directory
- Use mockito for HTTP mocking
- Tests MUST fail initially (no implementation yet)
- Cover all test cases from the relevant spec file
- Use descriptive test names

### When implementing:
- Make ALL related tests pass
- Follow the spec exactly
- Keep code clean and idiomatic Rust
- Handle errors properly with anyhow/thiserror

## Constraints

- Only work on ONE task per run
- Follow the specs in specs/*.md exactly
- Use Rust 2021 edition
- No unsafe code unless absolutely necessary
- Prefer async/await for I/O operations

## Working Directory
/home/gorkolas/projects/streamtui

## Design Notes
- Cyberpunk neon aesthetic (see specs/tui.md for colors)
- Cast to Chromecast is the primary use case
- Use catt CLI for casting (simpler than native protocol)
- Use webtorrent-cli for torrent streaming

## When Done
Update implementation_plan.md marking the task complete.
Report: What tests you wrote OR what you implemented to make tests pass.
