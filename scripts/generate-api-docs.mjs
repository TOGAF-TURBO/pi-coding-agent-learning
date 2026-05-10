#!/usr/bin/env node
/**
 * 从已注释的源文件生成 API 参考文档
 *
 * 用法：node scripts/generate-api-docs.mjs
 * 输出：docs/api/{包名}.md
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { generateDocumentation } from "tsdoc-markdown";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const DOCS_API = resolve(ROOT, "docs", "api");

mkdirSync(DOCS_API, { recursive: true });

const GITHUB_REPO = "https://github.com/earendil-works/pi-mono/blob/main";

/** 核心文件的包级配置 */
const PACKAGES = [
  {
    name: "pi-ai",
    files: [
      "packages/ai/src/types.ts",
      "packages/ai/src/stream.ts",
      "packages/ai/src/models.ts",
      "packages/ai/src/api-registry.ts",
    ],
  },
  {
    name: "pi-agent-core",
    files: [
      "packages/agent/src/types.ts",
      "packages/agent/src/agent.ts",
      "packages/agent/src/agent-loop.ts",
    ],
  },
  {
    name: "pi-coding-agent",
    files: [
      "packages/coding-agent/src/cli.ts",
      "packages/coding-agent/src/main.ts",
      "packages/coding-agent/src/core/sdk.ts",
      "packages/coding-agent/src/core/agent-session.ts",
      "packages/coding-agent/src/core/session-manager.ts",
      "packages/coding-agent/src/core/settings-manager.ts",
      "packages/coding-agent/src/core/model-registry.ts",
      "packages/coding-agent/src/core/compaction/compaction.ts",
      "packages/coding-agent/src/core/extensions/types.ts",
      "packages/coding-agent/src/core/extensions/loader.ts",
      "packages/coding-agent/src/core/tools/index.ts",
      "packages/coding-agent/src/core/tools/read.ts",
      "packages/coding-agent/src/core/tools/write.ts",
      "packages/coding-agent/src/core/tools/edit.ts",
      "packages/coding-agent/src/core/tools/bash.ts",
      "packages/coding-agent/src/core/tools/find.ts",
      "packages/coding-agent/src/core/tools/grep.ts",
      "packages/coding-agent/src/modes/print-mode.ts",
      "packages/coding-agent/src/modes/rpc/rpc-mode.ts",
      "packages/coding-agent/src/modes/rpc/rpc-types.ts",
    ],
  },
  {
    name: "pi-tui",
    files: [
      "packages/tui/src/tui.ts",
      "packages/tui/src/components/editor.ts",
      "packages/tui/src/components/markdown.ts",
      "packages/tui/src/components/select-list.ts",
      "packages/tui/src/keys.ts",
      "packages/tui/src/utils.ts",
    ],
  },
];

/** 每个包的供应商文件单独生成 */
const AI_PROVIDERS = {
  name: "pi-ai-providers",
  files: [
    "packages/ai/src/providers/anthropic.ts",
    "packages/ai/src/providers/openai-completions.ts",
    "packages/ai/src/providers/openai-responses.ts",
    "packages/ai/src/providers/google.ts",
    "packages/ai/src/providers/mistral.ts",
    "packages/ai/src/providers/amazon-bedrock.ts",
    "packages/ai/src/providers/faux.ts",
    "packages/ai/src/providers/transform-messages.ts",
    "packages/ai/src/providers/register-builtins.ts",
  ],
};

console.log("生成 API 参考文档...\n");

for (const pkg of PACKAGES) {
  const dest = resolve(DOCS_API, `${pkg.name}.md`);
  console.log(`  ${pkg.name} → ${pkg.files.length} 个文件`);
  try {
    generateDocumentation({
      inputFiles: pkg.files.map((f) => resolve(ROOT, f)),
      outputFile: dest,
      markdownOptions: { repo: GITHUB_REPO },
      buildOptions: { types: true, explore: true },
    });
    console.log(`    ✓ ${dest}`);
  } catch (e) {
    console.error(`    ✗ ${pkg.name}: ${e.message}`);
  }
}

// 供应商模块
console.log(`\n  ${AI_PROVIDERS.name} → ${AI_PROVIDERS.files.length} 个文件`);
try {
  generateDocumentation({
    inputFiles: AI_PROVIDERS.files.map((f) => resolve(ROOT, f)),
    outputFile: resolve(DOCS_API, `${AI_PROVIDERS.name}.md`),
    markdownOptions: { repo: GITHUB_REPO },
    buildOptions: { types: true, explore: true },
  });
  console.log(`    ✓ ${resolve(DOCS_API, `${AI_PROVIDERS.name}.md`)}`);
} catch (e) {
  console.error(`    ✗ ${AI_PROVIDERS.name}: ${e.message}`);
}

// 生成索引
const index = [
  "# pi API 参考文档",
  "",
  "由 [tsdoc-markdown](https://github.com/peterpeterparker/tsdoc-markdown) 从源码 JSDoc 注释自动生成。",
  "",
  "## 核心包",
  "",
  ...PACKAGES.map((p) => `- [${p.name}](./${p.name}.md) — ${p.files.length} 个源文件`),
  "",
  "## AI 供应商实现",
  "",
  `- [${AI_PROVIDERS.name}](./${AI_PROVIDERS.name}.md) — ${AI_PROVIDERS.files.length} 个源文件`,
  "",
  "## 相关文档",
  "",
  "- [术语索引](../terminology-index.md)",
  "- [能力分析报告](../capability-analysis.md)",
  "- [能力路径分析](../capability-paths.md)",
  "- [阅读路线图](../reading-roadmap.md)",
  "",
].join("\n");

writeFileSync(resolve(DOCS_API, "index.md"), index);
console.log(`\n  ✓ 索引 → ${resolve(DOCS_API, "index.md")}`);
console.log("\n完成！");
