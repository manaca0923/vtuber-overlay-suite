/**
 * Tauriエラーからエラーメッセージを抽出
 */
export function extractErrorMessage(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  } else if (typeof err === 'string') {
    return err;
  } else if (err && typeof err === 'object' && 'message' in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
}

/**
 * YouTube API関連のエラーメッセージをユーザーフレンドリーな日本語に変換
 */
export function formatYouTubeApiError(errorMessage: string): string {
  if (errorMessage.includes('API key is invalid') || errorMessage.includes('InvalidApiKey')) {
    return 'APIキーが無効です。正しいAPIキーを入力してください。';
  }
  if (errorMessage.includes('Quota exceeded')) {
    return 'APIクォータが超過しています。明日再度お試しください。';
  }
  if (errorMessage.includes('Rate limit exceeded')) {
    return 'レート制限に達しました。しばらく待ってから再度お試しください。';
  }
  if (errorMessage.includes('HTTP request failed')) {
    return 'ネットワークエラーが発生しました。インターネット接続を確認してください。';
  }
  return errorMessage;
}

/**
 * Tauriエラーを処理してユーザーフレンドリーなメッセージを返す
 */
export function handleTauriError(err: unknown, defaultMessage = 'エラーが発生しました'): string {
  const rawMessage = extractErrorMessage(err);
  if (!rawMessage || rawMessage === 'undefined' || rawMessage === '[object Object]') {
    return defaultMessage;
  }
  return formatYouTubeApiError(rawMessage);
}
