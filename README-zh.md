```markdown
# pi 源代码阅读路线图 - 交互式学习站点

基于 [Astro Starlight](https://starlight.astro.build) 构建的文档站点，14 天渐进式源码学习 + API 参考文档。

## 启动

```bash
cd docs/learning-site
npm install
npm run dev       # http://localhost:3000
```

## 构建

```bash
npm run build     # gen-docs → astro build → dist/
```

## 功能

- **⌘K 全文搜索**（Pagefind 索引）
- **侧边栏导航**：L1-L4 四个等级，14 天可折叠
- **API 参考**：从 JSDoc 注释自动生成的函数/类型文档
- **暗色/亮色主题**切换
- **静态生成**：21 个页面，零 JS 首屏

## 内容

| 内容 | 来源 |
|------|------|
| 14 天学习计划 | `docs/reading-roadmap.md` → `src/content/docs/day-N.md` |
| 首页导航 | `src/content/docs/index.mdx` |
| API 参考 | `npm run gen-docs` → `docs/api/` → `src/content/docs/api/` |
| 源码查看器 | `src/components/SourceViewer.ts`（Lit Web Component） |

## 定制

- 侧边栏：`astro.config.mjs` 的 `sidebar` 字段
- 样式：`src/styles/custom.css`
- 组件：`src/components/` 下可添加 Astro/Lit 组件
```