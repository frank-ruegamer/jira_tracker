# Jira Tracker

## Installation

Recommended/Optional Setup

### Required ENV variables

| Variable        | Description                                                |
|-----------------|------------------------------------------------------------|
| JIRA_ACCOUNT_ID | Jira Account ID associated with submitted worklogs         |
| TEMPO_API_TOKEN | API Token for Tempo API                                    |
| JSON_FILE       | Location of persistent state json file (preserve restarts) |

### Executable

Build with `cargo build --release` and put in PATH e.g. `~/bin`

### Systemd Service

Systemd user service in `~/.config/$HOME/systemd/user`

```ini
[Service]
ExecStart=%h/bin/jira_tracker

[Install]
WantedBy=default.target
```

### Environment setup

With `systemctl --user edit jira-tracker.service`.

Content

```ini
[Service]
Environment="JIRA_ACCOUNT_ID=<...>"
Environment="TEMPO_API_TOKEN=<...>"
Environment="JSON_FILE=~/.local/share/jira-tracker/file.json"
```

## Usage

### Systemd

*Start/Restart/Stop*

`systemctl --user start jira-tracker`

`systemctl --user restart jira-tracker`

`systemctl --user stop jira-tracker`

### HTTP API

Have a look in `src/web.rs`
