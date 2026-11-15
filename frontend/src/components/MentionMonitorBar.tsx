'use client';

import { useState } from 'react';
import Link from 'next/link';
import { invoke } from '@tauri-apps/api/core';
import { Bell, BellOff } from 'lucide-react';
import { useMentionMonitor } from '@/hooks/useMentionMonitor';

export function MentionMonitorBar() {
  const { snapshot } = useMentionMonitor();
  const [error, setError] = useState('');
  if (!snapshot?.recording) return null;
  async function toggle() {
    setError('');
    try { await invoke('mention_monitor_set_enabled', { enabled: !snapshot?.enabled }); }
    catch (e) { setError(String(e)); }
  }
  return <aside className="border-b border-gray-200 bg-blue-50 px-4 py-2 text-sm" aria-label="Meeting mention alerts">
    <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
      {snapshot.enabled ? <Bell size={16} className="text-blue-600" /> : <BellOff size={16} className="text-gray-500" />}
      <span role="status">{snapshot.status}</span>
      <button type="button" onClick={() => void toggle()} className="font-medium text-blue-700 underline">{snapshot.enabled ? 'Turn alerts off' : 'Enable alerts'}</button>
      <Link href="/settings" className="text-gray-600 underline">Watch list in Settings → Alerts</Link>
    </div>
    {snapshot.enabled && snapshot.ai_status && <p className="mt-1 text-xs text-gray-600">{snapshot.ai_status}</p>}
    {error && <p role="alert" className="mt-1 text-red-700">{error}</p>}
  </aside>;
}
