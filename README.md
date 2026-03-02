# Vauxl Matrix Client (MVP)

This repository is the new Matrix-first client for Vauxl.

## MVP Goals
- Authenticate against any compliant Matrix homeserver.
- Sync rooms and timeline via standard Matrix APIs.
- 1:1 and room messaging with media upload/download.
- Basic account/session settings.
- End-to-end encryption support using standard Matrix flows.

## Non-Goals (MVP)
- Non-standard protocol dependencies.
- Vendor lock-in features without open specification.

## Structure
- `src/` implementation
- `tests/` tests
- `docs/` architecture and decisions

## Standards Position
Vauxl extensions must be optional and capability-discoverable so other Matrix clients can interoperate.
