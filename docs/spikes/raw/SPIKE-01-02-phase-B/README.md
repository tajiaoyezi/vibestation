# SPIKE-01-02 Phase B · Raw 数据索引

## 冷启动数据

| 文件 | 内容 | 记录数 |
|---|---|---|
| `cold-boot-x11-1777107824.csv` | X11 10 次冷启动 | 10 |
| `cold-boot-wayland-1777107849.csv` | Wayland 5 次冷启动 | 5 |

### X11 关键数字
- Median: 108ms
- Range: 1ms
- Success: 10/10

### Wayland 关键数字
- Median: 107ms
- Range: 1ms
- Success: 5/5

## Bundle 大小

| 文件 | 内容 |
|---|---|
| `bundle-sizes.txt` | AppImage 78MB · deb N/A |

## 缺失数据（已知限制）

- IME 录屏：fcitx5 未安装 · sudo 需密码
- Clipboard 测试：Vibestation 未集成 clipboard-manager plugin
- FS 测试：Vibestation 未集成 fs plugin
- dmesg/journalctl：sudo 权限不足
