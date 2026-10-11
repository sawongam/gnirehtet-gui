# gnirehtet-controller

Supervises the external `gnirehtet relay` process for the desktop app.

- `RelaySupervisor::start(port)` refuses to start when the port is already taken
  (`PORT_IN_USE`), so a relay started by someone else is never adopted or killed.
- It waits until the relay actually listens before reporting success, and reports
  the relay's own error output if it exits during startup.
- `stop()`, and dropping the supervisor, only ever kill the process it started.
- The relay's stdout/stderr are handed to the caller (`RelayStdio`), which must
  keep draining them.
