/**
 * @fileoverview MIME — 文件 MIME 类型检测。
 *
 * 通过 magic bytes 检测文件的 MIME 类型：
 *   - 支持常见图片格式（jpg、png、gif、webp）
 *   - 使用 file-type 库进行精确检测
 *   - 用于 Read 工具的图片文件识别
 */
import { open } from "node:fs/promises";
import { fileTypeFromBuffer } from "file-type";

const IMAGE_MIME_TYPES = new Set(["image/jpeg", "image/png", "image/gif", "image/webp"]);

const FILE_TYPE_SNIFF_BYTES = 4100;

export async function detectSupportedImageMimeTypeFromFile(filePath: string): Promise<string | null> {
	const fileHandle = await open(filePath, "r");
	try {
		const buffer = Buffer.alloc(FILE_TYPE_SNIFF_BYTES);
		const { bytesRead } = await fileHandle.read(buffer, 0, FILE_TYPE_SNIFF_BYTES, 0);
		if (bytesRead === 0) {
			return null;
		}

		const fileType = await fileTypeFromBuffer(buffer.subarray(0, bytesRead));
		if (!fileType) {
			return null;
		}

		if (!IMAGE_MIME_TYPES.has(fileType.mime)) {
			return null;
		}

		return fileType.mime;
	} finally {
		await fileHandle.close();
	}
}
