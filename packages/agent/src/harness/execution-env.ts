/**
 * @fileoverview ExecutionEnv 重导出 — 统一导出 NodeExecutionEnv 和浏览器安全的类型。
 */

export { NodeExecutionEnv } from "./env/nodejs.js";
export type { ExecutionEnv, ExecutionEnvExecOptions, FileErrorCode, FileInfo, FileKind } from "./types.js";
export { FileError } from "./types.js";
