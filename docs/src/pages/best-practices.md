---
title: Best Practices
order: 5
group: Reference
eyebrow: Guidance
description: The gotchas worth knowing before they bite you.
---

# Best Practices

## Enable video at join time

You can't add video to a call you joined audio-only. If you might stream video, set
`isVideoEnabled: true` when your MTProto lib joins the call. To switch an existing audio-only call to
video, you have to `stop()` and re-join.

## Match the frame size exactly

Raw video frames must be exactly the `width` × `height` you declare in the `camera` description. Any
mismatch produces garbled or rejected video. Always normalize the source to a fixed canvas in your
ffmpeg filter:

```text
-vf "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2,fps=30"
```

## Mute before swapping tracks

When changing what's playing, `mute()` (or `pause()`) **before** the next `set_stream_sources` /
`set_audio_source` call. Swapping a live track directly can overlap audio buffers and cause clicks in
the voice chat.

## Use `stream-end` to drive a playlist

`set_audio_source` and `set_stream_sources` resolve when streaming *starts*, not when the media ends.
Listen for `stream-end` to know a track finished and play the next:

```js
ntg.on('stream-end', (chatId, streamType) => {
  if (streamType === 0) playNext(chatId); // 0 = audio
});
```

## Keep event handlers light

Events are delivered to your JS callbacks from native threads via the event loop. Do the minimum in
the handler — if you need heavy I/O or networking in response, hand it off with `setImmediate`,
`queueMicrotask`, or a worker so you don't stall the loop while more events arrive.

## IDs: `number` or `bigint`, your choice

Pass `chatId` / `userId` / `fingerprint` as either. The wrapper coerces a `bigint` to `Number` at
the boundary (napi-rs v3 expects a JS `Number` for `i64`), which is safe because Telegram IDs stay
within `Number.MAX_SAFE_INTEGER` (2⁵³ − 1). Values the library returns (like `time()`) come back as
`bigint`.

```js
await ntg.create(-1001234567890n);            // bigint
await ntg.create(-1001234567890);             // number
await ntg.create(BigInt('-1001234567890'));   // bigint from string

const elapsed = await ntg.time(chatId);       // -> bigint
```
