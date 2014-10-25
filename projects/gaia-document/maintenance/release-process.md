# 发布流程

本文档描述 Gaia Assembler 项目的版本发布流程和规范。

## 版本策略

### 语义化版本

我们遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/) 规范：

- **主版本号 (MAJOR)**: 不兼容的 API 修改
- **次版本号 (MINOR)**: 向下兼容的功能性新增
- **修订号 (PATCH)**: 向下兼容的问题修正

### 版本分支策略

```
main (稳定版本)
├── develop (开发分支)
├── release/v0.3.0 (发布分支)
└── hotfix/v0.2.1 (热修复分支)
```

## 发布准备

### 1. 版本规划

- 确定发布范围和功能
- 创建 GitHub Milestone
- 分配 Issue 和 PR 到 Milestone

### 2. 代码冻结

```bash
# 创建发布分支
git checkout develop
git pull origin develop
git checkout -b release/v0.3.0
git push origin release/v0.3.0
```

### 3. 版本号更新

更新以下文件中的版本号：

```bash
# 更新 Cargo.toml 文件
find . -name "Cargo.toml" -exec sed -i 's/version = "0.2.0"/version = "0.3.0"/g' {} \;

# 更新文档版本
sed -i 's/version: "0.2"/version: "0.3"/g' .vitepress/config.ts
```

### 4. 变更日志

更新 `CHANGELOG.md`：

```markdown
## [0.3.0] - 2024-01-15

### Added
- 新增 PE 后端支持
- 添加调试信息生成
- 支持增量编译

### Changed
- 优化 JVM 字节码生成
- 改进错误信息显示

### Fixed
- 修复 CLR 后端内存泄漏
- 解决符号解析问题

### Deprecated
- 废弃旧的 API 接口

### Removed
- 移除过时的实验性功能

### Security
- 修复潜在的安全漏洞
```

## 发布执行

### 1. 质量检查

```bash
# 运行完整测试套件
cargo test --all

# 代码格式检查
cargo fmt --check

# 代码质量检查
cargo clippy -- -D warnings

# 文档生成测试
cargo doc --no-deps

# 基准测试
cargo bench
```

### 2. 构建验证

```bash
# 清理构建
cargo clean

# 发布构建
cargo build --release --all

# 交叉编译验证
cargo build --target x86_64-pc-windows-gnu
cargo build --target x86_64-unknown-linux-gnu
cargo build --target x86_64-apple-darwin
```

### 3. 集成测试

```bash
# 运行集成测试
cargo test --test integration_tests

# 端到端测试
./scripts/e2e-tests.sh

# 性能回归测试
./scripts/performance-tests.sh
```

### 4. 文档更新

```bash
# 生成 API 文档
cargo doc --all --no-deps

# 构建用户文档
cd gaia-document
npm run build

# 部署文档预览
npm run preview
```

## 发布部署

### 1. 创建发布标签

```bash
# 合并到主分支
git checkout main
git merge release/v0.3.0

# 创建标签
git tag -a v0.3.0 -m "Release version 0.3.0"
git push origin main
git push origin v0.3.0
```

### 2. GitHub Release

1. 访问 GitHub Releases 页面
2. 点击 "Create a new release"
3. 选择标签 `v0.3.0`
4. 填写发布标题和说明
5. 上传构建产物
6. 发布 Release

### 3. 包发布

```bash
# 发布到 crates.io
cargo publish -p gaia-types
cargo publish -p gaia-assembler
cargo publish -p pe-assembler
cargo publish -p clr-assembler
cargo publish -p jvm-assembler
cargo publish -p wasm-assembler
```

### 4. 文档部署

```bash
# 部署文档到 GitHub Pages
cd gaia-document
npm run deploy
```

## 发布后维护

### 1. 监控和反馈

- 监控下载量和使用情况
- 收集用户反馈和问题报告
- 跟踪性能指标

### 2. 热修复流程

如果发现严重问题：

```bash
# 创建热修复分支
git checkout v0.3.0
git checkout -b hotfix/v0.3.1

# 修复问题
# ... 进行必要的修复 ...

# 发布热修复版本
git tag -a v0.3.1 -m "Hotfix version 0.3.1"
git push origin hotfix/v0.3.1
git push origin v0.3.1

# 合并回主分支和开发分支
git checkout main
git merge hotfix/v0.3.1
git checkout develop
git merge hotfix/v0.3.1
```

### 3. 版本支持策略

- **当前版本**: 完全支持，包括新功能和修复
- **前一个主版本**: 安全修复和严重错误修复
- **更早版本**: 仅安全修复

## 发布检查清单

### 发布前检查

- [ ] 所有计划功能已完成
- [ ] 所有测试通过
- [ ] 文档已更新
- [ ] 变更日志已完成
- [ ] 版本号已更新
- [ ] 性能测试通过
- [ ] 安全扫描通过

### 发布时检查

- [ ] 发布分支已创建
- [ ] 标签已创建并推送
- [ ] GitHub Release 已发布
- [ ] 包已发布到 crates.io
- [ ] 文档已部署

### 发布后检查

- [ ] 下载链接可用
- [ ] 文档网站正常
- [ ] 示例代码可运行
- [ ] 社区通知已发送
- [ ] 监控系统正常

## 回滚策略

如果发布出现严重问题：

1. **立即响应**: 在 GitHub 上标记问题版本
2. **通信**: 通知用户和社区
3. **修复**: 快速修复或回滚到稳定版本
4. **验证**: 确保修复版本正常工作
5. **重新发布**: 发布修复版本

## 相关资源

- [语义化版本规范](https://semver.org/)
- [GitHub Flow](https://guides.github.com/introduction/flow/)
- [Cargo 发布指南](https://doc.rust-lang.org/cargo/reference/publishing.html)