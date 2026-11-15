# Run this Meetily feature build

This checkout is on `enhance/live-mention-alerts`. Its standalone macOS
development bundle is built into the project, with the frontend and sidecars
included. It does not need a Next.js server to run.

## Open the standalone app on this Mac

```sh
open "./target/debug/bundle/macos/meetily.app"
```

Quit any other Meetily instance before switching builds. The application uses
a single-instance guard and the existing `com.meetily.ai` app-data location.
The project bundle is separate from `/Applications/meetily.app`.

## Try live mention alerts

1. Complete Meetily's first-run setup if it appears. Choose/download a local
   transcription model and grant the requested audio permissions when ready.
2. Open **Settings → Alerts**. Add your name, aliases, and project names, then
   save the watch list.
3. Click **Test saved alert settings** to check the popup and chime.
4. Start a recording using local Whisper or Parakeet, then click **Enable alerts**
   in the meeting bar. Enable monitoring separately for each meeting.
5. For catch-ups, select **Meetily Built-in AI** in **Settings → Alerts** and
   enable **Add a brief catch-up after alerting**. It reuses local summary models
   downloaded in **Settings → Summary**. Ollama remains an optional alternative.
   No model is downloaded automatically by the alert feature. Explicit name and
   project matches work without a summary model.

See [the feature guide](mention-alerts.md) for behavior, limitations, and tests.

## Run with hot reload

Quit the standalone app first, then run:

```sh
cd frontend
npx --yes pnpm@9.15.9 exec tauri dev
```

This starts the frontend on port 3118 and launches the native application.
Keep the terminal running; press Ctrl+C to stop development mode.

Dependencies and the native sidecar have already been prepared in this
checkout. If reinstalling frontend dependencies, use the repository's pnpm 9
version:

```sh
npx --yes pnpm@9.15.9 install --frozen-lockfile
```

The globally installed pnpm 11 can attempt to replace the pnpm 9 dependency
installation. Use the version-pinned commands above for reproducible results.

## Rebuild the standalone macOS development app

```sh
cd frontend
npx --yes pnpm@9.15.9 exec tauri build --debug --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'
```

This is a development build, not a release-performance benchmark. It uses the
project's local ad-hoc signing settings and does not produce updater artifacts.
For an optimized release or a fresh Windows/Linux environment, follow
[the platform build instructions](BUILDING.md) and the existing GPU build scripts,
which also prepare the appropriate native sidecar. Build caches and model
downloads require additional free disk space.
