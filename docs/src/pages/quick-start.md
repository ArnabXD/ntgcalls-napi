---
title: Quick Start
order: 2
group: Guide
eyebrow: Walkthrough
description: Join a Telegram group call and stream audio or video, end to end.
---

# Quick Start

This page takes you from nothing to audio playing in a group call, then adds video. Make sure you've
read the [prerequisites](/overview/#what-you-need-first) — you need an MTProto library logged in as a
**user account** and **ffmpeg** installed.

The call always follows the same four steps:

1. **Create** a call context and get a WebRTC offer.
2. **Join** the group call through your MTProto lib, passing that offer, and get Telegram's answer.
3. **Connect** with the answer to finish the handshake.
4. **Stream** media into the call.

## Audio

### 1. Create the context

```js
import { NtgCalls } from '@arnabxd/ntgcalls-napi';

const ntg = new NtgCalls();

// Register listeners before the call starts.
ntg.on('connection-change', (chatId, kind, state) => {
  console.log(`call ${chatId}: kind=${kind} state=${state}`);
});

const offer = await ntg.create(chatId); // WebRTC offer SDP
```

### 2. Join the call with your MTProto lib

You hand the offer to Telegram and get an answer back. Do this with whatever MTProto client you use:

{% tabs "mtproto" %}
{% tab "MTKruto" %}
```js
const answer = await client.joinVideoChat(videoChatId, offer, {
  isAudioEnabled: true,
  isVideoEnabled: false,
});
```
{% endtab %}
{% tab "GramJS" %}
```js
// phone.joinGroupCall — offer goes in `params`, result holds the answer
const result = await client.invoke(
  new Api.phone.JoinGroupCall({
    call: inputGroupCall,
    params: new Api.DataJSON({ data: offer }),
    muted: false,
    joinAs: 'me',
  }),
);
const answer = /* the DataJSON answer from `result` */;
```
{% endtab %}
{% endtabs %}

### 3. Connect

```js
await ntg.connect(chatId, answer, false); // `false` = not a presentation/screen share
```

### 4. Play audio

`set_audio_source` is the shortcut for the mic-only case. It runs your ffmpeg command in **SHELL
mode** and reads raw `s16le` 48 kHz mono audio from its stdout:

```js
await ntg.set_audio_source(
  chatId,
  `ffmpeg -i "${url}" -vn -f s16le -ar 48000 -ac 1 -`,
);
```

That's a full audio call. To stop:

```js
await ntg.stop(chatId);
```

## Video

Video isn't a separate method — `set_audio_source` is just a shortcut. For video you use
`set_stream_sources` with both a `microphone` and a `camera` description.

> [!IMPORTANT]
> Enable video **at join time**. If you join audio-only you can't hot-swap video in later — you have
> to stop and re-join with video enabled. So set `isVideoEnabled: true` in step 2.

Step 2, now with video enabled:

{% tabs "mtproto" %}
{% tab "MTKruto" %}
```js
const answer = await client.joinVideoChat(videoChatId, offer, {
  isAudioEnabled: true,
  isVideoEnabled: true,
});
```
{% endtab %}
{% tab "GramJS" %}
```js
// JoinGroupCall with video params enabled, offer in `params`
const result = await client.invoke(
  new Api.phone.JoinGroupCall({
    call: inputGroupCall,
    params: new Api.DataJSON({ data: offer }),
    muted: false,
    videoStopped: false,
    joinAs: 'me',
  }),
);
const answer = /* the DataJSON answer from `result` */;
```
{% endtab %}
{% endtabs %}

Then, instead of `set_audio_source`, push two tracks — one ffmpeg command each:

```js
await ntg.set_stream_sources(chatId, 0 /* capture */, {
  microphone: {
    mediaSource: 2, // SHELL
    input: `ffmpeg -i "${url}" -vn -f s16le -ar 48000 -ac 1 -`,
    sampleRate: 48000,
    channelCount: 1,
    keepOpen: false,
  },
  camera: {
    mediaSource: 2, // SHELL
    input: `ffmpeg -i "${url}" -an -f rawvideo -pix_fmt yuv420p -vf "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2,fps=30" -`,
    width: 1280,
    height: 720,
    fps: 30,
    keepOpen: false,
  },
});
```

> [!WARNING]
> The raw frames ffmpeg produces must match the `width` × `height` you declare **exactly**. That's
> what the `scale=…,pad=…` filter is for: it fits any source into a fixed 1280×720 canvas. Get this
> wrong and you'll see garbled or rejected video.

`mediaSource: 2` is SHELL mode — ntgcalls spawns the command and reads the track from its stdout.
Audio is `s16le` 48 kHz; video is `rawvideo` `yuv420p`. The `streamMode` argument is `0` for capture
(what you send) and `1` for playback.

## Knowing when a track ends

`set_audio_source` / `set_stream_sources` return as soon as streaming starts — they don't wait for
the media to finish. Listen for `stream-end` to queue the next track (e.g. a music bot playlist):

```js
ntg.on('stream-end', (chatId, streamType, streamDevice) => {
  if (streamType === 0) {
    // 0 = audio track finished — play the next one
    playNext(chatId);
  }
});
```

## A working reference

For a complete, real-world bot built on this exact flow, see
[**TGVCBot**](https://github.com/ArnabXD/TGVCBot) (branch `feature/bun-rewrite`) — look at
`src/tgcalls.ts` and `src/ntgcalls/client.ts`.

Next: the full [API Reference](/api-reference/), or [Best Practices](/best-practices/) for the
gotchas worth knowing up front.
