# ADR-001: 许可证 = Apache License 2.0（不签 CLA）

**状态**：accepted
**日期**：2026-04-18（Phase 1 锁定 · Phase 3 ADR 建立）
**决策者**：项目发起人（@leaf） · Phase 1 多 agent 评审
**对应 `CLAUDE.md` 决策表**：#1

---

## 背景与问题

Vibestation 作为开源桌面应用，需要选择许可证。核心诉求：
- 允许商业使用 / 分发 / 修改
- 明确专利授权（避免 submarine patent 风险）
- 社区贡献门槛低（不额外签署 CLA）
- 与 Rust / JavaScript 生态兼容度高

候选许可证（MIT / BSD / Apache 2.0 / GPL）之间差别不只在"是否允许商用"，核心差别是**专利条款 + 贡献者授权机制**。

## 决策驱动因素

- **D1 · 专利保护**：企业用户 / 企业贡献者引入代码时的专利陷阱
- **D2 · 贡献友好**：CLA 签署会劝退非正式贡献者（尤其 AI agent 产出的 patch）
- **D3 · 生态兼容**：与 Rust crates.io / npm 主流库许可证不冲突
- **D4 · 法律清晰度**：条款明确 · 不需法律团队解读

## 考虑的选项

- **选项 A · MIT**：极简 · 无专利条款 · 不推荐大型项目用于商业场景
- **选项 B · BSD 3-Clause**：与 MIT 类似 · 略有品牌保护 · 无专利条款
- **选项 C · Apache License 2.0**：含 patent grant · 含贡献者隐式授权 · 无需 CLA
- **选项 D · GPL v3**：copyleft · 衍生作品必须开源 · 企业使用受限
- **选项 E · MPL 2.0**：文件级 copyleft · 比 GPL 宽松 · 适合库不适合应用

## 决策

**选择**：选项 C · **Apache License 2.0**

**理由**：
1. **Patent grant 条款**（§3）：贡献者默认授予用户专利权 · 贡献者不得起诉用户专利侵权（若起诉则授权失效）· 企业贡献者防御机制清晰
2. **贡献者默认授权**（§5）：贡献被默认视为 Apache 2.0 授权 · **无需单独 CLA 签署**
3. **Rust 主流**：Rust 核心工具链 + 主流 crates（tokio / serde / clap）均用 Apache 2.0 或 MIT-OR-Apache 双许可 · 零兼容性摩擦
4. **用户可信度**：企业用户 / 商业分发场景下 Apache 2.0 是"最熟悉"的选项 · 法务审查成本低

## 后果

### 正面

- **零 CLA 阻力**：AI agent（Claude / Codex / 其他）提交的 patch 天然合规 · 不需账户认证
- **专利防御**：贡献者若起诉用户专利 → 自动丧失其贡献的使用权 · 企业贡献者 incentive-aligned
- **品牌保护**：许可证要求保留 NOTICE 文件 · 二次发行者必须注明上游
- **生态顺畅**：兼容 MIT / BSD / Apache 2.0 依赖 · 不能兼容 GPL 依赖

### 负面

- **不是 copyleft**：衍生作品不强制开源 · 若目标是"强制分享"则不适合（但本项目目标不是）
- **NOTICE 文件维护开销**：第三方依赖若要求归属声明，需要维护 NOTICE（本项目已建 `NOTICE` 文件）

### 风险

- **引入 GPL 依赖**：若未来不慎引入 GPL 库 → 本项目必须改 GPL 或移除依赖 · 规避手段：`.github/workflows` 加 license-check（Phase 5 scope · 当前人工审）
- **商标**：Apache 2.0 不涵盖商标 · "Vibestation" 名字的商标注册需另行处理（Phase 5+）

## 与 `implementation-plan.md` 的映射

- 对应章节：§11.1 License 与贡献框架
- 对应风险：无（选型稳定 · 无 R 标号）

## 相关

- `CLAUDE.md` 决策表：#1
- 仓库根：`LICENSE`（Apache 2.0 全文）· `NOTICE`（归属声明）
- 贡献流程：`CONTRIBUTING.md`（Phase 3 建立）

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code（Phase 3 · 把 Phase 1 锁定决策正式化为 ADR）
