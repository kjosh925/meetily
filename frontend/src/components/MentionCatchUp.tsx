'use client';

import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useMentionMonitor } from '@/hooks/useMentionMonitor';
import { Dialog, DialogContent, DialogTitle, DialogDescription } from '@/components/ui/dialog';

export function MentionCatchUp() {
  const { snapshot } = useMentionMonitor();
  const [open, setOpen] = useState(false);
  useEffect(() => {
    const unlisten = listen('mention-monitor-open-transcript', () => setOpen(true));
    return () => { void unlisten.then((stop) => stop()); };
  }, []);
  return <Dialog open={open} onOpenChange={setOpen}>
    <DialogContent className="max-h-[80vh] max-w-2xl overflow-auto bg-white">
      <DialogTitle>Recent live transcript</DialogTitle>
      <DialogDescription>{snapshot?.status} · Recent 60 seconds of monitored discussion</DialogDescription>
      {snapshot?.alert?.summary && <p className="mb-4 rounded-lg bg-blue-50 p-3 text-sm">{snapshot.alert.summary}</p>}
      <div className="space-y-3 text-sm leading-relaxed">{snapshot?.recent.map((segment) => <p key={segment.sequence_id}><span className="mr-2 text-xs text-gray-400">{Math.floor(segment.audio_start_time / 60)}:{String(Math.floor(segment.audio_start_time % 60)).padStart(2, '0')}</span>{segment.text}</p>)}</div>
      {!snapshot?.recent.length && <p className="text-sm text-gray-500">No monitored transcript yet. Start recording and enable alerts to collect recent context.</p>}
    </DialogContent>
  </Dialog>;
}
