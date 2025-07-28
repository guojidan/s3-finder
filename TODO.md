# S3 Finder - 开发任务清单

## 项目现状分析 📊

### ✅ **已实现功能 (约20%完成度)**
- **本地文件管理**: 完整的本地文件系统操作
  - 目录浏览、创建文件夹、删除、重命名、复制/移动
  - 文件搜索和预览 (文本/图片，最大10MB)
  - 安全路径验证，防止目录遍历攻击
- **基础UI**: Leptos 单列文件浏览器
- **Tauri后端**: 完整的文件操作API

### ❌ **核心功能缺失 (80%待实现)**
- **S3集成**: 完全缺失 - 无AWS SDK依赖，无S3相关代码
- **Miller列视图**: 当前单列 vs 目标三列布局
- **拖拽操作**: 无任何拖拽功能实现
- **统一存储接口**: 数据模型不支持S3对象
- **多账户管理**: 无账户配置和管理

### 🎯 **与README.md目标差距**
- **实际**: 本地文件管理器
- **目标**: S3对象存储管理器
- **差距**: 需要从零开始构建S3集成

---

## 核心功能重构计划 🚀

### 阶段1: S3基础架构 (关键路径 - 4周)

#### 1.1 依赖和SDK集成
- [ ] **添加AWS SDK依赖**
  ```toml
  aws-sdk-s3 = "1.0"
  aws-config = "1.0"
  tokio-stream = "0.1"
  ```
- [ ] **S3客户端配置**
  - [ ] 实现S3客户端初始化
  - [ ] 支持多种认证方式 (credentials, IAM, profile)
  - [ ] 区域和端点配置

#### 1.2 数据模型重构
- [ ] **扩展FileItem结构体**
  ```rust
  pub enum StorageType { Local, S3 }
  pub struct FileItem {
      // 现有字段...
      pub storage_type: StorageType,
      pub bucket: Option<String>,
      pub key: Option<String>,
      pub storage_class: Option<String>,
  }
  ```
- [ ] **统一存储接口**
  - [ ] 创建`StorageProvider` trait
  - [ ] 实现`LocalStorage`和`S3Storage`

#### 1.3 S3服务层实现
- [ ] **创建s3_service.rs模块**
  - [ ] `list_buckets()` - 列举buckets
  - [ ] `list_objects()` - 列举对象
  - [ ] `get_object_metadata()` - 获取元数据
  - [ ] `upload_object()` - 上传对象
  - [ ] `download_object()` - 下载对象
  - [ ] `delete_object()` - 删除对象

#### 1.4 Tauri命令扩展
- [ ] **添加S3相关命令**
  - [ ] `connect_s3_account`
  - [ ] `list_s3_buckets`
  - [ ] `read_s3_directory`
  - [ ] `upload_to_s3`
  - [ ] `download_from_s3`

### 阶段2: Miller列视图重构 (关键路径 - 3周)

#### 2.1 UI架构重设计
- [ ] **重构app.rs组件结构**
  - [ ] 移除单列布局
  - [ ] 实现三列Miller视图组件
  - [ ] 添加列间导航逻辑
- [ ] **状态管理重构**
  ```rust
  pub struct ColumnState {
      pub storage_type: StorageType,
      pub path: String,
      pub items: Vec<FileItem>,
  }
  pub struct AppState {
      pub columns: [Option<ColumnState>; 3],
      pub active_column: usize,
  }
  ```

#### 2.2 侧边栏重新设计
- [ ] **存储位置管理**
  - [ ] 本地收藏夹 (Home, Applications, etc.)
  - [ ] S3账户和bucket列表
  - [ ] 存储位置切换逻辑

