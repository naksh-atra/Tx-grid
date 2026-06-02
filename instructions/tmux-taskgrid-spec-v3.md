# tmux-taskgrid: Full Development Specification (Expanded + Architect Additions)

## 1. Product vision and scope

### 1.1 One-sentence vision
A Rust-based tmux plugin that opens a popup “task grid” showing all long-running commands and AI agents across tmux panes, with status, runtime, and one-keystroke jump/kill/restart.

**By the end of the project** you will have a production-ready Rust binary and tmux plugin script that users can install via TPM to get a task dashboard in a popup.

### 1.2 Primary use cases
- **AI / agent workflows**: users running multiple AI coding agents, LLM backends, or tools (e.g., Claude Code, OpenAI tools, local LLMs) in many panes and windows.
- **General dev workflows**: users running tests, servers, workers, scrapers, and training jobs who want a transient dashboard of what is running where.
- **Ops / remote servers**: users attached via SSH to long-lived tmux sessions with many background jobs.

**Expected outcome:** A tool that works well for both AI-heavy and generic long-running workflows and remains tmux-agnostic.

### 1.3 Non-goals (v1)
- Not an AI assistant; no calls to external APIs.
- Not a full tmux session manager like tmuxp; no YAML layouts in v1.
- Not a replacement for statusline plugins (battery, CPU, etc.).

**Expected outcome:** A focused, simple, reliable tool that does one thing well: visualize and manage long-running tasks.


## 2. Technical overview

### 2.1 High-level architecture
- **tmux glue**: a keybinding in tmux that opens a popup and launches the Rust binary inside it using `display-popup` (tmux 3.2+).
- **Rust binary (`tmux-taskgrid`)**:
  - Discovers all panes and their associated PIDs via tmux commands.
  - Inspects process information to infer the foreground command and runtime.
  - Applies heuristics to classify panes as “tasks” (long-running, agent-like, etc.).
  - Renders a text-based grid UI using a Rust TUI library (e.g., `ratatui`) and handles keyboard input.
  - Issues commands back to tmux to jump/kill/restart panes via either `tmux_interface` or `tmux` subprocesses.

**Expected outcome:** Clear separation between tmux as a host and the Rust app as the logic/UI engine.

### 2.2 Target environment
- **tmux**: version 3.2+ (for `display-popup`).
- **OS**: Linux first-class; macOS support desirable but optional for v1.
- **Shells**: bash/zsh/fish supported implicitly (we inspect processes, not shell internals).
- **Plugin manager**: TPM compatible (`tmux-plugins/tpm`).

**Expected outcome:** A binary and plugin script that work on a typical modern dev setup (Linux + tmux + TPM).

### 2.3 Tech stack and dependencies

**Languages & toolchain**
- Rust (edition 2021+)
- Cargo (standard Rust package manager)

**Core crates**
- `ratatui` – TUI framework for rendering the grid UI.
- `crossterm` – terminal backend for `ratatui` (input, raw mode, screen control).
- `tmux_interface` – high-level Rust API to send commands to tmux (`list-panes`, `display-popup`, etc.).
- `procfs` (Linux) or `sysinfo` – for process inspection (command line, start time, status).
- `serde`, `serde_json` – for structured logging or exporting summaries (optional but recommended).
- `anyhow` or `eyre` – ergonomic error handling.
- `thiserror` – for custom error enums when needed.
- `log` + `env_logger` – logging with `RUST_LOG` control.

**Dev tooling**
- `rustfmt` – formatting.
- `clippy` – linting.
- Git + GitHub (or similar) – version control and CI.

**Expected outcome:** A reproducible, documented dependency set so another Rust dev can build and extend the project easily.


## 3. Functional requirements (v1)

### 3.1 Core features

1. **Pane discovery**
   - Implement tmux integration to enumerate panes in all sessions.
   - Fields needed per pane: `session_name`, `session_id`, `window_index`, `window_name`, `pane_index`, `pane_id`, `pane_pid`, `pane_active`.

   **Dependencies:** `tmux` binary, `tmux_interface` or `std::process::Command`.

   **By the end of this subtask you will have:**
   - A function `list_panes()` returning a `Vec<PaneInfo>`.
   - Unit tests validating parsing of `tmux list-panes` output.

2. **Process inspection and classification**
   - Implement process introspection:
     - For each `pane_pid`, inspect `/proc` (Linux) or use `sysinfo` to get:
       - Full command line string.
       - Start time (or CPU time) for runtime computation.
     - Gracefully handle PIDs that no longer exist.
   - Define and compute `TaskState`:
     - `Running` – process alive.
     - `Exited` – process ended (based on missing PID or exit code).
     - `Idle` – only shell, no meaningful child process.
     - `Unknown` – fallback.

   **Dependencies:** `procfs` or `sysinfo`, `chrono` or standard time APIs.

   **By the end of this subtask you will have:**
   - A `ProcessInfo` struct and functions `get_process_info(pid) -> Result<ProcessInfo>`.
   - Deterministic logic to map `ProcessInfo` + `PaneInfo` → `TaskState`.

