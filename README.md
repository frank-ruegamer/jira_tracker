# Jira Tracker

## Installation

Recommended/Optional Setup

### Required ENV variables

| Variable        | Description                                                | Default |
|-----------------|------------------------------------------------------------|---------|
| JIRA_EMAIL      | Jira Account Email                                         |         |
| JIRA_API_TOKEN  | API Token for Jira API                                     |         |
| TEMPO_API_TOKEN | API Token for Tempo API                                    |         |
| JSON_FILE       | Location of persistent state json file (preserve restarts) |         |
| TRACKER_PORT    | Port the web server will run on (optional)                 | 8080    |

### Executable

`cargo install --git https://github.com/frankruegamer/jira_tracker jira_tracker`

### Systemd Service

Systemd user service in `~/.config/systemd/user`

```ini
[Service]
# or simply ExecStart=jira_tracker if it's in your PATH
ExecStart=%h/.cargo/bin/jira_tracker

[Install]
WantedBy=default.target
```

### Environment setup

With `systemctl --user edit jira-tracker.service`.

Content

```ini
[Service]
Environment="JIRA_EMAIL=<...>"
Environment="JIRA_API_TOKEN=<...>"
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

## Known Issues

None
