/**
 * weather-widget.js のユニットテスト
 *
 * PR#59で指摘されたWeatherWidgetのテストケース:
 * - デフォルト値のフォールバック
 * - update()でのnull/undefined処理
 * - getIconForCode()のマッピング
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { JSDOM } from 'jsdom';

// スクリプトパスを解決
function resolveScriptPath(): string {
  const relativePath = 'src-tauri/overlays/components/weather-widget.js';

  try {
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    const rootDir = path.resolve(__dirname, '../..');
    const scriptPath = path.join(rootDir, relativePath);
    if (fs.existsSync(scriptPath)) {
      return scriptPath;
    }
  } catch {
    // fileURLToPathが失敗した場合はフォールバック
  }

  return path.join(process.cwd(), relativePath);
}

// BaseComponentのモックスクリプト
const baseComponentMock = `
class BaseComponent {
  constructor(config) {
    this.id = config.id || 'test';
    this.slot = config.slot || 'left.topBelow';
    this.style = config.style || {};
    this.rules = config.rules || {};
    this.el = null;
  }
  createElement(tag, options = {}) {
    const el = document.createElement(tag);
    if (options.className) el.className = options.className;
    if (options.textContent) el.textContent = options.textContent;
    if (options.style) Object.assign(el.style, options.style);
    return el;
  }
  mount() {
    this.el = this.render();
    return this.el;
  }
  destroy() {
    if (this.el && this.el.parentNode) {
      this.el.parentNode.removeChild(this.el);
    }
  }
  render() {
    throw new Error('render() must be implemented');
  }
}
`;

// ComponentRegistryモック
const componentRegistryMock = `
window.ComponentRegistry = {
  _components: new Map(),
  register(name, cls) {
    this._components.set(name, cls);
  },
  get(name) {
    return this._components.get(name);
  }
};
`;

// WeatherWidgetを読み込んで取得
function loadWeatherWidget(): {
  dom: JSDOM;
  WeatherWidget: WeatherWidgetClass;
} {
  const scriptPath = resolveScriptPath();
  const scriptContent = fs.readFileSync(scriptPath, 'utf-8');

  const dom = new JSDOM('<!DOCTYPE html><html><body></body></html>', {
    runScripts: 'dangerously',
    url: 'http://localhost/',
  });

  // BaseComponentとComponentRegistryをセットアップ
  const setupScript = dom.window.document.createElement('script');
  setupScript.textContent = baseComponentMock + componentRegistryMock;
  dom.window.document.body.appendChild(setupScript);

  // WeatherWidgetスクリプトを実行
  const script = dom.window.document.createElement('script');
  script.textContent = scriptContent;
  dom.window.document.body.appendChild(script);

  const WeatherWidget = (
    dom.window as unknown as { ComponentRegistry: { get: (name: string) => WeatherWidgetClass } }
  ).ComponentRegistry.get('WeatherWidget');

  return { dom, WeatherWidget };
}

// 型定義
interface WeatherWidgetClass {
  new (config: WeatherWidgetConfig): WeatherWidgetInstance;
  WEATHER_ICONS: Record<string, string>;
  getIconForCode(code: string): string;
}

interface WeatherWidgetConfig {
  id?: string;
  slot?: string;
  style?: {
    icon?: string;
    temp?: number;
    description?: string;
    location?: string;
  };
  rules?: Record<string, unknown>;
}

interface WeatherWidgetInstance {
  id: string;
  slot: string;
  style: Record<string, unknown>;
  icon: string;
  temp: number;
  description: string;
  location: string;
  iconEl: HTMLElement;
  tempEl: HTMLElement;
  descEl: HTMLElement;
  locationEl: HTMLElement;
  mount(): HTMLElement;
  render(): HTMLElement;
  update(data: Partial<{
    icon: string;
    temp: number;
    description: string;
    location: string | null;
  }>): void;
  destroy(): void;
}

describe('WeatherWidget', () => {
  let dom: JSDOM;
  let WeatherWidget: WeatherWidgetClass;

  beforeEach(() => {
    const loaded = loadWeatherWidget();
    dom = loaded.dom;
    WeatherWidget = loaded.WeatherWidget;
  });

  afterEach(() => {
    dom.window.close();
  });

  describe('constructor デフォルト値', () => {
    it('style未指定時はデフォルト値が使われる', () => {
      const widget = new WeatherWidget({});
      expect(widget.icon).toBe('☀️');
      expect(widget.temp).toBe(25);
      expect(widget.description).toBe('晴れ');
      expect(widget.location).toBe('');
    });

    it('style.icon未指定時は☀️がデフォルト', () => {
      const widget = new WeatherWidget({ style: {} });
      expect(widget.icon).toBe('☀️');
    });

    it('style.temp未指定時は25がデフォルト', () => {
      const widget = new WeatherWidget({ style: {} });
      expect(widget.temp).toBe(25);
    });

    it('style.temp=0は0として扱われる（falsyでもデフォルトにならない）', () => {
      const widget = new WeatherWidget({ style: { temp: 0 } });
      expect(widget.temp).toBe(0);
    });

    it('style.description未指定時は晴れがデフォルト', () => {
      const widget = new WeatherWidget({ style: {} });
      expect(widget.description).toBe('晴れ');
    });

    it('style.location未指定時は空文字がデフォルト', () => {
      const widget = new WeatherWidget({ style: {} });
      expect(widget.location).toBe('');
    });

    it('カスタム値が指定された場合はその値を使用', () => {
      const widget = new WeatherWidget({
        style: {
          icon: '🌧️',
          temp: 15,
          description: '雨',
          location: '東京',
        },
      });
      expect(widget.icon).toBe('🌧️');
      expect(widget.temp).toBe(15);
      expect(widget.description).toBe('雨');
      expect(widget.location).toBe('東京');
    });
  });

  describe('render()', () => {
    it('コンテナ要素を生成する', () => {
      const widget = new WeatherWidget({});
      const el = widget.render();
      expect(el.tagName).toBe('DIV');
      expect(el.classList.contains('weather-widget')).toBe(true);
      expect(el.classList.contains('panel')).toBe(true);
    });

    it('各要素が正しく生成される', () => {
      const widget = new WeatherWidget({
        style: { icon: '⛅', temp: 20, description: 'くもり', location: '大阪' },
      });
      widget.render();

      expect(widget.iconEl.textContent).toBe('⛅');
      expect(widget.tempEl.textContent).toBe('20°C');
      expect(widget.descEl.textContent).toBe('くもり');
      expect(widget.locationEl.textContent).toBe('大阪');
    });

    it('locationElは常に生成される', () => {
      const widget = new WeatherWidget({ style: {} });
      widget.render();
      expect(widget.locationEl).toBeDefined();
      expect(widget.locationEl.textContent).toBe('');
    });
  });

  describe('update()', () => {
    it('iconを更新できる', () => {
      const widget = new WeatherWidget({});
      widget.render();

      widget.update({ icon: '❄️' });
      expect(widget.icon).toBe('❄️');
      expect(widget.iconEl.textContent).toBe('❄️');
    });

    it('tempを更新できる', () => {
      const widget = new WeatherWidget({});
      widget.render();

      widget.update({ temp: 30 });
      expect(widget.temp).toBe(30);
      expect(widget.tempEl.textContent).toBe('30°C');
    });

    it('descriptionを更新できる', () => {
      const widget = new WeatherWidget({});
      widget.render();

      widget.update({ description: '快晴' });
      expect(widget.description).toBe('快晴');
      expect(widget.descEl.textContent).toBe('快晴');
    });

    it('locationを更新できる', () => {
      const widget = new WeatherWidget({});
      widget.render();

      widget.update({ location: '福岡' });
      expect(widget.location).toBe('福岡');
      expect(widget.locationEl.textContent).toBe('福岡');
    });

    it('location=nullは空文字にフォールバック', () => {
      const widget = new WeatherWidget({ style: { location: '東京' } });
      widget.render();

      widget.update({ location: null as unknown as string });
      expect(widget.location).toBe('');
      expect(widget.locationEl.textContent).toBe('');
    });

    it('location=undefinedは更新しない', () => {
      const widget = new WeatherWidget({ style: { location: '東京' } });
      widget.render();

      widget.update({ icon: '🌧️' }); // locationはundefined
      expect(widget.location).toBe('東京'); // 変更されない
    });

    it('複数フィールドを同時に更新できる', () => {
      const widget = new WeatherWidget({});
      widget.render();

      widget.update({
        icon: '⚡',
        temp: 28,
        description: '雷雨',
        location: '名古屋',
      });

      expect(widget.icon).toBe('⚡');
      expect(widget.temp).toBe(28);
      expect(widget.description).toBe('雷雨');
      expect(widget.location).toBe('名古屋');
    });

    it('undefinedフィールドは更新しない', () => {
      const widget = new WeatherWidget({
        style: { icon: '☀️', temp: 25, description: '晴れ', location: '東京' },
      });
      widget.render();

      widget.update({ temp: 30 });

      expect(widget.icon).toBe('☀️'); // 変更なし
      expect(widget.temp).toBe(30); // 更新
      expect(widget.description).toBe('晴れ'); // 変更なし
      expect(widget.location).toBe('東京'); // 変更なし
    });
  });

  describe('WEATHER_ICONS', () => {
    it('定義済みアイコンマッピングが存在する', () => {
      expect(WeatherWidget.WEATHER_ICONS.clear).toBe('☀️');
      expect(WeatherWidget.WEATHER_ICONS.sunny).toBe('☀️');
      expect(WeatherWidget.WEATHER_ICONS.cloudy).toBe('☁️');
      expect(WeatherWidget.WEATHER_ICONS.partlyCloudy).toBe('⛅');
      expect(WeatherWidget.WEATHER_ICONS.rain).toBe('🌧️');
      expect(WeatherWidget.WEATHER_ICONS.heavyRain).toBe('⛈️');
      expect(WeatherWidget.WEATHER_ICONS.snow).toBe('❄️');
      expect(WeatherWidget.WEATHER_ICONS.thunder).toBe('⚡');
      expect(WeatherWidget.WEATHER_ICONS.fog).toBe('🌫️');
      expect(WeatherWidget.WEATHER_ICONS.wind).toBe('💨');
    });
  });

  describe('getIconForCode()', () => {
    it('既知のコードはマッピングされたアイコンを返す', () => {
      expect(WeatherWidget.getIconForCode('clear')).toBe('☀️');
      expect(WeatherWidget.getIconForCode('rain')).toBe('🌧️');
      expect(WeatherWidget.getIconForCode('snow')).toBe('❄️');
    });

    it('未知のコードはデフォルトアイコン🌡️を返す', () => {
      expect(WeatherWidget.getIconForCode('unknown')).toBe('🌡️');
      expect(WeatherWidget.getIconForCode('')).toBe('🌡️');
      expect(WeatherWidget.getIconForCode('hail')).toBe('🌡️');
    });
  });
});