#### 2.3 导航和面包屑
- [ ] **路径导航系统**
  - [ ] 统一路径表示 (local:// 和 s3://)
  - [ ] 面包屑导航组件
  - [ ] 历史记录和前进/后退

### 阶段3: 文件传输核心 (关键路径 - 3周)

#### 3.1 拖拽系统实现
- [ ] **HTML5拖拽API集成**
  - [ ] 拖拽事件处理 (dragstart, dragover, drop)
  - [ ] 拖拽视觉反馈
  - [ ] 跨列拖拽支持

#### 3.2 传输引擎
- [ ] **传输管理器**
  ```rust
  pub struct TransferManager {
      pub queue: Vec<TransferTask>,
      pub active_transfers: HashMap<String, TransferProgress>,
  }
  pub enum TransferType {
      LocalToS3, S3ToLocal, S3ToS3, LocalToLocal
  }
  ```
- [ ] **进度跟踪**
  - [ ] 实时进度更新
  - [ ] 传输速度计算
  - [ ] 剩余时间估算

#### 3.3 错误处理和重试
- [ ] **网络错误处理**
  - [ ] 自动重试机制
  - [ ] 断点续传支持
  - [ ] 传输失败恢复

### 阶段4: 账户和配置管理 (2周)

#### 4.1 S3账户管理
- [ ] **账户配置界面**
  - [ ] 添加/编辑/删除S3账户
  - [ ] 凭证安全存储 (keychain/credential manager)
  - [ ] 连接测试和验证

#### 4.2 多账户支持
- [ ] **账户切换**
  - [ ] 侧边栏账户列表
  - [ ] 账户状态指示器
  - [ ] 并发多账户操作

### 阶段5: 搜索和预览增强 (2周)

#### 5.1 统一搜索
- [ ] **跨存储搜索**
  - [ ] 本地文件系统搜索 (现有)
  - [ ] S3对象搜索 (使用S3 API)
  - [ ] 搜索结果合并和排序

#### 5.2 S3对象预览
- [ ] **无下载预览**
  - [ ] S3对象流式预览
  - [ ] 预览缓存机制
  - [ ] 支持更多文件类型

### 阶段6: 性能和用户体验 (2周)

#### 6.1 性能优化
- [ ] **虚拟滚动**
  - [ ] 大量文件列表优化
  - [ ] 懒加载实现
- [ ] **API调用优化**
  - [ ] S3请求批处理
  - [ ] 智能预加载
  - [ ] 缓存策略

#### 6.2 用户体验
- [ ] **快捷键支持**
- [ ] **主题切换**
- [ ] **多标签页**

---

## 技术债务和质量保证 📋

### 测试策略
- [ ] **单元测试** (S3服务层优先)
- [ ] **集成测试** (使用MinIO本地测试)
- [ ] **UI测试** (Miller列视图交互)

### 代码质量
- [ ] **错误处理标准化**
- [ ] **日志记录系统**
- [ ] **性能监控**

---

## 里程碑和时间线 🗓️

### 里程碑1: S3基础功能 (4周)
**目标**: 能够连接S3并在Miller列视图中浏览
- S3 SDK集成和基础API
- Miller列视图重构
- 基本的S3浏览功能

### 里程碑2: 文件传输 (3周)  
**目标**: 实现本地和S3之间的文件传输
- 拖拽操作实现
- 传输管理器
- 进度跟踪

### 里程碑3: 完整功能 (3周)
**目标**: 多账户管理和高级功能
- 账户管理界面
- 搜索和预览增强
- 性能优化

### 里程碑4: 质量和发布 (2周)
**目标**: 测试覆盖和发布准备
- 测试套件完善
- 文档更新
- 发布准备

**总计**: 12周完成核心S3 Finder功能

---

## 风险和依赖 ⚠️

### 技术风险
- **AWS SDK学习曲线**: Rust AWS SDK相对较新
- **UI重构复杂度**: Miller列视图需要重大架构改变
- **性能挑战**: 大文件传输和大量对象列表

### 外部依赖
- **AWS凭证**: 需要有效的AWS账户进行测试
- **网络环境**: S3操作需要稳定网络连接
- **MinIO**: 本地测试环境搭建

**注意**: 此计划专注于将现有本地文件管理器转换为真正的S3 Finder，实现README.md中描述的核心目标。