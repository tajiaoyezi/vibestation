# C · correctness

- resize trap output: `40 100`
- fd before: 14
- fd after: 14
- rss before: 116368 KB
- rss after: 117040 KB

## Checks

- SIGWINCH: PASS
- cleanup(fd delta): 0
- reader alive: true
