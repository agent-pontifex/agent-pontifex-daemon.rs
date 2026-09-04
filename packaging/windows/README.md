# Windows Service Control Manager

Register with `sc.exe`:

```powershell
sc.exe create AgentPontifexDaemon `
  binPath= "C:\Program Files\Agent Pontifex\agent-pontifex-daemon.exe" `
  start= auto `
  DisplayName= "Agent Pontifex Daemon"

sc.exe failure AgentPontifexDaemon reset= 86400 actions= restart/5000/restart/5000/restart/30000
```

## Stop timeout

The SCM default stop timeout is 30 s, taken from
`HKLM\SYSTEM\CurrentControlSet\Control\WaitToKillServiceTimeout`. It must exceed
`shutdown-grace-seconds` or an active drain is killed mid-job — the same
constraint as `TimeoutStopSec` (systemd) and `ExitTimeOut` (launchd).

## Configuration

Environment values come from the service's own environment block or from
`%PROGRAMDATA%\Agent Pontifex\daemon.env`. Never pass credentials on the command
line: `binPath` is world-readable via `sc.exe qc`.
