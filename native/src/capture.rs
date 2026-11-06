//! NCAP digital event and frame-interleaved sampled payload codecs.

mod header_json;

use crate::validation::validate_capture_semantics;
use num_bigint::BigUint;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt, sync::OnceLock};

const MAGIC: &[u8; 4] = b"NCAP";
const PREFIX_BYTES: usize = 9;
const DIGITAL_RECORD_BYTES: usize = 11;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const SAMPLED_HEADER: &str = "AnalogCaptureHeader";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureError {
    InvalidCapture,
    CaptureCorrupt,
    InvalidFrame,
}

impl CaptureError {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidCapture => "INVALID_CAPTURE",
            Self::CaptureCorrupt => "CAPTURE_CORRUPT",
            Self::InvalidFrame => "INVALID_FRAME",
        }
    }
}

impl fmt::Display for CaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for CaptureError {}

#[derive(Deserialize)]
struct Layout {
    max_header_bytes: usize,
    digital_v1: DigitalLayout,
    sampled_v2: SampledLayout,
}

#[derive(Deserialize)]
struct DigitalLayout {
    version: u8,
    encoding: String,
    max_events: usize,
    max_channels: usize,
    unknown_value_code: u8,
}

#[derive(Deserialize)]
struct SampledLayout {
    version: u8,
    max_frames: u64,
    max_channels: usize,
    max_rate_term: u64,
    max_lost_frames: u64,
}

fn layout() -> &'static Layout {
    static LAYOUT: OnceLock<Layout> = OnceLock::new();
    LAYOUT.get_or_init(|| {
        serde_json::from_str(include_str!("../../contracts/capture/layout.json"))
            .expect("committed NCAP layout")
    })
}

fn split_payload(payload: &[u8], version: u8) -> Option<(&[u8], &[u8])> {
    let prefix = payload.get(..PREFIX_BYTES)?;
    if &prefix[..4] != MAGIC || prefix[4] != version {
        return None;
    }
    let size = u32::from_le_bytes(prefix[5..9].try_into().ok()?) as usize;
    if !(1..=layout().max_header_bytes).contains(&size) {
        return None;
    }
    payload[PREFIX_BYTES..].split_at_checked(size)
}

fn start_payload(version: u8, header: &[u8], body_len: usize) -> Option<Vec<u8>> {
    if !(1..=layout().max_header_bytes).contains(&header.len()) {
        return None;
    }
    let mut payload = Vec::with_capacity(PREFIX_BYTES + header.len() + body_len);
    payload.extend_from_slice(MAGIC);
    payload.push(version);
    payload.extend_from_slice(&(header.len() as u32).to_le_bytes());
    payload.extend_from_slice(header);
    Some(payload)
}

fn has_fields(value: &Value, fields: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == fields.len() && fields.iter().all(|key| object.contains_key(*key))
    })
}

/// Encode NCAP1 events in caller order with a sorted signal table.
pub fn encode_digital(samples: &Value, duration_ns: &Value) -> Result<Vec<u8>, CaptureError> {
    encode_digital_inner(samples, duration_ns).ok_or(CaptureError::InvalidCapture)
}

