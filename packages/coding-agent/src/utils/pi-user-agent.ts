/**
 * @fileoverview Pi User-Agent — 构建 pi 的 HTTP User-Agent 字符串。
 */

export function getPiUserAgent(version: string): string {
	const runtime = process.versions.bun ? `bun/${process.versions.bun}` : `node/${process.version}`;
	return `pi/${version} (${process.platform}; ${runtime}; ${process.arch})`;
}
