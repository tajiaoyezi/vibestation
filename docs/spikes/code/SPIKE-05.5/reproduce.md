# reproduce

```bash
cd spike-tmp/spike-05.5-pty
./scripts/bench-compare.sh 3
./scripts/check-correctness.sh shared 1
./scripts/check-correctness.sh per-session 1
python3 ./scripts/generate-report.py
```

## 关键产物

- `raw-data/shared/single-yes/run-*/`
- `raw-data/shared/four-yes/run-*/`
- `raw-data/per-session/single-yes/run-*/`
- `raw-data/per-session/four-yes/run-*/`
- `reports/compare-table.csv`
- `reports/SUMMARY.md`
- `reports/SPIKE-05.5-report.md`