fn encode_digital_inner(samples: &Value, duration_ns: &Value) -> Option<Vec<u8>> {
    let config = &layout().digital_v1;
    let samples = samples.as_array()?;
    let duration = duration_ns.as_u64()?;
    if duration > MAX_SAFE_INTEGER || samples.len() > config.max_events {
        return None;
    }

    let mut events = Vec::with_capacity(samples.len());
    let mut signals = BTreeSet::new();
    for sample in samples {
        if !has_fields(sample, &["signal", "time_ns", "value"]) {
            return None;
        }
        let signal = sample["signal"].as_str()?;
        let timestamp = sample["time_ns"].as_u64()?;
        if signal.is_empty() || timestamp > duration {
            return None;
        }
        let value = match (sample["value"].as_u64(), sample["value"].as_str()) {
            (Some(0), _) => 0,
            (Some(1), _) => 1,
            (_, Some("X")) => config.unknown_value_code,
            _ => return None,
        };
        signals.insert(signal);
        events.push((signal, timestamp, value));
    }
    if signals.len() > config.max_channels {
        return None;
    }
    let signals: Vec<_> = signals.into_iter().collect();
    let header = serde_json::to_vec(&json!({
        "encoding": config.encoding,
        "signals": signals,
        "sample_count": events.len(),
        "duration_ns": duration,
    }))
    .ok()?;
    let mut payload = start_payload(config.version, &header, events.len() * DIGITAL_RECORD_BYTES)?;
    for (signal, timestamp, value) in events {
        let channel = u16::try_from(signals.binary_search(&signal).ok()?).ok()?;
        payload.extend_from_slice(&channel.to_le_bytes());
        payload.extend_from_slice(&timestamp.to_le_bytes());
        payload.push(value);
    }
    Some(payload)
}

/// Decode an NCAP1 payload, preserving event order and exact integer timestamps.
///
/// Signal strings must contain Unicode scalar values. A manually authored header
/// containing an escaped unpaired surrogate returns `CAPTURE_CORRUPT`.
pub fn decode_digital(payload: &[u8]) -> Result<Value, CaptureError> {
    decode_digital_inner(payload).ok_or(CaptureError::CaptureCorrupt)
}

fn decode_digital_inner(payload: &[u8]) -> Option<Value> {
    let config = &layout().digital_v1;
    if payload.len()
        > PREFIX_BYTES + layout().max_header_bytes + config.max_events * DIGITAL_RECORD_BYTES
    {
        return None;
    }
    let (text, body) = split_payload(payload, config.version)?;
    let header = header_json::parse(text, false)?;
    if !has_fields(
        &header,
        &["encoding", "signals", "sample_count", "duration_ns"],
    ) || header["encoding"] != config.encoding
    {
        return None;
    }
    let signals = header["signals"].as_array()?;
    let count = usize::try_from(header["sample_count"].as_u64()?).ok()?;
    let duration = header["duration_ns"].as_u64()?;
    if signals.len() > config.max_channels
        || count > config.max_events
        || duration > MAX_SAFE_INTEGER
        || body.len() != count * DIGITAL_RECORD_BYTES
    {
        return None;
    }
    let mut seen = BTreeSet::new();
    for signal in signals {
        let signal = signal.as_str()?;
        if signal.is_empty() || !seen.insert(signal) {
            return None;
        }
    }
    let mut samples = Vec::with_capacity(count);
    for record in body.chunks_exact(DIGITAL_RECORD_BYTES) {
        let channel = u16::from_le_bytes(record[..2].try_into().ok()?) as usize;
        let timestamp = u64::from_le_bytes(record[2..10].try_into().ok()?);
        let signal = signals.get(channel)?;
        if timestamp > duration {
            return None;
        }
        let value = match record[10] {
            value @ (0 | 1) => json!(value),
            value if value == config.unknown_value_code => json!("X"),
            _ => return None,
        };
        samples.push(json!({"signal": signal, "time_ns": timestamp, "value": value}));
    }
    Some(Value::Object(
        [
            ("samples".to_owned(), Value::Array(samples)),
            ("duration_ns".to_owned(), json!(duration)),
        ]
        .into_iter()
        .collect(),
    ))
}

#[derive(Clone, Copy)]
enum Encoding {
    Int16,
    Uint16,
    Float32,
}

impl Encoding {
    fn parse(encoding: &Value) -> Option<Self> {
        match encoding.as_str()? {
            "INT16" => Some(Self::Int16),
            "UINT16" => Some(Self::Uint16),
            "FLOAT32" => Some(Self::Float32),
            _ => None,
        }
    }

    fn record_bytes(self) -> usize {
        match self {
            Self::Int16 | Self::Uint16 => 2,
            Self::Float32 => 4,
        }
    }

