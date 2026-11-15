'use client';

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Bell, X } from 'lucide-react';
import { useMentionMonitor } from '@/hooks/useMentionMonitor';

export default function MentionAlertPage() {
  const { snapshot, error: loadError } = useMentionMonitor();
  const [error, setError] = useState('');
  const alert = snapshot?.alert;
  async function action(command: string, args?: Record<string, unknown>) {
    try { await invoke(command, args); setError(''); }
    catch (e) { setError(String(e)); }
  }
  return <main className="flex min-h-screen flex-col bg-white p-5 text-gray-900">
    <header className="flex items-start justify-between gap-3 border-b pb-4">
      <div><p className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-blue-700"><Bell size={15} /> Meetily · {alert?.is_test ? 'Test alert' : 'Live alert'}</p>
        <h1 className="mt-2 text-xl font-semibold">{alert?.label || 'Meeting alerts'}</h1></div>
      {alert && <button aria-label="Dismiss alert" className="rounded p-1 hover:bg-gray-100" onClick={() => void action('mention_monitor_dismiss', { alertId: alert.id })}><X size={20} /></button>}
    </header>
    {alert ? <div className="flex-1 space-y-4 py-4">
      <p className="text-sm font-medium text-gray-600">{alert.kind === 'topic' ? 'Related discussion' : 'Mention detected'}</p>
      <blockquote className="rounded-lg border-l-4 border-blue-500 bg-blue-50 p-3 text-sm leading-relaxed">“{alert.excerpt}”</blockquote>
      {alert.summary ? <section><h2 className="mb-1 text-xs font-semibold uppercase text-gray-500">Catch-up · AI generated</h2><p className="text-sm leading-relaxed">{alert.summary}</p></section>
        : !alert.is_test && <p role="status" className="text-xs text-gray-500">{snapshot?.config.recap_enabled ? snapshot.ai_status || 'Preparing a local catch-up…' : 'Catch-up generation is off.'}</p>}
    </div> : <p className="flex-1 py-6 text-sm text-gray-500">{loadError || 'No active alert. You can test delivery in Settings → Alerts.'}</p>}
    {error && <p role="alert" className="mb-3 text-xs text-red-700">{error}</p>}
    <footer className="flex flex-wrap gap-2 border-t pt-4">
      <button className="rounded-md bg-blue-600 px-3 py-2 text-sm font-medium text-white" onClick={() => void action('mention_monitor_view_transcript')}>View live transcript</button>
      <button className="rounded-md border px-3 py-2 text-sm" onClick={() => void action('mention_monitor_snooze')}>Snooze 5 min</button>
      {alert && <button className="rounded-md border px-3 py-2 text-sm" onClick={() => void action('mention_monitor_dismiss', { alertId: alert.id })}>Dismiss</button>}
    </footer>
  </main>;
}
