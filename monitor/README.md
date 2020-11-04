# monitor

Simple binary that's designed to execute arbitrary tasks and notify a provided healthchecks.io check about their status.

## Usage

### Execute an arbitrary task

```shell
HEALTHCHECKS_CHECK_ID=<check_id> healthchecks-monitor -X sleep 10
```

### Start off a timer server-side

```shell
HEALTHCHECKS_CHECK_ID=<check_id> healthchecks-monitor -tX sleep 10
```

### Use a custom user agent

```shell
HEALTHCHECKS_USERAGENT=crontab HEALTHCHECKS_CHECK_ID=<check_id> healthchecks-monitor -tX sleep 10
```