    fn push(self, sample: &Value, value: f64, payload: &mut Vec<u8>) -> Option<()> {
        match self {
            Self::Int16 => {
                payload.extend_from_slice(&i16::try_from(sample.as_i64()?).ok()?.to_le_bytes());
            }
            Self::Uint16 => {
                payload.extend_from_slice(&u16::try_from(sample.as_u64()?).ok()?.to_le_bytes());
            }
            Self::Float32 => {
                let rounded = value as f32;
                if !rounded.is_finite() {
                    return None;
                }
                payload.extend_from_slice(&rounded.to_le_bytes());
            }
        }
        Some(())
    }

    fn read(self, record: &[u8]) -> Option<(Value, f64)> {
        match self {
            Self::Int16 => {
                let value = i16::from_le_bytes(record.try_into().ok()?);
                Some((value.into(), value as f64))
            }
            Self::Uint16 => {
                let value = u16::from_le_bytes(record.try_into().ok()?);
                Some((value.into(), value as f64))
            }
            Self::Float32 => {
                let value = f32::from_le_bytes(record.try_into().ok()?) as f64;
                value.is_finite().then(|| (value.into(), value))
            }
        }
    }
}

struct SampledHeader {
    encoding: Encoding,
    frame_count: usize,
    calibration: Vec<(f64, f64)>,
}

impl SampledHeader {
    fn checked(header: &Value) -> Option<Self> {
        if !sampled_header_schema().is_valid(header)
            || validate_capture_semantics(SAMPLED_HEADER, header, "transport").is_err()
        {
            return None;
        }
        let calibration = header["channels"]
            .as_array()?
            .iter()
            .map(|channel| Some((channel["scale"].as_f64()?, channel["offset"].as_f64()?)))
            .collect::<Option<_>>()?;
        Some(Self {
            encoding: Encoding::parse(&header["encoding"])?,
            frame_count: usize::try_from(header["frame_count"].as_u64()?).ok()?,
            calibration,
        })
    }

    fn values(&self) -> usize {
        self.frame_count * self.calibration.len()
    }

    fn calibration_is_finite(&self, value: f64, index: usize) -> bool {
        let (scale, offset) = self.calibration[index % self.calibration.len()];
        (value * scale + offset).is_finite()
    }
}

fn sampled_header_schema() -> &'static jsonschema::Validator {
    static SCHEMA: OnceLock<jsonschema::Validator> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let mut document: Value =
            serde_json::from_str(include_str!("../../contracts/capture/ncap.schema.json"))
                .expect("committed NCAP schema");
        document["$ref"] = json!("#/$defs/AnalogCaptureHeader");
        jsonschema::validator_for(&document).expect("valid NCAP header schema")
    })
}

/// Encode an explicit NCAP2 header and its frame-interleaved raw values.
pub fn encode_sampled(header: &Value, samples: &Value) -> Result<Vec<u8>, CaptureError> {
    encode_sampled_inner(header, samples).ok_or(CaptureError::InvalidCapture)
}

fn encode_sampled_inner(header: &Value, samples: &Value) -> Option<Vec<u8>> {
    // Serialize the bounded header to Python-compatible binary64 tokens before
    // letting the schema engine inspect any numbers.
    let text = header_json::encode_sampled_header(header, layout().max_header_bytes)?;
    let parsed = header_json::parse(&text, true)?;
    let header = SampledHeader::checked(&parsed)?;
    let samples = samples.as_array()?;
    if samples.len() != header.values() {
        return None;
    }
    let mut payload = start_payload(
        layout().sampled_v2.version,
        &text,
        samples.len() * header.encoding.record_bytes(),
    )?;
    for (index, sample) in samples.iter().enumerate() {
        let value = sample.as_f64()?;
        if !header.calibration_is_finite(value, index) {
            return None;
        }
        header.encoding.push(sample, value, &mut payload)?;
    }
    Some(payload)
}

