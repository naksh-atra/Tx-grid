# Problem Statement

## The Pain

tmux users running multiple long-running processes (AI agents, dev servers, tests, scrapers) have no built-in way to see what's running where without manually cycling through panes.

## Who This Is For

1. **AI workflow users** — Running Claude Code, OpenAI tools, or local LLMs across many panes
2. **General developers** — Running tests, servers, workers, and wanting a quick overview
3. **Ops engineers** — SSH'd into remote servers with many background jobs

## What "Task" Means

A task is any pane running a foreground process that is not just an idle shell. We classify:
- **Running**: Active non-shell process (nvim, cargo, python, etc.)
- **Idle**: Only a shell is running, no meaningful child
- **Exited**: The process has ended

## Critical States

- Running
- Idle
- Exited
- Unknown (fallback)
