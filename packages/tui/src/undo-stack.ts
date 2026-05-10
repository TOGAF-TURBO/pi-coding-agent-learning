/**
 * @fileoverview Undo Stack — 通用撤销栈，支持 clone-on-push 语义。
 *
 * 每次推入新状态时自动克隆，确保撤销到历史状态时
 * 不会因为引用共享而丢失数据。
 */
export class UndoStack<S> {
	private stack: S[] = [];

	/** Push a deep clone of the given state onto the stack. */
	push(state: S): void {
		this.stack.push(structuredClone(state));
	}

	/** Pop and return the most recent snapshot, or undefined if empty. */
	pop(): S | undefined {
		return this.stack.pop();
	}

	/** Remove all snapshots. */
	clear(): void {
		this.stack.length = 0;
	}

	get length(): number {
		return this.stack.length;
	}
}