3. **Task grid UI in tmux popup**
   - Integrate with `display-popup -E 'tmux-taskgrid'` via tmux keybinding.
   - Implement a TUI table using `ratatui` with columns:
     - Index, Session:Window.Pane, Command, Runtime, State.
   - Color-code states (green running, red exited, dim idle).
   - Handle resize events.

   **Dependencies:** `ratatui`, `crossterm`, tmux `display-popup`.

   **By the end of this subtask you will have:**
   - A standalone `tmux-taskgrid` binary that, when run in a tmux popup, displays a static grid of tasks.
   - No actions yet (jump/kill/restart), but navigation and rendering work.

4. **Keyboard navigation and actions**
   - Implement key handling for:
     - `j`/`k`/arrows: move selection.
     - `g`/`G`: jump to first/last.
     - `PgUp`/`PgDn`: page up/down.
     - `/`: filter mode.
     - `Enter`: jump to pane (tmux `select-pane`).
     - `x`: kill pane (`kill-pane` or `send-keys C-c`).
     - `r`: restart last command (best-effort).
     - `q`/`Esc`: quit.

   **Dependencies:** `crossterm` for input, `tmux_interface` or `Command` for actions.

   **By the end of this subtask you will have:**
   - A fully interactive TUI where the selection can move, tasks can be filtered, and actions are executed against tmux.

5. **Filtering and sorting**
   - Implement simple case-insensitive substring filtering by:
     - Command text.
     - Session/window name.
   - Implement sorting by:
     - Runtime (descending default).
     - Session name.
     - State.

   **Dependencies:** Standard Rust collections and iterator utilities.

   **By the end of this subtask you will have:**
   - A `FilteredTaskList` wrapper or equivalent that takes tasks, filter, and sort order and produces a display list, updated when the user changes filter or sort.

6. **Restart last command in pane (v1 best-effort)**
   - Implement minimal restart strategy:
     - Use `pane_current_command` and any available heuristics.
     - Optionally rely on shell integration later.
   - On `r`, send:
     - `C-c`, small delay, then the recorded command, then `Enter`.

   **Dependencies:** `tmux_interface` or `Command`, possibly shell helper in future versions.

   **By the end of this subtask you will have:**
   - A working but documented “best-effort” restart feature.

7. **Configuration via tmux options**
   - Read tmux options like `@taskgrid-popup-width`, `@taskgrid-runtime-threshold`, etc., with defaults.
   - Map options to a `Config` struct.

   **Dependencies:** `tmux` binary (`show-options`), configuration module in Rust.

   **By the end of this subtask you will have:**
   - A configuration system where users can tweak behavior entirely from `.tmux.conf`.


## 4. SDLC and project phases (with outcomes and dependencies)

### Phase 0: Project bootstrap

**Goals:** Create a clean, idiomatic Rust project with CI, linting, formatting, and basic tmux integration.

**Tasks & outcomes:**
1. **Create repo**
   - Steps:
     - Create GitHub repo `tmux-taskgrid`.
     - Initialize with `.gitignore`, `LICENSE`, `README.md` stub.
   - Dependencies: Git, GitHub account.
   - Expected outcome: Public repo ready for code pushes.

2. **Initialize Rust crate**
   - Steps:
     - `cargo new tmux-taskgrid --bin` inside repo.
   - Dependencies: Rust toolchain, Cargo.
   - Expected outcome: Compilable “Hello world” binary with `cargo run`.

3. **Add essential dependencies**
   - Steps:
     - Add dependencies in `Cargo.toml`:
       - `ratatui`, `crossterm`, `tmux_interface`, `procfs`/`sysinfo`, `serde`, `serde_json`, `anyhow`/`eyre`, `thiserror`, `log`, `env_logger`.
   - Expected outcome: `cargo build` succeeds with all dependencies pulled.

4. **Tooling configuration**
   - Steps:
     - Add `rustfmt.toml`, set formatting rules.
     - Ensure `cargo fmt` and `cargo clippy` run cleanly.
   - Expected outcome: Consistent formatting and linting baseline.

5. **CI/CD setup**
   - Steps:
     - Add `.github/workflows/ci.yml` with jobs for build/test/fmt/clippy.
   - Dependencies: GitHub Actions.
   - Expected outcome: Every PR/commit gets automated checks.

