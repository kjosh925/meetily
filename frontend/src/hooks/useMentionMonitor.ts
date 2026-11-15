'use client';

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type WatchKind = 'name' | 'project' | 'topic';
export interface WatchItem { id: string; label: string; kind: WatchKind; aliases: string[]; description: string }
export interface MonitorConfig {
  version: number; items: WatchItem[]; cooldown_seconds: number; sound: boolean;
  recap_enabled: boolean; ollama_model: string; ai_provider: 'built_in' | 'ollama';
}
export interface MentionAlert {
  id: number; item_id: string; label: string; kind: WatchKind; excerpt: string;
  summary: string | null; audio_time: number; created_at_ms: number; is_test: boolean;
}
export interface MonitorSnapshot {
  config: MonitorConfig; recording: boolean; enabled: boolean; paused: boolean;
  snoozed_until_ms: number; status: string; ai_status: string; alert: MentionAlert | null;
  transcript_lag_seconds: number;
  recent: { sequence_id: number; text: string; audio_start_time: number; audio_end_time: number }[];
}

export function useMentionMonitor() {
  const [snapshot, setSnapshot] = useState<MonitorSnapshot | null>(null);
  const [error, setError] = useState('');
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    let eventVersion = 0;
    const refresh = async () => {
      const version = eventVersion;
      try {
        const value = await invoke<MonitorSnapshot>('mention_monitor_snapshot');
        if (!disposed && version === eventVersion) { setSnapshot(value); setError(''); }
      } catch (e) { if (!disposed) setError(String(e)); }
    };
    void listen<MonitorSnapshot>('mention-monitor-update', (event) => {
      eventVersion++;
      if (!disposed) setSnapshot(event.payload);
    }).then((stop) => { if (disposed) stop(); else { unlisten = stop; void refresh(); } })
      .catch((e) => { if (!disposed) setError(String(e)); });
    // Refresh snooze expiration and recover state after a missed event/reload.
    const interval = setInterval(() => void refresh(), 2000);
    return () => { disposed = true; clearInterval(interval); unlisten?.(); };
  }, []);
  return { snapshot, error };
}
