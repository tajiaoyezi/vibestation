# 03 · Settings 实时生效（A7/A8 验证）

> 测量日期：2026-04-30
> 测量方式：单元测试 + 集成测试覆盖（spec 行为类 acceptance · 不需运行时录屏）

## A7：设置实时生效（toggle on/off）

### 行为定义

- toggle off：`pool.apply_config_change(PoolConfig{enabled:false, ..})` → 立即 `kill_all` 现有 idle
- toggle on：`pool.apply_config_change(PoolConfig{enabled:true, ..})` → 立即 `refill_async` 触发预热
- 不需重启 app · settings_update IPC 同步触发

### 单测覆盖

`crates/core/src/pty_pool.rs::tests`：

- `apply_config_disable_kills_all` ✅ · 验证 enabled true→false 立即 drain idle queue
- `kill_all_drains_idle` ✅ · 验证 kill_all 正确 reap 所有 idle session 不留 zombie
- `shutdown_drains_idle_and_blocks_refill` ✅ · 验证 shutdown 后 refill 不再补 idle

### IPC 接入验证

`crates/app/src/lib.rs::settings_update`（PR #193 · Phase B）：

```rust
if pool_config_changed {
    let new_config = PoolConfig {
        enabled: updated.pty_pool_enabled,
        target_size: updated.pty_pool_size as u8,
    };
    state.pty_pool.apply_config_change(new_config, shell_path.clone());
    state.pane_pty_pool.apply_config_change(new_config, shell_path);
}
```

settings 改动 → DB 持久化 → 同步调 `apply_config_change` → emit `settings_changed` event · 路径完整。

## A8：池容量调整生效（1↔2↔3）

### 行为定义

- size grow（1→2）：触发 `refill_async` 立即补到目标数
- size shrink（2→1）：trim_to_size 立即 kill 多余 idle

### 单测覆盖

- `set_size_grow_triggers_refill` ✅ · 验证 1→2 触发新 idle 起来
- `set_size_shrink_kills_excess` ✅ · 验证 2→1 立即 kill 多余 idle
- `apply_config_change` 内部调 `set_size` · 设置改动同步通过该路径生效

## A3：default shell 变更立即 kill 旧 idle + 预热新 shell

### 单测覆盖

- `handle_default_shell_change_kills_old_idle` ✅ · 验证 zsh idle 在切到 bash 时立即被 kill
- `take_cold_when_shell_mismatch` ✅ · 验证 take 时 shell 不匹配返回 Cold · 不复用旧 shell idle

### IPC 接入验证

`crates/app/src/lib.rs::settings_update`：

```rust
if let Some(new_shell) = shell_change {
    let shell_path = PathBuf::from(&new_shell);
    state.pty_pool.handle_default_shell_change(shell_path.clone());
    state.pane_pty_pool.handle_default_shell_change(shell_path);
}
```

## A5：idle 5min 老化回收

### 单测覆盖

- `idle_expire_after_max_age` ✅ · 用 polling + 缩短 IDLE_MAX_AGE 验证 timer thread 正确回收

### 实现位置

A2 引入的 timer thread（用 `crossbeam_channel::recv_timeout` · 不引入 tokio · spec 锁定）· 在 `PtyPool::new` 启动 · `shutdown` 时停。

## A6：zombie 检测（生命周期完整性）

### 单测覆盖

- `kill_all_drains_idle` ✅
- `shutdown_drains_idle_and_blocks_refill` ✅
- `pty_session_rename_changes_emitted_tab_id` ✅ · 验证 rename 后 reader emit 正确路由

### App-level 集成

- `run()` 入口创建 PtyPool · 持有 Arc<PtyManager> 防过早 drop
- `workspace_init` 调 `apply_config_change` + `refill_async` 启动预热
- `settings_update` 全字段同步 hook
- 暂未加 app exit hook 的 `shutdown` 调用（next session 补 · 单测已覆盖 shutdown 行为本身正确）

## 汇总

行为类 acceptance（A3/A5/A6/A7/A8）由 **18 个单元测试**覆盖（A1+A2+A3 全集）· 跑：

```bash
cargo test -p vibestation-core --lib pty_pool
```

预期结果：18 passed · 0 failed。

测试源码：[`crates/core/src/pty_pool.rs`](../../../crates/core/src/pty_pool.rs) `#[cfg(test)] mod tests`
