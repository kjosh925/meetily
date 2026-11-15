'use client';

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Bell, Plus, Trash2 } from 'lucide-react';
import { MonitorConfig, WatchKind, useMentionMonitor } from '@/hooks/useMentionMonitor';

export function MentionMonitorSettings() {
  const { snapshot, error: loadError } = useMentionMonitor();
  const [config, setConfig] = useState<MonitorConfig | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  useEffect(() => { if (snapshot && config === null) setConfig(snapshot.config); }, [snapshot, config]);

  async function save() {
    if (!config) return;
    setBusy(true); setMessage(''); setError('');
    try {
      const saved = { ...config, items: config.items.map((item) => ({ ...item,
        label: item.label.trim(), aliases: item.aliases.map((alias) => alias.trim()).filter(Boolean),
      })) };
      await invoke('mention_monitor_save_config', { config: saved }); setConfig(saved);
      setMessage('Saved. Settings apply to this meeting too.');
    }
    catch (e) { setError(String(e)); }
    finally { setBusy(false); }
  }
  const inputClass = 'w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500';
  if (!config) return <p role="status" className="p-6 text-sm">{loadError || 'Loading alert settings…'}</p>;
  const update = (index: number, patch: Partial<MonitorConfig['items'][number]>) => setConfig({
    ...config, items: config.items.map((item, i) => i === index ? { ...item, ...patch } : item),
  });

  return <section className="max-w-3xl space-y-6 py-6">
    <div>
      <h2 className="flex items-center gap-2 text-xl font-semibold"><Bell size={20} /> Notify me when I’m relevant</h2>
      <p className="mt-2 text-sm text-gray-600">Get a popup when your name, project, or work comes up. Detection runs on your computer on macOS, Windows, and Linux.</p>
      <p className="mt-2 text-sm text-gray-600">Save your watch list, start recording with local Whisper or Parakeet, then enable alerts in the meeting bar. Enable them separately for each meeting.</p>
    </div>
    <div className="space-y-4">
      {config.items.map((item, index) => <fieldset key={item.id} className="space-y-3 rounded-xl border border-gray-200 bg-white p-4">
        <legend className="px-1 text-sm font-medium">Watch item {index + 1}</legend>
        <div className="flex items-center gap-3">
          <label className="flex-1 text-sm">Type
            <select className={`${inputClass} mt-1`} value={item.kind} onChange={(e) => update(index, { kind: e.target.value as WatchKind })}>
              <option value="name">My name</option><option value="project">Project</option><option value="topic">Topic / responsibility</option>
            </select>
          </label>
          <button type="button" className="mt-5 rounded p-2 text-gray-500 hover:bg-red-50 hover:text-red-600" aria-label={`Remove watch item ${index + 1}`}
            onClick={() => setConfig({ ...config, items: config.items.filter((_, i) => i !== index) })}><Trash2 size={18} /></button>
        </div>
        <label className="block text-sm">{item.kind === 'name' ? 'Your name' : 'Label'}
          <input className={`${inputClass} mt-1`} maxLength={160} value={item.label} placeholder={item.kind === 'name' ? 'Alex Morgan' : 'Payment reliability'} onChange={(e) => update(index, { label: e.target.value })} />
        </label>
        {item.kind === 'topic' ? <label className="block text-sm">What work should trigger an alert?
          <textarea className={`${inputClass} mt-1`} maxLength={1000} rows={3} value={item.description} placeholder="I own retrying failed subscription renewals and resolving duplicate charges." onChange={(e) => update(index, { description: e.target.value })} />
        </label> : <label className="block text-sm">Aliases and transcription variants, separated by commas
          <input className={`${inputClass} mt-1`} value={item.aliases.join(',')} placeholder="Alex, AM" onChange={(e) => update(index, { aliases: e.target.value.split(',') })} />
          <span className="mt-1 block text-xs text-gray-500">The label itself is also matched. Use specific phrases to avoid unrelated alerts.</span>
        </label>}
      </fieldset>)}
      <button type="button" disabled={config.items.length >= 50} className="flex items-center gap-2 rounded-md border px-3 py-2 text-sm hover:bg-gray-50 disabled:opacity-50"
        onClick={() => setConfig({ ...config, items: [...config.items, { id: crypto.randomUUID(), label: '', kind: 'name', aliases: [], description: '' }] })}><Plus size={16} /> Add watch item</button>
    </div>
    <div className="space-y-4 rounded-xl border border-gray-200 p-4">
      <h3 className="font-medium">Local topics and catch-ups</h3>
      <label className="block text-sm">Local AI engine
        <select className={`${inputClass} mt-1`} value={config.ai_provider} onChange={(e) => setConfig({ ...config, ai_provider: e.target.value as MonitorConfig['ai_provider'] })}>
          <option value="built_in">Meetily Built-in AI</option><option value="ollama">Ollama</option>
        </select>
      </label>
      {config.ai_provider === 'ollama' && <label className="block text-sm">Installed Ollama model
        <input className={`${inputClass} mt-1`} maxLength={160} value={config.ollama_model} placeholder="For example: qwen2.5:3b" onChange={(e) => setConfig({ ...config, ollama_model: e.target.value.trim() })} />
      </label>}
      <p className="text-xs text-gray-500">Built-in AI reuses Meetily’s downloaded summary models. Install one in Settings → Summary. Ollama is optional and connects only to this computer. There is no cloud fallback. Names and project aliases work without an AI model.</p>
      <label className="flex items-center gap-2 text-sm"><input type="checkbox" checked={config.recap_enabled} onChange={(e) => setConfig({ ...config, recap_enabled: e.target.checked })} /> Add a brief catch-up after alerting</label>
      <label className="flex items-center gap-2 text-sm"><input type="checkbox" checked={config.sound} onChange={(e) => setConfig({ ...config, sound: e.target.checked })} /> Play an alert sound</label>
      <label className="block text-sm">Repeat alert cooldown (seconds)
        <input type="number" min={5} max={300} className={`${inputClass} mt-1 max-w-32`} value={config.cooldown_seconds} onChange={(e) => setConfig({ ...config, cooldown_seconds: Number(e.target.value) })} />
      </label>
    </div>
    {(error || loadError) && <p role="alert" className="text-sm text-red-700">{error || loadError}</p>}
    {message && <p role="status" className="text-sm text-green-700">{message}</p>}
    <div className="flex gap-3">
      <button type="button" disabled={busy} onClick={() => void save()} className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white disabled:opacity-50">{busy ? 'Saving…' : 'Save watch list'}</button>
      <button type="button" onClick={() => { void invoke('mention_monitor_test').catch((e) => setError(String(e))); }} className="rounded-md border px-4 py-2 text-sm">Test saved alert settings</button>
    </div>
    <p className="text-xs text-gray-500">Try the test while another app is open. The chime plays through your default audio output; check system volume and connected headphones. Actual alert speed also depends on transcription speed.</p>
  </section>;
}
