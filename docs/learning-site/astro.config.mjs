import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { defineConfig } from "vite";
import starlight from "@astrojs/starlight";
import lit from "@astrojs/lit";

const PROJECT_ROOT = join(import.meta.dirname, "..", "..");

export default defineConfig({
  integrations: [
    lit(),
    starlight({
      title: "pi 源代码阅读路线图",
      description: "14 天渐进式源码学习 + API 参考文档",
      defaultLocale: "zh-CN",
      customCss: ["./src/styles/custom.css"],
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/earendil-works/pi-mono" },
      ],
      sidebar: [
        { label: "概述", slug: "" },
        {
          label: "L1 入门 — 核心类型和 LLM 调用",
          collapsed: false,
          items: [
            { label: "Day 1: 核心类型定义", slug: "day-1" },
            { label: "Day 2: 流式调用入口", slug: "day-2" },
            { label: "Day 3: 模型查找与成本计算", slug: "day-3" },
          ],
        },
        {
          label: "L2 熟练 — Agent 循环和工具执行",
          collapsed: true,
          items: [
            { label: "Day 4: Agent 主类", slug: "day-4" },
            { label: "Day 5: Agent 循环核心", slug: "day-5" },
            { label: "Day 6: 工具执行", slug: "day-6" },
          ],
        },
        {
          label: "L3 精通 — 扩展、会话、压缩",
          collapsed: true,
          items: [
            { label: "Day 7: 扩展系统", slug: "day-7" },
            { label: "Day 8: 会话管理", slug: "day-8" },
            { label: "Day 9: 上下文压缩", slug: "day-9" },
            { label: "Day 10: 消息转换与 Provider 适配", slug: "day-10" },
          ],
        },
        {
          label: "L4 专家 — TUI、RPC、Provider 实现",
          collapsed: true,
          items: [
            { label: "Day 11: TUI 引擎", slug: "day-11" },
            { label: "Day 12: 编辑器组件", slug: "day-12" },
            { label: "Day 13: RPC 协议", slug: "day-13" },
            { label: "Day 14: Provider 实现全景对比", slug: "day-14" },
          ],
        },
        {
          label: "API 参考",
          collapsed: true,
          items: [
            { label: "pi-ai — AI 接口层", slug: "api/pi-ai" },
            { label: "pi-ai Suppliers", slug: "api/pi-ai-providers" },
            { label: "pi-agent-core — Agent 运行时", slug: "api/pi-agent-core" },
            { label: "pi-coding-agent — 编码助手", slug: "api/pi-coding-agent" },
            { label: "pi-tui — 终端 UI", slug: "api/pi-tui" },
          ],
        },
        {
          label: "分析文档",
          collapsed: true,
          items: [
            { label: "架构文档", slug: "guides/architecture" },
            { label: "能力分析报告", slug: "guides/capability-analysis" },
            { label: "能力路径分析", slug: "guides/capability-paths" },
            { label: "程序 = 数据结构 + 算法", slug: "guides/program-as-data-structures-algorithms" },
            { label: "阅读路线图（完整版）", slug: "guides/reading-roadmap" },
            { label: "术语索引", slug: "guides/terminology-index" },
          ],
        },
      ],
    }),
  ],
  vite: {
    plugins: [
      {
        name: "source-files",
        configureServer(server) {
          server.middlewares.use("/api/source", (req, res) => {
            const url = new URL(req.url, `http://${req.headers.host || 'localhost' || 'localhost'}`);
            const filePath = url.searchParams.get("path");
            if (!filePath) {
              res.statusCode = 400;
              res.end(JSON.stringify({ error: "Missing path parameter" }));
              return;
            }
            const colonIdx = filePath.lastIndexOf(":");
            const pathPart = colonIdx > 0 ? filePath.slice(0, colonIdx) : filePath;
            const fullPath = join(PROJECT_ROOT, pathPart);
            if (!existsSync(fullPath)) {
              res.statusCode = 404;
              res.end(JSON.stringify({ error: `File not found: ${pathPart}` }));
              return;
            }
            const content = readFileSync(fullPath, "utf-8");
            const maxLines = 600;
            const lines = content.split("\n");
            const truncated = lines.length > maxLines
              ? lines.slice(0, maxLines).join("\n") + `\n\n// ... (${lines.length - maxLines} 更多行)`
              : content;
            res.setHeader("Content-Type", "application/json");
            res.end(JSON.stringify({ content: truncated, totalLines: lines.length }));
          });
        },
      },
    ],
  },
});
