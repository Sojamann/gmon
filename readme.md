# GMON - Gitlab Monitoring

## Config
The config file is located at `~/.config/gmon/config.toml`.
```toml
host = "your.gitlab.instance" # HOSTNAME not URL
token = "your personal access token"
```

## Commands
### Pipelines
Monitor the status of pipelines in general. It shows a bunch of *colored* blocks
indicating the status of the last pipelines of the given project.

```bash
gmon pipelines path/to/project[@ref]

┌gitlab-org/gitlab────────────────────────────────────────┐
│┌master─────────────────────────────────────────────────┐│
││    ███  ███  ███  ███  ███   »   ███  ███  ███  ███   ││
│└───────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

|Symbol | Pipeline Status   |
-----------------------------
| green block  | successful |
| red block    | failed     |
| grey block   | created    |
| blue block   | running    |
|     »        | skipped    |
| scull        | cancled    |
| hand symbol  | manual     |
| clock symbol | schedueled |
| dim block    | pending    |
| dim block    | preparing  |
| dim block    | canceling  |
| dim block    | waiting for resource |
| dim block    | waiting for callback |
