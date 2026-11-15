# Live mention alerts

The initial implementation adds a watch list, immediate name/project alerts,
local topic detection, and short catch-ups to the existing Tauri desktop app.
It shares the same Rust policy and popup UI on macOS, Windows, and Linux.

## Use it

1. Build/run Meetily using the existing platform build instructions. The
   frontend lockfile requires pnpm 9; use `pnpm@9` for dependency installation.
2. Open **Settings → Alerts**. Add your name, aliases/transcription variants,
   project names, and topic descriptions. Be specific about responsibilities.
3. For topics and catch-ups, select **Meetily Built-in AI** (the default). It
   reuses the local summary engine and your configured built-in summary model,
   or an available downloaded summary model when your summary provider is different.
   Download models in **Settings → Summary**. Alternatively select Ollama and
   enter an installed local model. Explicit names/projects require no AI model.
4. Save the watch list. **Test saved alert settings** shows the popup and, if
   enabled in the saved settings, plays a two-tone chime through the default output.
5. Start a recording using local Whisper or Parakeet. Click **Enable alerts**
   in the meeting bar. Monitoring must be enabled for each meeting.
6. The floating popup quotes the matching text immediately. A short AI catch-up
   arrives separately. **View live transcript** opens recent monitored discussion
   in the main Meetily window. You can dismiss the alert or snooze for five minutes.

Enable alerts soon after starting the recording. If a long utterance was
already buffered before enabling, the transcription worker may initially need
to catch up. The meeting bar reports when received transcripts lag live audio
by more than five seconds. This is a diagnostic, not a latency guarantee.

## Behavior and limits

- Matching normalizes Unicode, case, punctuation, and whitespace and observes
  word boundaries. It supports aliases spanning adjacent transcript segments.
  Configure specific aliases for names that speech recognition often misspells.
- Repeat mentions update an existing visible alert within the cooldown. A
  distinct explicit question containing a configured name can alert again.
- A recent explicit mention takes precedence over a weaker topic match.
- While monitoring, the live VAD emits approximately three-second speech windows
  with 300 ms of overlap, even without silence. The final speech event/flush
  contains only its remaining tail. Imports and retranscription do not enable
  this behavior. Recording audio is unchanged; overlapping ASR windows may
  repeat a few words in the transcript and shorter windows can affect accuracy.
- Topic detection uses a local classifier directly. Embedding-based candidate
  selection from the original plan is deferred until latency/accuracy benchmarks
  justify another model and inference dependency. The single AI worker keeps
  the latest pending context rather than building an unbounded queue. Very brief
  topics can be missed if local inference cannot keep up; explicit matching
  continues independently. The latest pending context also preserves a needed
  catch-up for an existing alert.
- Catch-ups are capped in length, grounded in preceding transcript context, and
  labeled AI generated. They can still be inaccurate. Topic evidence must be an
  exact quote from the latest analyzed transcript segment; unknown topic IDs and
  invented quotes are rejected. The feature does not identify speakers.
- Pause, Stop, disabling monitoring, and settings changes invalidate pending AI
  results. Dismissed popups are not reopened by a recap completion. Closing the
  popup with its system close button hides it so later alerts still work.
- A transcription failure disables monitoring with an explanation. A local AI
  failure leaves explicit alerts active and reports that topics/catch-ups are
  unavailable. Recording and normal transcript persistence keep their existing
  lifecycle.
- The recent context buffer is limited to roughly 60 seconds, 40 segments, and
  16 KB. State is backend-owned, so hiding or reloading the main UI does not
  stop detection. Persistent configuration is saved atomically in Tauri app data
  as `mention-monitor.json`; arming and alert history are not persisted.

## Local inference

This feature sends analysis only to `http://127.0.0.1:11434`, with HTTP proxies
and redirects disabled and bounded requests/responses. It does not use the
application's cloud summary provider settings or fall back to a cloud provider.

