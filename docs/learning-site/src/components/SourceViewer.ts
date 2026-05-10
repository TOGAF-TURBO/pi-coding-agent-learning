import { LitElement, html, css } from "lit";
import { customElement, property, state } from "lit/decorators.js";

@customElement("source-viewer")
export class SourceViewer extends LitElement {
  static styles = css`
    :host {
      display: block;
      background: var(--sl-color-black, #0e0e1e);
      border: 1px solid var(--sl-color-hairline, rgba(124, 138, 255, 0.15));
      border-radius: 8px;
      overflow: hidden;
      font-family: "JetBrains Mono", "Fira Code", "Cascadia Code", monospace;
      font-size: 0.8125rem;
      position: relative;
    }

    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 0.5em 1em;
      background: var(--sl-color-gray-5, #242438);
      border-bottom: 1px solid var(--sl-color-hairline, rgba(124, 138, 255, 0.15));
    }

    .filename {
      color: var(--sl-color-accent-high, #b4c0ff);
      font-size: 0.8125rem;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .close-btn {
      background: none;
      border: 1px solid var(--sl-color-gray-3, #606078);
      color: var(--sl-color-gray-2, #a0a0b0);
      cursor: pointer;
      padding: 0.15em 0.6em;
      border-radius: 4px;
      font-size: 0.75rem;
      font-family: inherit;
      flex-shrink: 0;
    }

    .close-btn:hover {
      color: var(--sl-color-white, #e8e8f0);
      border-color: var(--sl-color-accent, #7c8aff);
    }

    .loading {
      padding: 2em 1em;
      text-align: center;
      color: var(--sl-color-gray-2, #a0a0b0);
    }

    .error {
      padding: 1em;
      color: #ff6b6b;
      background: rgba(255, 107, 107, 0.08);
    }

    .code-container {
      overflow-x: auto;
      overflow-y: auto;
      max-height: 420px;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      margin: 0;
    }

    tr {
      line-height: 1.55;
    }

    .line-num {
      color: var(--sl-color-gray-3, #606078);
      text-align: right;
      padding: 0 0.8em 0 0.6em;
      user-select: none;
      min-width: 3em;
      white-space: nowrap;
      vertical-align: top;
      border-right: 1px solid rgba(124, 138, 255, 0.08);
      font-size: 0.75rem;
    }

    .line-content {
      padding: 0 0.8em;
      white-space: pre;
      color: var(--sl-color-gray-1, #c8c8d8);
    }

    .truncated-note {
      padding: 0.8em 1em;
      color: var(--sl-color-gray-2, #a0a0b0);
      font-size: 0.75rem;
      border-top: 1px solid var(--sl-color-hairline, rgba(124, 138, 255, 0.15));
      background: var(--sl-color-gray-5, #242438);
      text-align: center;
    }

    .comment-cn {
      color: #6a9955;
    }

    .keyword {
      color: #569cd6;
    }

    .string {
      color: #ce9178;
    }

    .function {
      color: #dcdcaa;
    }

    .type {
      color: #4ec9b0;
    }

    .number {
      color: #b5cea8;
    }
  `;

  @property({ type: String }) filePath = "";

  @state() private _content = "";
  @state() private _loading = false;
  @state() private _error = "";
  @state() private _totalLines = 0;

  async _fetchSource() {
    if (!this.filePath) return;
    this._loading = true;
    this._error = "";
    try {
      const resp = await fetch(`/api/source?path=${encodeURIComponent(this.filePath)}`);
      if (!resp.ok) {
        const err = await resp.json().catch(() => ({}));
        this._error = (err as { error?: string }).error || `HTTP ${resp.status}`;
        return;
      }
      const data = await resp.json() as { content: string; totalLines: number };
      this._content = data.content;
      this._totalLines = data.totalLines;
    } catch (e) {
      this._error = (e as Error).message;
    } finally {
      this._loading = false;
    }
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has("filePath") && this.filePath) {
      this._fetchSource();
    }
  }

  _highlightLine(line: string): ReturnType<typeof html> {
    const cnComment = /\/\/\s*[\u4e00-\u9fff]/.test(line);
    if (cnComment) {
      return html`<span class="comment-cn">${line}</span>`;
    }
    return html`${line}`;
  }

  _handleClose() {
    this.dispatchEvent(new CustomEvent("source-viewer-close", { bubbles: true, composed: true }));
    this.remove();
  }

  render() {
    const lines = this._content ? this._content.split("\n") : [];

    return html`
      <div class="header">
        <span class="filename" title=${this.filePath}>${this.filePath}</span>
        <button class="close-btn" @click=${this._handleClose}>关闭</button>
      </div>

      ${this._loading
        ? html`<div class="loading">加载中...</div>`
        : this._error
          ? html`<div class="error">${this._error}</div>`
          : html`
              <div class="code-container">
                <table>
                  <tbody>
                    ${lines.map(
                      (line, i) => html`
                        <tr>
                          <td class="line-num">${i + 1}</td>
                          <td class="line-content">${this._highlightLine(line)}</td>
                        </tr>
                      `
                    )}
                  </tbody>
                </table>
              </div>
              ${this._totalLines > lines.length
                ? html`<div class="truncated-note">（仅显示前 ${lines.length} 行，文件共 ${this._totalLines} 行）</div>`
                : ""}
            `}
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "source-viewer": SourceViewer;
  }
}
