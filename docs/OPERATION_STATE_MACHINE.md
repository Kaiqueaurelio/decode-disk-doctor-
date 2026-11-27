# Operation state machine

Suggested persisted job states:

- `planned`: request is assembled; no device opened.
- `preflight_passed`: platform and policy checks passed; still no write.
- `awaiting_confirmation`: exact operation summary shown to the user.
- `confirmed`: confirmation matches the immutable plan fingerprint.
- `running`: operation is active.
- `pausing`: cancellation/pause requested and process is being stopped safely.
- `paused`: resumable state recorded where the underlying tool supports it.
- `verifying`: output is being checked.
- `completed`: all required verification passed.
- `failed`: operation ended with a recorded error.
- `cancelled`: user cancelled; partial outputs remain explicitly incomplete.

Only legal transitions should be persisted. A terminal state must not transition back to running. Resume creates a new preflight and revalidates source, destination, parameters, artifact and mapfile state. A partial output must never be labeled verified or complete.
