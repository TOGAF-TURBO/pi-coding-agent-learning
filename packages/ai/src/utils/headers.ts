/**
 * @fileoverview HTTP 头部工具 — 构建和解析 HTTP 请求头。
 */

export function headersToRecord(headers: Headers): Record<string, string> {
	const result: Record<string, string> = {};
	for (const [key, value] of headers.entries()) {
		result[key] = value;
	}
	return result;
}
