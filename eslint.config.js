import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'

export default [
  { ignores: ['dist', 'src-tauri/target'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }],
    },
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
  },
  {
    files: ['src-tauri/overlays/**/*.js'],
    languageOptions: {
      ecmaVersion: 2020,
      sourceType: 'script',
      globals: {
        ...globals.browser,
        // オーバーレイ共通グローバル
        BaseComponent: 'readonly',
        ComponentRegistry: 'readonly',
        SlotManager: 'readonly',
        CLAMP_RANGES: 'readonly',
        UpdateBatcher: 'readonly',
        DensityManager: 'readonly',
      },
    },
    rules: {
      'no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-unused-vars': 'off', // JSファイルにはTypeScriptルールを適用しない
    },
  },
  {
    // グローバルを定義するファイルはno-redeclareを無効化
    files: [
      'src-tauri/overlays/components/base-component.js',
      'src-tauri/overlays/shared/component-registry.js',
      'src-tauri/overlays/shared/slots.js',
      'src-tauri/overlays/shared/clamp-constants.js',
      'src-tauri/overlays/shared/update-batcher.js',
      'src-tauri/overlays/shared/density-manager.js',
    ],
    rules: {
      'no-redeclare': 'off',
    },
  },
]
