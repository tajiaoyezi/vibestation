# C · correctness

- resize trap output: `40 100`
- fd before: 14
- fd after: 14
- rss before: 115440 KB
- rss after: 119296 KB

## Checks

- SIGWINCH: PASS
- cleanup(fd delta): 0
- reader alive: true
