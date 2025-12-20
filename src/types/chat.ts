/**
 * YouTube Live Chat メッセージ種別
 */
export type MessageType =
  | { type: 'text' }
  | { type: 'superChat'; amount: string; currency: string }
  | { type: 'superSticker'; stickerId: string }
  | { type: 'membership'; level: string }
  | { type: 'membershipGift'; count: number };

/**
 * YouTube Live Chat メッセージ
 */
export interface ChatMessage {
  id: string;
  message: string;
  authorName: string;
  authorChannelId: string;
  authorImageUrl: string;
  publishedAt: string;
  isOwner: boolean;
  isModerator: boolean;
  isMember: boolean;
  isVerified: boolean;
  messageType: MessageType;
}
