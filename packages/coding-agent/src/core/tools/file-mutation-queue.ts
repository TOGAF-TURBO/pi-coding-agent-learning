/**
 * @fileoverview File Mutation Queue — 文件变更的顺序执行队列。
 *
 * 确保 edit/write 工具对同一文件的变更顺序执行，
 * 避免并发写入导致的数据竞争。每个文件路径对应一个独立的队列。
 */
import { realpathSync } from "node:fs";
import { resolve } from "node:path";

const fileMutationQueues = new Map<string, Promise<void>>();

function getMutationQueueKey(filePath: string): string {
	const resolvedPath = resolve(filePath);
	try {
		return realpathSync.native(resolvedPath);
	} catch {
		return resolvedPath;
	}
}

/**
 * Serialize file mutation operations targeting the same file.
 * Operations for different files still run in parallel.
 */
export async function withFileMutationQueue<T>(filePath: string, fn: () => Promise<T>): Promise<T> {
	const key = getMutationQueueKey(filePath);
	const currentQueue = fileMutationQueues.get(key) ?? Promise.resolve();

	let releaseNext!: () => void;
	const nextQueue = new Promise<void>((resolveQueue) => {
		releaseNext = resolveQueue;
	});
	const chainedQueue = currentQueue.then(() => nextQueue);
	fileMutationQueues.set(key, chainedQueue);

	await currentQueue;
	try {
		return await fn();
	} finally {
		releaseNext();
		if (fileMutationQueues.get(key) === chainedQueue) {
			fileMutationQueues.delete(key);
		}
	}
}
