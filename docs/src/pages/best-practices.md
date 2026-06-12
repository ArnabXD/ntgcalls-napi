---
title: Best Practices
order: 5
group: Reference
eyebrow: Guidance
description: Event-loop hygiene, graceful state transitions, and BigInt handling.
---

# Integration Best Practices

## A. JavaScript/TypeScript Event Loop Preservation

Native callbacks invoke JavaScript asynchronously. Keep your callbacks **lightweight and fast**. If
you need to perform intensive disk I/O or heavy networking on event notifications, delegate them to
standard microtasks (using `setImmediate`, `setTimeout`, or secondary worker threads) to prevent
stalling the main event loop.

## B. Muting / Pausing Graceful States

When a track finishes, the `stream-end` callback fires. Clients must capture this state to queue the
next audio track in their music-bot playlist.

> [!TIP]
> Mute or stop playback **before** issuing a secondary `set_stream_sources` or `set_audio_source`
> payload, to prevent audio buffer overlapping or clicking sounds in the voice chat channel.

## C. BigInt Handling

ID and timestamp arguments (`chatId`, `userId`, `fingerprint`, broadcast timestamps, etc.) accept
either a `number` or a `bigint` (`number | bigint`). Internally the JS wrapper coerces any `bigint`
you pass to a standard `Number` before crossing the N-API boundary, because napi-rs v3 requires a JS
`Number` at runtime for `i64` arguments. This is safe: Telegram IDs stay within
`Number.MAX_SAFE_INTEGER` (2^53 − 1), well below the `i64` range used natively.

You may therefore pass `1234567890n`, `BigInt("chat_id_string")`, or a plain `1234567890`
interchangeably. Return values that the native library produces (such as `time()`) come back as
`bigint`.

```typescript
// All three are equivalent at the call boundary:
await ntg.create(1234567890n);
await ntg.create(BigInt("1234567890"));
await ntg.create(1234567890);

// Returned values are bigint:
const elapsed: bigint = await ntg.time(1234567890n);
```