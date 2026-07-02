import type { Channel } from "@/stores/channels";

/**
 * Format a channel number for display (e.g., "5.1" or "12")
 */
export function formatChannelNumber(channel: Channel): string {
  return channel.minor > 0 ? `${channel.major}.${channel.minor}` : `${channel.major}`;
}

/**
 * Format a relative date (Today, Yesterday, X days ago, or short date)
 */
export function formatRelativeDate(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return `${days} days ago`;
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}
