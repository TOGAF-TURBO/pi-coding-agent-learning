/**
 * @fileoverview 组件位置 — 演示 setWidget 的放置选项。
 */

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export default function widgetPlacementExtension(pi: ExtensionAPI) {
	pi.on("session_start", (_event, ctx) => {
		if (!ctx.hasUI) return;
		ctx.ui.setWidget("widget-above", ["Above editor widget"]);
		ctx.ui.setWidget("widget-below", ["Below editor widget"], { placement: "belowEditor" });
	});
}