A loopback Ollama daemon can itself host cloud aliases. Before sending meeting
text, the adapter checks `/api/show` for local weight metadata and refuses
`remote_host`, `remote_model`, or unverified model metadata. These fields are
defined in [Ollama's official response types](https://github.com/ollama/ollama/blob/main/api/types.go).
The daemon and its local configuration remain part of the trusted machine.
Downloads of models and ordinary app installation are separate setup steps.

No monitoring aliases, excerpts, or catch-ups are added to analytics. Existing
meeting storage and application-wide settings retain their normal behavior.

## Implementation map

| Area | Location |
| --- | --- |
| Pure monitor policy, Unicode matching, recent context, cooldowns | `frontend/src-tauri/src/mention_monitor/core.rs` |
| Local-only classifier and recap adapter | `frontend/src-tauri/src/mention_monitor/local_ai.rs` |
| Session lifecycle, latest-value AI queue, IPC, popup delivery | `frontend/src-tauri/src/mention_monitor/mod.rs` |
| Direct live transcript feed | `frontend/src-tauri/src/audio/transcription/worker.rs` |
| Bounded speech cursor and VAD integration | `frontend/src-tauri/src/audio/live_speech_windows.rs`, `audio/vad.rs`, `audio/pipeline.rs` |
| Settings, status, and catch-up UI | `frontend/src/components/Mention*.tsx` |
| Minimal popup route without main-window providers | `frontend/src/app/mention-alert/page.tsx`, `frontend/src/app/layout.tsx` |

The popup uses Tauri window operations on all platforms and does not request
keyboard focus. A CPAL adapter plays the chime directly, without depending on
notification permission or OS sound names. It respects the default output and
system volume; stream failures appear in monitor status. Built-in catch-ups
share a serialized sidecar transaction with summaries, use at most 384 output
tokens, and never route to cloud providers. Stale results are discarded while
already-started generation drains safely. A busy summary can delay catch-ups.
Fullscreen/Spaces/Wayland behavior requires testing on the target desktop.

## Verification

Run the pure production-module tests without native audio or sidecar setup:

```sh
cargo test --manifest-path frontend/src-tauri/tests/mention-monitor/Cargo.toml
```

Run the native live-VAD test after following the normal desktop prerequisites:

```sh
cargo test -p meetily audio::vad::tests::live_windows_deliver
```

Check desktop integration and the frontend:

```sh
cargo check -p meetily
cd frontend
pnpm exec tsc --noEmit
pnpm build
```

The dedicated GitHub workflow runs the policy/adapter tests on macOS, Windows,
and Linux and builds/type-checks the frontend on Linux. Existing platform build
workflows remain available for full native builds. This development host can
verify native macOS compilation; Windows/Linux native delivery and real meeting
latency/accuracy still need target-device checks. No local AI model is downloaded
automatically by this feature.

Development verification: 11 shared policy/adapter tests and all 8 native VAD
tests pass on macOS, including the new continuous-speech delivery/tail test.
The native Rust integration type-check, frontend TypeScript check, and static
production frontend build pass. The upstream standalone lint command opens an
ESLint setup prompt; no new lint configuration was introduced here. The CI
matrix has been added but has not been run remotely from this local branch.
Ollama was not running on the development host, so real-model accuracy and
speech-to-popup latency have not been measured. A subsequent macOS desktop
[smoke test](mention-alerts-smoke-test.md) verified both the direct popup and
live spoken-alias delivery. It also exposed name transcription errors that
prevent exact alias matches. Other target desktops remain unverified. See the subsequent sound/built-in verification below.

### Sound and built-in catch-ups

The subsequent verification passed 12 native monitor tests plus two explicit
integration tests against the Mac audio output and the installed Gemma 3 1B model.
See [the smoke-test record](mention-alerts-smoke-test.md) for evidence and limits.
Ordinary built-in catch-ups request short plain text; topic analysis still uses
validated structured output. The default provider is Built-in AI, including when
loading settings that predate the provider field. Existing recap/sound toggles
and watch items remain unchanged.
