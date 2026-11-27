# Security policy

## Reporting a vulnerability

Please do not publish exploitable details in a public issue. Report suspected vulnerabilities privately through the Decode Analytics project owner or an approved private security contact.

Include affected version/commit and operating system, reproduction steps, expected versus actual behavior, logs with serial numbers/personal data/disk contents redacted, impact and suggested mitigation.

## Security design requirements

- No telemetry or disk-content upload by default.
- No network dependency for core diagnostics and recovery.
- Validate all UI-to-Rust inputs.
- Invoke external tools without shell interpolation.
- Apply least privilege; request elevation only for operations that require it.
- Keep logs free of secrets and minimize personally identifiable device information.
- Pin dependencies and scan them in CI.
- Treat device paths, filesystem labels and tool output as untrusted input.
- Require explicit consent before any destructive operation.