6. **License and community files**
   - Steps:
     - Add `LICENSE`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`.
   - Expected outcome: Basic OSS hygiene in place.


### Phase 1: tmux data model and process inspection

**Goals:** Build a standalone library to discover panes and annotate them with process info.

**Tasks & outcomes:**
1. **Define core domain structs**
   - Steps:
     - Create `PaneId`, `PaneInfo`, `TaskState`, `Task`, `ProcessInfo` structs/enums.
   - Dependencies: none beyond Rust std.
   - Expected outcome: Shared types imported by later modules; clear domain language.

2. **tmux integration module (`src/tmux.rs`)**
   - Steps:
     - Implement `list_panes()` using `tmux_interface` or `Command` to call `tmux list-panes -a -F ...` and parse output.
     - Create unit tests with fixture strings of `list-panes` output.
   - Dependencies: tmux installed, `tmux_interface` crate.
   - Expected outcome: Reliable retrieval of pane metadata from tmux.

3. **Process inspection module (`src/process.rs`)**
   - Steps:
     - Implement `get_process_info(pid)` using `procfs` or `sysinfo`.
     - Abstract process provider to allow mocking in tests.
   - Dependencies: `/proc` or equivalent API, `procfs`/`sysinfo` crates.
   - Expected outcome: For a given PID, you can retrieve command and runtime metrics.

4. **Task builder module (`src/task.rs`)**
   - Steps:
     - Implement `build_tasks(panes: &[PaneInfo]) -> Vec<Task>`.
     - Integrate process info and heuristics.
     - Unit-test classification rules.
   - Dependencies: modules from this phase + config defaults.
   - Expected outcome: A list of `Task` objects ready to display.

5. **Logging and diagnostics**
   - Steps:
     - Wire `env_logger` with `RUST_LOG`.
     - Emit debug logs inside `list_panes` and `build_tasks`.
   - Expected outcome: Easier debugging during dev; logs disabled by default for users.


### Phase 2: TUI design and implementation

**Goals:** Implement popup UI, keyboard handling, and basic interaction.

**Tasks & outcomes:**
1. **TUI architecture (`src/ui/mod.rs`)**
   - Steps:
     - Define `App` struct holding tasks, selection, mode, filter, sort order.
     - Implement `App::new` and `App::update(Event)`.
   - Dependencies: `ratatui`, `crossterm` event types.
   - Expected outcome: Central stateful object to drive rendering and input.

2. **Layout and rendering**
   - Steps:
     - Design layout with header, main grid, footer.
     - Implement `fn draw<B: Backend>(f: &mut Frame<B>, app: &App)`.
   - Dependencies: `ratatui` widgets and layout.
   - Expected outcome: Clean, readable task grid UI.

3. **Event loop**
   - Steps:
     - Implement main loop: initialize terminal, poll for events, call `App::update`, redraw.
     - Handle `Ctrl-C`, `q`, and panic-safe teardown.
   - Dependencies: `crossterm` for raw mode and event polling.
   - Expected outcome: Responsive interactive app that exits cleanly.

4. **Filtering and sorting logic**
   - Steps:
     - Implement filter mode (`/`), capturing user input into `filter_text`.
     - Implement sort order toggles.
   - Expected outcome: Dynamic view that updates as user types.

5. **User feedback and errors**
   - Steps:
     - Add `status_message: Option<String>` to `App`.
     - Display messages in footer for errors and confirmations.
   - Expected outcome: Clear communication of actions and failures to user.

6. **TUI integration test / non-interactive mode**
   - Steps:
     - Add CLI flag `--check` that queries tmux and prints task summary JSON then exits.
   - Expected outcome: CI-safe way to verify integration without full TTY UI.


### Phase 3: Actions back into tmux

**Goals:** Wire UI actions to tmux commands.

**Tasks & outcomes:**
1. **Jump to pane**
   - Steps:
     - Implement `jump_to_pane(pane_id)` using `tmux select-pane -t ...` and possibly `select-window` first.
   - Expected outcome: Pressing `Enter` from the popup focuses the chosen pane.

2. **Kill pane**
   - Steps:
     - Implement `kill_pane(pane_id)` using `kill-pane` or `send-keys C-c` then `kill-pane`.
     - Add config-based confirmation dialog.
   - Expected outcome: Pane processes can be terminated safely from the grid.

3. **Restart last command**
   - Steps:
     - Implement minimal `restart_task(task)` logic as described earlier.
     - Document limitations in README.
   - Expected outcome: Users can quickly re-run tasks with one key.

4. **Error handling and reporting**
   - Steps:
     - Wrap tmux calls with error handling and surface errors in status area.
   - Expected outcome: Failures are visible and do not crash the app.


### Phase 4: Configuration system

**Goals:** Allow customization via tmux options.

**Tasks & outcomes:**
1. **Option reading helper**
   - Steps:
     - Implement `get_tmux_option(name, default)` helper using `tmux show-options -gqv`.
   - Expected outcome: Safe way to read tmux config with fallback defaults.

2. **Config struct & mapping**
   - Steps:
     - Define `Config` and map tmux options into it.
   - Expected outcome: Config passed into task builder and UI for behavior control.

3. **Application of config**
   - Steps:
     - Apply config for runtime thresholds, patterns, confirm flags, theme.
   - Expected outcome: Behavior changes correctly when user updates `.tmux.conf`.

4. **Docs**
   - Steps:
     - Update README and `docs/configuration.md` with all options.
   - Expected outcome: Users understand how to tune the plugin.


### Phase 5: Packaging as a tmux plugin

**Goals:** Make plugin easy to install and update via TPM or manually.

**Tasks & outcomes:**
1. **Repository layout for plugin**
   - Steps:
     - Organize folder structure as previously outlined.
   - Expected outcome: Clear separation of src, scripts, docs, tests, dist.

2. **Tmux plugin script (`scripts/taskgrid.tmux`)**
   - Steps:
     - Provide plugin declaration and keybinding setup compatible with TPM.
   - Expected outcome: Users add `set -g @plugin 'yourname/tmux-taskgrid'` and get a working keybinding.

3. **Installation instructions**
   - Steps:
     - Document TPM and manual install in README.
   - Expected outcome: Installation friction minimized.

4. **Versioning and releases**
   - Steps:
     - Adopt semver, tag releases, attach binaries.
   - Expected outcome: Stable versions users can pin to.


### Phase 6: Testing strategy

**Goals:** Ensure reliability with unit, integration, and manual tests.

**Tasks & outcomes:**
1. **Unit tests**
   - Steps:
     - Implement comprehensive tests for parsing, classification, and UI state.
   - Expected outcome: High confidence in core logic.

2. **Integration tests**
   - Steps:
     - Use test tmux server and `--check` mode to verify pane discovery and classification.
   - Dependencies: tmux available in CI.
   - Expected outcome: Integration between Rust and tmux validated.

3. **Manual acceptance tests**
   - Steps:
     - Follow `docs/testing.md` scenarios to validate UX.
   - Expected outcome: Confirmed behavior in realistic developer environments.

4. **Performance sanity checks**
   - Steps:
     - Manually test with large numbers of panes.
   - Expected outcome: No noticeable lag for typical workloads.


### Phase 7: Documentation and polish

**Goals:** Finalize docs, community artifacts, and UX polish.

**Tasks & outcomes:**
1. **README polish**
   - Steps:
     - Add badges, GIF demo, quickstart.
   - Expected outcome: Attractive landing page for GitHub visitors.

2. **Additional docs**
   - Steps:
     - Flesh out overview, usage, configuration, testing docs.
   - Expected outcome: Contributors and users understand design and usage patterns.

3. **Changelog & templates**
   - Steps:
     - Maintain `CHANGELOG.md` and add issue/PR templates.
   - Expected outcome: Structured evolution of the project.


## 5. Developer workflows (recap)

### Local development
- Install Rust, tmux, and dependencies.
- Run `cargo fmt`, `cargo clippy`, `cargo test` regularly.
- Use tmux popup (`display-popup -E "target/debug/tmux-taskgrid"`) during iteration.

### Release
- Update version and changelog.
- Tag and push; GitHub Actions builds binaries.
- Update README if install steps change.

---

This expanded specification now includes for each phase/task: dependencies, what to expect by the end, and the full set of steps you’d hand to a professional developer to implement `tmux-taskgrid` from scratch.

## 6. Senior architect additions

The following sections are **additions only**. They do not replace prior requirements; they extend them with the missing professional planning, GitHub workflow, pre-build considerations, review gates, release discipline, and final finishing touches.

### 6.1 Developer mindset before writing code

Before the first line of code is written, the developer should treat the project as a product, not just a binary.

**Things to think through first:**
- What exact user pain is being solved in one sentence?
- What are the 3 most important user actions in v1?
- What is explicitly out of scope for v1 to avoid scope creep?
- What does “good enough to publish” mean for quality, performance, and usability?
- What would make a tmux power user star the repo after trying it once?

**Expected outcome:**
- A written v1 scope note in the repo, ideally in `docs/product-scope.md`.
- A clearly defined product promise and a short “will not build in v1” list.

### 6.2 Architecture principles

These principles should guide every implementation decision:
- Prefer **simple over clever**.
- Prefer **observable behavior over hidden magic**.
- Prefer **stable, typed modules** over ad hoc scripts.
- Prefer **graceful degradation** over hard failure.
- Prefer **small reusable functions** over deeply nested orchestration.
- Prefer **explicit command execution and error surfaces** over silent fallback logic.

**Expected outcome:**
- Code review decisions can be justified against these principles.
- The project stays maintainable when new contributors arrive.

### 6.3 Definition of done at the project level

The project is only “done” for a release when all of the following are true:
- The binary builds on supported platforms.
- Unit and integration tests pass.
- Manual acceptance checks pass.
- README install steps work from a clean machine.
- The tmux plugin script works with TPM.
- The UI is usable at standard terminal sizes.
- Error states are understandable.
- Release artifacts are attached and documented.

**Expected outcome:**
- A release checklist in `docs/release-checklist.md`.
- No ambiguity about when to cut a release.

## 7. Pre-build planning and product discovery

### 7.1 Problem validation

Before implementation starts, the developer should validate that the problem is real and narrow.

**Tasks:**
1. Write a short problem statement.
2. Identify who the first user is: solo dev, AI workflow user, ops engineer, or general tmux user.
3. Capture 5–10 realistic commands/workloads the tool must handle.
4. Define what the tool will call a “task.”
5. Define which states are critical: running, idle, exited, failed, attention-needed.

**Artifacts to create:**
- `docs/problem-statement.md`
- `docs/user-personas.md`
- `docs/example-workloads.md`

**Expected outcome:**
- The team understands what the product must detect and display before implementation starts.

### 7.2 UX contract before coding

The interaction model should be frozen before the TUI is built.

**Tasks:**
1. Write the complete keymap.
2. Decide the exact default sort and filter behavior.
3. Decide whether actions are immediate or confirmed.
4. Decide what happens when no tasks are found.
5. Decide what happens when tmux is unavailable or too old.
6. Decide whether the UI is static snapshot or live-refresh in v1.

**Artifacts to create:**
- `docs/ux-contract.md`
- `docs/keybindings.md`

**Expected outcome:**
- Fewer UI rewrites later because interaction decisions were made early.

### 7.3 Risk register

A professional project should explicitly track risks.

**Initial risks:**
- Incorrect process attribution from `pane_pid`.
- Restart logic being unreliable across shells.
- Popup sizing behaving differently across terminals.
- macOS process inspection differences.
- tmux version drift and option incompatibilities.
- Terminal rendering quirks with narrow widths.
- User expectations that the plugin is a full task manager.

**Artifacts to create:**
- `docs/risks.md`
- `docs/assumptions.md`

**Expected outcome:**
- Known risks are visible and can shape implementation order.

## 8. Recommended repository structure

Use a structure that supports product code, docs, scripts, and future contributors.

```text
tmux-taskgrid/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   ├── workflows/
│   │   ├── ci.yml
│   │   ├── release.yml
│   │   └── quality.yml
│   ├── pull_request_template.md
│   └── dependabot.yml
├── docs/
│   ├── architecture.md
│   ├── assumptions.md
│   ├── configuration.md
│   ├── decision-log.md
│   ├── example-workloads.md
│   ├── keybindings.md
│   ├── manual-test-plan.md
│   ├── problem-statement.md
│   ├── product-scope.md
│   ├── release-checklist.md
│   ├── risks.md
│   ├── testing.md
│   ├── troubleshooting.md
│   ├── ux-contract.md
│   └── user-personas.md
├── scripts/
│   ├── install.sh
│   ├── open_taskgrid.sh
│   ├── taskgrid.tmux
│   ├── test-e2e.sh
│   └── dev-fixture-session.sh
├── src/
│   ├── app.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── errors.rs
│   ├── logging.rs
│   ├── main.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── pane.rs
│   │   ├── process.rs
│   │   └── task.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── process_service.rs
│   │   ├── task_service.rs
│   │   └── tmux_service.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── events.rs
│   │   ├── layout.rs
│   │   ├── render.rs
│   │   └── state.rs
│   └── utils/
│       ├── mod.rs
│       ├── format.rs
│       └── parse.rs
├── tests/
│   ├── fixtures/
│   │   ├── list_panes_basic.txt
│   │   ├── list_panes_complex.txt
│   │   └── process_samples/
│   ├── integration_tmux.rs
│   ├── parsing.rs
│   └── task_classification.rs
├── Cargo.toml
├── Cargo.lock
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── LICENSE
├── README.md
└── rustfmt.toml
```

**Expected outcome:**
- A repo that looks production-ready from day one and can scale beyond a prototype.

## 9. Detailed implementation work breakdown

### 9.1 Milestone plan

#### Milestone M0 — Foundation
**Tasks:**
- Initialize repo and crate.
- Add docs scaffold.
- Add lint/test/format CI.
- Add baseline command runner.

**Expected outcome:**
- Clean repo with repeatable tooling and no feature code yet.

#### Milestone M1 — Data discovery
**Tasks:**
- List panes.
- Parse tmux output.
- Build pane model.
- Add parse tests.

**Expected outcome:**
- `tmux-taskgrid --check` can enumerate panes reliably.

#### Milestone M2 — Process inference
**Tasks:**
- Inspect process tree.
- Compute command and runtime.
- Implement state classification.
- Add classification tests.

**Expected outcome:**
- A stable task list can be produced from real tmux panes.

#### Milestone M3 — First UI
**Tasks:**
- Render header/table/footer.
- Support selection movement.
- Support quit.
- Support empty state.

**Expected outcome:**
- Users can open the popup and browse tasks.

#### Milestone M4 — Actions
**Tasks:**
- Jump to pane.
- Kill pane.
- Add confirmation.
- Add status messaging.

**Expected outcome:**
- The popup becomes operational, not just informational.

#### Milestone M5 — Configuration + installability
**Tasks:**
- Add tmux option parsing.
- Add plugin script.
- Add README install docs.
- Add manual install flow.

**Expected outcome:**
- Third-party users can install and use the tool.

#### Milestone M6 — Hardening
**Tasks:**
- Improve error handling.
- Improve edge-case handling.
- Expand tests.
- Run manual acceptance suite.

**Expected outcome:**
- Candidate release quality.

#### Milestone M7 — Release
**Tasks:**
- Prepare changelog.
- Tag version.
- Publish binaries.
- Announce repo.

**Expected outcome:**
- Public release with a professional OSS presentation.

### 9.2 Task sequencing guidance

The developer should implement in this order:
1. Models and parsing.
2. Process inspection.
3. Classification logic.
4. Basic CLI output.
5. TUI rendering.
6. TUI navigation.
7. tmux actions.
8. Config.
9. Packaging.
10. Documentation hardening.
11. Release automation.

**Expected outcome:**
- Lower rework, fewer architectural dead ends, and continuous usable checkpoints.

## 10. Engineering standards

### 10.1 Rust code standards

**Rules:**
- Prefer small modules with single responsibility.
- Avoid god structs and god modules.
- Return rich errors using `thiserror`/`anyhow`.
- Do not panic in expected runtime failures.
- Avoid hidden global mutable state.
- Keep public APIs minimal.
- Prefer immutable data flow where practical.
- Use enums for state, not string literals.
- Use typed wrappers for pane IDs and task IDs where helpful.

**Expected outcome:**
- Code remains readable and safe as complexity grows.

### 10.2 Error handling standards

Every failure mode should be designed.

**Cases to handle explicitly:**
- tmux not installed.
- tmux server not running.
- tmux version too old.
- popup command unavailable.
- pane disappears during scan.
- process info missing.
- restart command unavailable.
- command execution returns non-zero.
- unsupported platform for process inspection.

**Expected outcome:**
- No confusing crashes for normal operational failures.

### 10.3 Logging standards

**Logging levels:**
- `error`: command failures and unrecoverable problems.
- `warn`: partial degradation or skipped panes.
- `info`: startup summary, config source, mode.
- `debug`: raw tmux output, parse details, classification reasons.
- `trace`: extremely detailed internals for local debugging only.

**Expected outcome:**
- Fast diagnosis without cluttering the normal user experience.

## 11. Command-line interface plan

Even if the main experience is the popup, the binary should expose explicit CLI modes.

### 11.1 Recommended commands

- `tmux-taskgrid` → launch TUI.
- `tmux-taskgrid --check` → print summary and exit.
- `tmux-taskgrid --json` → print machine-readable task data.
- `tmux-taskgrid --debug` → enable verbose logs.
- `tmux-taskgrid --version` → print version/build metadata.
- `tmux-taskgrid doctor` → environment diagnostics.

**Dependencies:**
- `clap` or `argh` for argument parsing.

**Expected outcome:**
- Better developer ergonomics, easier CI, and easier bug reporting.

### 11.2 Doctor command

`doctor` should check:
- tmux present?
- tmux version supported?
- currently inside tmux?
- popup support available?
- binary on PATH?
- plugin script installed?
- shell integration installed or not?

**Expected outcome:**
- Users can self-diagnose common setup problems.

## 12. Git and GitHub workflow

### 12.1 Repository creation steps

1. Create a public repo named `tmux-taskgrid`.
2. Add description and topics such as `tmux`, `terminal`, `rust`, `tui`, `cli`, `developer-tools`.
3. Add MIT or Apache-2.0 license.
4. Protect the default branch.
5. Require pull request reviews for changes once collaboration begins.
6. Require status checks for CI before merge.
7. Enable Discussions if community feedback is desired.
8. Add issue templates and PR template.
9. Enable Dependabot for GitHub Actions and Cargo.
10. Configure release page and pinned README screenshots/GIF.

**Expected outcome:**
- A professional GitHub repository that is contributor-ready and trustworthy.

### 12.2 Branching strategy

Use a lightweight trunk-based workflow initially.

**Recommended branches:**
- `main` → always releasable.
- `feat/*` → feature branches.
- `fix/*` → bugfix branches.
- `docs/*` → documentation-only branches.
- `release/*` → optional stabilization branch for bigger releases.

**Expected outcome:**
- Predictable history and safe merges.

### 12.3 Commit message standard

Use conventional commits where possible:
- `feat: add pane discovery parser`
- `fix: handle tmux popup unsupported versions`
- `docs: expand installation guide`
- `test: add integration coverage for pane classification`
- `chore: enable clippy in CI`

**Expected outcome:**
- Cleaner changelog generation and better commit readability.

### 12.4 Pull request workflow

Each PR should include:
- Problem being solved.
- Scope of change.
- Screenshots/GIF for UI changes.
- Test evidence.
- Risks or follow-up items.

**PR checklist:**
- [ ] Code builds locally.
- [ ] Tests pass locally.
- [ ] Formatting applied.
- [ ] Linting passes.
- [ ] Docs updated if needed.
- [ ] No debug code left behind.
- [ ] Manual test evidence included if behavior changed.

**Expected outcome:**
- Higher merge quality and easier review cycles.

### 12.5 GitHub Actions workflows

Create these workflows:

#### `ci.yml`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets`

#### `quality.yml`
- Link checking in markdown docs.
- Shell script linting (`shellcheck`) for `scripts/*.sh`.
- Markdown linting if desired.

#### `release.yml`
- Trigger on version tag.
- Build release binaries.
- Generate checksums.
- Publish GitHub Release.
- Attach artifacts.

**Expected outcome:**
- Reliable automation from code quality to release publishing.

### 12.6 Issue management

Use labels from the beginning:
- `bug`
- `enhancement`
- `good first issue`
- `help wanted`
- `docs`
- `performance`
- `ux`
- `question`
- `blocked`

**Expected outcome:**
- Better triage and contributor onboarding.

## 13. Test plan expansion

### 13.1 Unit test matrix

Add tests for:
- Empty tmux output.
- Malformed tmux output.
- Pane names with unusual characters.
- Very long commands.
- Commands with spaces and quotes.
- Missing process metadata.
- Runtime formatting edge cases.
- Filter and sort stability.
- Selection clamping after filter changes.

**Expected outcome:**
- Strong correctness coverage at the lowest level.

### 13.2 Integration test matrix

Test these scenarios:
- One session, one pane, one long-running task.
- Multiple sessions with mixed tasks.
- Pane closes during scan.
- Hidden or detached session.
- Idle shell pane only.
- Mixed shells and processes.
- Unsupported tmux version fallback behavior.

**Expected outcome:**
- Confidence that real tmux environments behave correctly.

### 13.3 Manual test matrix

The developer should manually validate:
- Popup opens at expected size.
- Narrow terminal behavior.
- Wide terminal behavior.
- Color and readability on dark/light terminal themes.
- Keybindings feel natural.
- Jump action lands in the correct pane.
- Kill action confirms correctly.
- Restart action behaves predictably or fails clearly.
- Empty state messaging is helpful.
- Error state messaging is helpful.

**Expected outcome:**
- UI and UX feel polished, not merely functional.

### 13.4 Performance testing

Measure and document:
- Startup time with 5 panes.
- Startup time with 20 panes.
- Startup time with 50 panes.
- Rendering performance during repeated refreshes if live refresh is added.

**Expected outcome:**
- Baseline performance targets are known and can be improved over time.

## 14. Documentation set to complete before first public release

### 14.1 README sections

README should include:
- What it is.
- Why it exists.
- Demo GIF.
- Installation.
- Requirements.
- Keybindings.
- Configuration.
- Limitations.
- Troubleshooting.
- Development.
- Contributing.
- License.

**Expected outcome:**
- A first-time visitor can install and understand the project without opening the code.

### 14.2 Troubleshooting guide

Create `docs/troubleshooting.md` with sections like:
- Popup does not open.
- Binary not found.
- No tasks shown.
- Wrong command detected.
- Restart not working.
- tmux version incompatible.
- macOS limitations.

**Expected outcome:**
- Reduced support burden and better user trust.

### 14.3 Decision log

Create `docs/decision-log.md` to record important architectural choices:
- Why Rust?
- Why ratatui?
- Why tmux_interface vs raw commands?
- Why best-effort restart in v1?
- Why Linux-first?

**Expected outcome:**
- Future contributors understand the reasoning behind the design.

## 15. Security and safety considerations

Even a local CLI tool should define safe behavior.

**Rules:**
- Do not execute arbitrary user input without explicit action.
- Clearly distinguish read-only actions from destructive actions.
- Require confirmation for kill by default.
- Make restart behavior transparent.
- Never silently delete sessions or panes beyond the selected target.
- Avoid shell injection risks by passing command arguments safely.

**Expected outcome:**
- The tool feels safe to try, even for cautious users.

## 16. Packaging and release engineering

### 16.1 Release artifact plan

For each release, publish:
- Linux x86_64 binary.
- Linux aarch64 binary if possible.
- macOS binaries when support is mature.
- SHA256 checksums.
- Install instructions in release notes.

**Expected outcome:**
- Users can install without building from source.

### 16.2 Versioning policy

Use semantic versioning:
- `0.1.0` → first public preview.
- `0.2.0` → new features, still pre-1.0.
- `0.2.1` → bug fixes.
- `1.0.0` → stable public contract.

**Expected outcome:**
- Clear expectations for stability and compatibility.

### 16.3 Release checklist

Before tagging a release:
- [ ] Update version.
- [ ] Update changelog.
- [ ] Run fmt/clippy/tests.
- [ ] Run integration tests.
- [ ] Run manual acceptance checks.
- [ ] Verify README install path from a clean environment.
- [ ] Verify release notes draft.
- [ ] Confirm artifacts build in CI.

**Expected outcome:**
- Fewer broken releases and more user confidence.

## 17. Post-release workflow

### 17.1 First release actions

After the first public release:
- Open a feedback issue template.
- Watch installation issues closely.
- Track feature requests but defend v1 scope.
- Fix high-friction onboarding issues first.
- Improve screenshots/GIFs based on user confusion.

**Expected outcome:**
- Faster polish loop and better adoption.

### 17.2 Maintenance rhythm

Adopt a simple maintenance cadence:
- Weekly review of issues/discussions.
- Monthly dependency updates.
- Quarterly review of roadmap and priorities.
- Release bugfixes quickly when installation or core commands break.

**Expected outcome:**
- The project stays alive and credible after launch.

## 18. Finishing touches before calling it complete

These are the often-missed details that make the repo feel mature.

### 18.1 UX finishing touches
- Helpful empty state copy.
- Clear footer help text in the popup.
- Consistent runtime formatting.
- Sensible truncation for long commands.
- Clear color contrast in the table.
- Friendly, non-cryptic error messages.
- Confirmation dialog wording that is brief and precise.

**Expected outcome:**
- The tool feels intentionally designed.

### 18.2 GitHub finishing touches
- Add a clean social preview image.
- Pin the repo on the profile if relevant.
- Add badges for build/release/license.
- Add animated demo to README.
- Write release notes in plain language.
- Add a roadmap section without overpromising.

**Expected outcome:**
- The repository looks serious enough for people to try and share.

### 18.3 Codebase finishing touches
- Remove debug prints.
- Remove dead code and unused dependencies.
- Normalize module naming.
- Audit public API exposure.
- Ensure comments explain intent, not obvious behavior.
- Ensure every config option has documentation.

**Expected outcome:**
- Cleaner long-term maintainability and better contributor experience.

## 19. Final handoff checklist for a developer

When handing this project to a developer, the assignment should be:

1. Read the scope, UX contract, and risk docs.
2. Set up the repo exactly as structured.
3. Implement milestone by milestone in order.
4. Do not skip tests while building features.
5. Keep `main` releasable.
6. Update docs with every user-facing change.
7. Publish only after release checklist passes.
8. Treat installation and first-run experience as part of the product.

**Expected outcome:**
- The developer has a complete architect-level blueprint from idea to launch without ambiguity.

## 20. Recommended next document additions beyond this spec

If the project continues, the next supporting documents to create are:
- `docs/architecture.md` — component and dependency diagrams.
- `docs/manual-test-plan.md` — step-by-step QA scripts.
- `docs/release-checklist.md` — release gate checklist.
- `docs/troubleshooting.md` — support guide.
- `docs/decision-log.md` — architecture decisions over time.

**Expected outcome:**
- The specification evolves into a full operating handbook for the project.
