# macOS live popup smoke test

Tested the running feature bundle on 2026-10-18 using the actual desktop UI.
No alert settings were changed during this test.

## Results

| Check | Observed result |
| --- | --- |
| Settings → Alerts → Test saved alert settings | PASS: native “Meetily · Mention alert” window displayed the test alert. |
| Start recording and click Enable alerts | PASS: meeting bar reported “Monitoring active.” |
| Play an 8.416-second locally synthesized speech sample into the recording | PASS: the live transcription worker recognized “KJ, please review the project.” |
| Spoken saved alias → matching → desktop popup | PASS: actual popup showed the saved watch-item label and excerpt “KJ, please review the project.” |
| Recording finished | Confirmed idle/saved recording state afterward; no test recording left running. |

The desktop test confirmed that the popup contained the saved watch-item label
and the “Mention detected” excerpt.

## Remaining recognition issue

The test used a saved name and spoken initials as aliases. The transcription
misheard the full name in several attempts, while the recognized initials
triggered the popup. A second pair of spoken initials was transcribed as
“Can you”, illustrating that initials can also be misrecognized.

This verifies popup delivery and a recognized-alias path. It does not establish
reliable detection of every pronunciation of the user's name, or explain every
earlier missed alert. The current matcher operates on transcript text, so
speech recognition errors can prevent a name match. Recognized repeat mentions
are also suppressed by the saved 30-second cooldown unless a distinct explicit
question containing the name overrides it.

No code fix or alias changes were made on the basis of this smoke test.
Windows/Linux desktop delivery, local recap generation, and end-to-end latency
remain unverified by this test. The UI assertion verifies presence, not that an
audible sound was heard.

## Sound and built-in catch-up verification (2026-10-18)

The previous delivery adapter requested a named OS notification sound and ignored
notification delivery errors. Popup visibility therefore did not prove audio
playback. The replacement CPAL adapter renders a bounded two-tone chime to the
current default output and reports stream/device errors in monitor status.

- 12 native mention-monitor tests passed, including non-silent waveform/volume
  bounds, duration, legacy settings, response parsing, and lifecycle regressions.
- The explicit `real_output_plays_chime` integration test passed on this Mac:
  the output callback consumed the complete chime without a device error.
  This verifies output delivery, not a microphone measurement of speaker loudness.
- The installed Gemma 3 1B model generated: “The transcript requests a review of
  changes to the payment retry process before Friday.” Input was synthetic test
  text. No Ollama service or cloud provider was used. Both real integration tests
  passed together in 5.61 seconds with cached Metal kernels; initial model/kernel
  startup was slower.
- An initial real-model test failed because the small model emitted incomplete
  structured output. Ordinary built-in catch-ups now request plain text, while
  topic responses retain strict JSON and literal-evidence validation.
- The built-in engine serializes summary/catch-up transactions and bounds catch-up
  output to 384 tokens. Existing summary generation retains its normal output limit.
- Frontend TypeScript and production static compilation passed. Windows and Linux
  use the same CPAL adapter but have not had native audio playback verified here.

The rebuilt macOS app was launched successfully. Settings → Alerts showed
**Meetily Built-in AI** selected and **Play an alert sound** enabled. Clicking
**Test saved alert settings** opened the real popup; it was dismissed afterward.
The user's existing watch list and disabled catch-up checkbox were preserved.
The app remains open at Settings → Alerts. The desktop bundle build completed
successfully with the production frontend export.
