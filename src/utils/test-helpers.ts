/**
 * テスト用共通ヘルパー関数
 *
 * PR#66レビューで指摘された重複コードを共通化:
 * - resolveOverlayScriptPath(): オーバーレイスクリプトのパス解決
 * - createTestDOM(): JSDOMインスタンス生成
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { JSDOM, type DOMWindow } from 'jsdom';

/**
 * オーバーレイスクリプトのパスを解決
 *
 * import.meta.urlベースの解決を優先し、失敗時はprocess.cwdにフォールバック
 *
 * @param relativePath - プロジェクトルートからの相対パス
 * @returns 解決された絶対パス
 *
 * @example
 * const scriptPath = resolveOverlayScriptPath('src-tauri/overlays/shared/overlay-core.js');
 */
export function resolveOverlayScriptPath(relativePath: string): string {
  try {
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    // src/utils/test-helpers.ts → ルートディレクトリへ
    const rootDir = path.resolve(__dirname, '../..');
    const scriptPath = path.join(rootDir, relativePath);
    if (fs.existsSync(scriptPath)) {
      return scriptPath;
    }
  } catch {
    // fileURLToPathが失敗した場合はフォールバック
  }

  // フォールバック: process.cwdベース
  return path.join(process.cwd(), relativePath);
}

/**
 * スクリプトファイルの内容を読み込む
 *
 * @param relativePath - プロジェクトルートからの相対パス
 * @returns スクリプトの内容
 * @throws ファイルが見つからない、または読み込めない場合
 */
export function loadScriptContent(relativePath: string): string {
  const scriptPath = resolveOverlayScriptPath(relativePath);
  try {
    return fs.readFileSync(scriptPath, 'utf-8');
  } catch (error) {
    throw new Error(
      `Failed to load script: ${scriptPath} (${error instanceof Error ? error.message : String(error)})`
    );
  }
}

/**
 * テスト用JSDOMインスタンスを生成
 *
 * @param options - JSDOM設定オプション
 * @returns JSDOMインスタンス
 */
export function createTestDOM(options: {
  html?: string;
  runScripts?: boolean;
} = {}): JSDOM {
  const {
    html = '<!DOCTYPE html><html><body></body></html>',
    runScripts = true,
  } = options;

  return new JSDOM(html, {
    runScripts: runScripts ? 'dangerously' : undefined,
    url: 'http://localhost/',
  });
}

/**
 * JSDOMのwindowにperformanceモックを設定
 *
 * window.performanceはread-onlyのため、Object.definePropertyで上書きする必要がある
 *
 * @param window - JSDOMのwindowオブジェクト
 */
export function mockPerformance(window: DOMWindow): void {
  Object.defineProperty(window, 'performance', {
    value: {
      now: () => Date.now(),
    },
    writable: true,
    configurable: true,
  });
}

/**
 * JSDOMのwindowにrequestAnimationFrameモックを設定
 *
 * @param window - JSDOMのwindowオブジェクト
 */
export function mockRequestAnimationFrame(window: DOMWindow): void {
  (window as unknown as { requestAnimationFrame: (cb: () => void) => number }).requestAnimationFrame = (cb) => {
    return setTimeout(cb, 0) as unknown as number;
  };
}

/**
 * JSDOMにスクリプトを実行
 *
 * @param dom - JSDOMインスタンス
 * @param scriptContent - 実行するスクリプト内容
 */
export function executeScript(dom: JSDOM, scriptContent: string): void {
  const script = dom.window.document.createElement('script');
  script.textContent = scriptContent;
  dom.window.document.body.appendChild(script);
}
