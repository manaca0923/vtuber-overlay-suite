use crate::youtube::{client::YouTubeClient, types::ChatMessage};

#[tauri::command]
pub async fn validate_api_key(api_key: String) -> Result<bool, String> {
    let client = YouTubeClient::new(api_key);
    client.validate_api_key().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_live_chat_id(api_key: String, video_id: String) -> Result<String, String> {
    let client = YouTubeClient::new(api_key);
    client
        .get_live_chat_id(&video_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_chat_messages(
    api_key: String,
    live_chat_id: String,
    page_token: Option<String>,
) -> Result<(Vec<ChatMessage>, Option<String>, u64), String> {
    let client = YouTubeClient::new(api_key);
    let response = client
        .get_live_chat_messages(&live_chat_id, page_token.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    // レスポンスをChatMessage型に変換
    let messages: Vec<ChatMessage> = response
        .items
        .into_iter()
        .filter_map(|item| {
            use chrono::DateTime;

            // publishedAtのパースに失敗した場合はそのメッセージをスキップ
            let published_at = match DateTime::parse_from_rfc3339(&item.snippet.published_at) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(e) => {
                    log::warn!(
                        "Failed to parse publishedAt for message {}: {}. Skipping message.",
                        item.id,
                        e
                    );
                    return None;
                }
            };

            Some(ChatMessage {
                id: item.id,
                message: item.snippet.display_message,
                author_name: item.author_details.display_name,
                author_channel_id: item.author_details.channel_id,
                author_image_url: item.author_details.profile_image_url,
                published_at,
                is_owner: item.author_details.is_chat_owner,
                is_moderator: item.author_details.is_chat_moderator,
                is_member: item.author_details.is_chat_sponsor,
                is_verified: item.author_details.is_verified,
                message_type: crate::youtube::types::MessageType::Text, // 簡易実装
            })
        })
        .collect();

    Ok((
        messages,
        response.next_page_token,
        response.polling_interval_millis,
    ))
}
