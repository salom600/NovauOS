---
name: Bug report
about: Report a problem with NovauOS
title: "[BUG] "
labels: bug, triage
assignees: ''
---

**Describe the bug**
A clear description of what the bug is.

**To reproduce**
Steps to reproduce the behavior:
1. Boot NovauOS ISO version `...`
2. Click on `...`
3. See error

**Expected behavior**
What you expected to happen.

**Screenshots / logs**
If applicable, paste screenshots or relevant log output. For full logs:
```
journalctl -b > journal.txt
```

**Environment**
- NovauOS version: `cat /etc/os-release`
- Kernel: `uname -r`
- GPU: `lspci | grep -i vga`
- CPU: `lscpu | grep Model name`

**Additional context**
Anything else relevant.
