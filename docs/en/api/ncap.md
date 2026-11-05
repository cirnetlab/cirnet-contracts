# NCAP capture formats

English · [Tiếng Việt](../../vi/api/ncap.md)

[Documentation index](../index.md)

The checked [binary layout](../../../contracts/capture/layout.json) is the layout authority. The contract generator emits browser constants and `contracts.boundary.capture_layout`. Python can also read the packaged resource directly during generation. The [NCAP schema](../../../contracts/capture/ncap.schema.json) defines closed sampled descriptors at `https://cirnet.local/contracts/capture/ncap/2.0/schema.json`. Historical CaptureMetadata 1.0 remains readable.

Both versions start with a 9-byte prefix. It contains ASCII `NCAP`, one unsigned version byte and the little-endian unsigned 32-bit JSON-header byte length. The UTF-8 JSON header has 1–65,536 bytes and is followed by the exact binary body. Reject invalid UTF-8, a BOM, duplicate members even when their values match, additional fields and trailing body bytes. Header integer fields require decimal integer tokens. Fractional and exponent spellings are invalid for integer counts. Version 2 calibration numbers must retain their written digits after decimal-to-binary64-to-decimal conversion. Integer tokens, including calibration integer tokens, must be within JavaScript’s safe integer range. Scientific notation can express representable large floating-point calibration values.

## Version 1 digital events

The header contains exactly `encoding: "digital_events"`, the ordered `signals` table, `sample_count` and `duration_ns`. Each 11-byte little-endian `<HQB` record contains an unsigned 16-bit channel index, unsigned 64-bit nanoseconds and a value byte. Values `0` and `1` represent digital levels, and `2` represents unknown `X`.

Decoding preserves event order, including equal or nonmonotonic timestamps. Values and timestamps retain the original samples. Timestamps are bounded by both the declared duration and 9,007,199,254,740,991. The channel table allows at most 4,096 unique nonempty names and a payload allows at most 100,000 events. The encoder sorts the channel table and preserves the committed legacy golden bytes.

The runtime endpoint, MIME version and CaptureStore produce and validate v1. The SHA-256 covers the entire prefix, JSON header and body and is stored externally in CaptureMetadata. There is no checksum field in the v1 binary header.

## Version 2 sampled frames

`AnalogCaptureDescriptor` 2.0 includes the complete expected header, exact `payload_byte_count` and lowercase `sha256` of the complete binary payload. The `AnalogCaptureHeader` values must agree with the descriptor regardless of JSON member order. Capture identity and generation participate in both hash and agreement checks. The digest stays outside the binary header to avoid a circular hash.

One encoding applies to the whole capture. `INT16` and `UINT16` use two bytes per value. IEEE-754 `FLOAT32` uses four. All are little-endian. The body interleaves channels in their declared order within each frame. For two channels the body begins frame 0/channel 0, frame 0/channel 1, then frame 1/channel 0. `frame_count` counts stored frames. Require exactly `frame_count * channels.length * bytes_per_value` body bytes. The parser limits are 100,000 frames and eight channels. Nonfinite FLOAT32 values or calibrated results reject the artifact, even in invalid-quality ranges.

Every channel declares an ID, unit, nonzero scale, offset, increasing physical range and nullable calibration ID. The conversion is `raw * scale + offset`. A null calibration ID makes no traceability claim. Values outside the declared range are retained and marked as clipping. The clipped-channel list must refer to known channels.

The time axis declares `capture_relative`, `unix_epoch` or `device_clock`. Capture-relative coordinates require a zero anchor. Device-clock coordinates require an opaque clock ID and cannot use a UTC label. The other references require a null clock ID. The axis uses a decimal-string nanosecond anchor and a positive rational sample rate `numerator / denominator` Hz. The exact frame time remains an integer rational.

Gap positions are strictly increasing. A gap at stored index `i` represents missing frames before that frame. Index zero represents leading loss. Index `frame_count` represents trailing loss, which remains quality evidence without a stored sample. Frame `i` has exact time `anchor + (i + sum(gaps at or before i)) * 1e9 * rate.denominator / rate.numerator`.

Quality records the status, termination reason, exact lost-frame count, gaps, sorted disjoint invalid stored ranges and clipped channel IDs. The gaps account for every lost frame. The format permits up to 256 gaps, 256 invalid ranges and 1,000,000 lost frames. Positive sample-rate numerator and denominator each have a maximum of 1,000,000,000,000.

| Status | Meaning |
| --- | --- |
| `VALID` | Complete termination with no loss or invalid ranges. |
| `PARTIAL` | Incomplete termination or invalid ranges, with no transport loss. |
| `LOSS` | Recorded frame loss. |
| `INVALID` | Preserve the evidence and withhold trusted amplitude display. |

The termination reasons are `COMPLETE`, `STOPPED`, `DURATION_LIMIT`, `BUFFER_LIMIT`, `TRANSPORT_LOSS`, `DRIVER_ERROR` and `UNKNOWN`. They do not establish successful hardware application.

## Decoding, transport and display

Python `decode_sampled` verifies the descriptor shape, shared relationships, SHA, header agreement and exact body layout before returning owned raw values. Browser `decodeAnalogCapture` performs structural decoding and reports `integrity: "unchecked"`. `verifyAndDecodeAnalogCapture` verifies SHA before reporting `integrity: "verified"`. A missing integrity primitive fails explicitly. Cancellation after a late body or hash completion prevents publication.

`loadAnalogCapture` requires an explicit `AnalogPayloadSource`. The source receives capture identity, cancellation and an exact byte bound. It must enforce owner access and use an appropriate versioned transport. Runtime still exposes v1, default Scope has zero channels, and there is no default v2 network source. Connecting sampled Scope data requires an explicit station profile and versioned endpoint. This format has no sampled hardware producer.

`createDisplayEnvelope` builds at most 4,096 min/max buckets for a selected frame range and channel. It retains valid extrema and preserves the raw typed arrays. The display envelope carries source, synthetic or measured provenance, integrity, quality, units, discontinuity, invalidity and clipping. Invalid ranges are omitted from amplitudes. Wholly `INVALID` captures have null amplitudes. Exact rational times remain available alongside approximate relative plotting offsets. The helper does not smooth, interpolate, reconnect across gaps, grade results or replace raw evidence.

## Shared fixtures

The [conformance fixtures](../../../contracts/fixtures/capture/conformance.json) contain independently packed synthetic golden bytes for Python and browser tests. They cover legacy byte preservation, UTF-8 names, exact timestamps and unknown levels, all three sampled encodings, calibration including negative scale, rate and anchor precision, empty, partial, invalid and lost captures, clipping, malformed headers, body sizes and checksums. Annotated semantic cases compare issue code, path, severity and profile. These tests establish format interoperability. See the [original format reference](../../../contracts/capture/README.md).