/// Verify an NCAP2 descriptor and payload and decode owned raw sample values.
pub fn decode_sampled(payload: &[u8], descriptor: &Value) -> Result<Value, CaptureError> {
    decode_sampled_inner(payload, descriptor).ok_or(CaptureError::CaptureCorrupt)
}

fn decode_sampled_inner(payload: &[u8], descriptor: &Value) -> Option<Value> {
    let layout = layout();
    let maximum = PREFIX_BYTES
        + layout.max_header_bytes
        + Encoding::Float32.record_bytes()
            * layout.sampled_v2.max_channels
            * layout.sampled_v2.max_frames as usize;
    if payload.len() > maximum
        || !has_fields(
            descriptor,
            &["schema_version", "header", "payload_byte_count", "sha256"],
        )
        || descriptor["schema_version"] != "2.0"
        || descriptor["payload_byte_count"].as_f64()? != payload.len() as f64
    {
        return None;
    }
    let (text, body) = split_payload(payload, layout.sampled_v2.version)?;
    if descriptor["sha256"].as_str()? != format!("{:x}", Sha256::digest(payload)) {
        return None;
    }
    let parsed = header_json::parse(text, true)?;
    if !header_json::python_equal(&parsed, &descriptor["header"]) {
        return None;
    }
    // The payload header passes the schema and semantic rules below. Equality
    // also checks the descriptor header, allowing Python's int/float equality.
    let header = SampledHeader::checked(&parsed)?;
    let record_bytes = header.encoding.record_bytes();
    if body.len() != header.values() * record_bytes {
        return None;
    }
    let mut samples = Vec::with_capacity(header.values());
    for (index, record) in body.chunks_exact(record_bytes).enumerate() {
        let (sample, value) = header.encoding.read(record)?;
        if !header.calibration_is_finite(value, index) {
            return None;
        }
        samples.push(sample);
    }
    Some(Value::Object(
        [
            ("descriptor".to_owned(), descriptor.clone()),
            ("samples".to_owned(), Value::Array(samples)),
        ]
        .into_iter()
        .collect(),
    ))
}

/// Return the exact frame time in nanoseconds as a decimal numerator and rate denominator.
///
/// Descriptors may use integral floats for integer fields. Convert those
/// bounded values to integers to keep the rational exact even when the Python
/// helper would round the intermediate time through binary64 arithmetic.
pub fn sampled_frame_time(capture: &Value, frame: &Value) -> Result<(String, u64), CaptureError> {
    sampled_frame_time_inner(capture, frame).ok_or(CaptureError::InvalidFrame)
}

fn sampled_frame_time_inner(capture: &Value, frame: &Value) -> Option<(String, u64)> {
    let layout = &layout().sampled_v2;
    let frame = frame.as_u64()?;
    let header = &capture["descriptor"]["header"];
    if frame >= bounded_integer(&header["frame_count"], 0, layout.max_frames)? {
        return None;
    }
    let axis = &header["time_axis"];
    let rate = &axis["sample_rate_hz"];
    let numerator = bounded_integer(&rate["numerator"], 1, layout.max_rate_term)?;
    let denominator = bounded_integer(&rate["denominator"], 1, layout.max_rate_term)?;
    let anchor = axis["anchor_ns"].as_str()?;
    if anchor.len() > 39 || !anchor.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let anchor = anchor.parse::<BigUint>().ok()?;
    let mut position = frame;
    for gap in header["quality"]["gaps"].as_array()? {
        if bounded_integer(&gap["at_frame"], 0, layout.max_frames)? <= frame {
            position = position.checked_add(bounded_integer(
                &gap["missing_frames"],
                1,
                layout.max_lost_frames,
            )?)?;
        }
    }
    let time = anchor * numerator + BigUint::from(position) * 1_000_000_000_u64 * denominator;
    Some((time.to_string(), numerator))
}

fn bounded_integer(value: &Value, minimum: u64, maximum: u64) -> Option<u64> {
    let number = value.as_f64()?;
    (number.fract() == 0.0 && number >= minimum as f64 && number <= maximum as f64)
        .then_some(number as u64)
}
