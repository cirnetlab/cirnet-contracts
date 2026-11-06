use cirnet_contracts_native::capture::{
    decode_digital, decode_sampled, encode_digital, encode_sampled, sampled_frame_time,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn cases() -> Vec<Value> {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../contracts/fixtures/capture/conformance.json"
    ))
    .unwrap();
    corpus["cases"].as_array().unwrap().clone()
}

fn payload(case: &Value) -> Vec<u8> {
    case["payload_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn sampled_fixture(id: &str) -> Value {
    cases().into_iter().find(|case| case["id"] == id).unwrap()
}

fn header_bytes(payload: &[u8]) -> &str {
    let length = u32::from_le_bytes(payload[5..9].try_into().unwrap()) as usize;
    std::str::from_utf8(&payload[9..9 + length]).unwrap()
}

fn sampled_payload_with_header(case: &mut Value, header: &str) -> Vec<u8> {
    let original = payload(case);
    let original_header_len = header_bytes(&original).len();
    let mut bytes = original[..5].to_vec();
    bytes.extend((header.len() as u32).to_le_bytes());
    bytes.extend(header.bytes());
    bytes.extend_from_slice(&original[9 + original_header_len..]);
    case["descriptor"]["payload_byte_count"] = json!(bytes.len());
    case["descriptor"]["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    bytes
}

#[test]
fn shared_digital_corpus_preserves_order_timestamps_and_exact_bytes() {
    for case in cases().into_iter().filter(|case| case["version"] == 1) {
        let bytes = payload(&case);
        let actual = decode_digital(&bytes);
        let id = &case["id"];
        if case["valid"] == false {
            assert_eq!(actual.unwrap_err().code(), "CAPTURE_CORRUPT", "{id}");
            continue;
        }
        let actual = actual.unwrap_or_else(|error| panic!("{id}: {error}"));
        assert_eq!(actual, case["expected"], "{id}");
        if case["canonical_encoding"] == true {
            assert_eq!(
                encode_digital(&actual["samples"], &actual["duration_ns"]).unwrap(),
                bytes,
                "{id}",
            );
        }
    }
}

#[test]
fn shared_sampled_corpus_preserves_interleaving_exact_times_and_bytes() {
    for case in cases().into_iter().filter(|case| case["version"] == 2) {
        let bytes = payload(&case);
        let actual = decode_sampled(&bytes, &case["descriptor"]);
        let id = &case["id"];
        if case["valid"] == false {
            assert_eq!(actual.unwrap_err().code(), "CAPTURE_CORRUPT", "{id}");
            continue;
        }
        let actual = actual.unwrap_or_else(|error| panic!("{id}: {error}"));
        let header = &actual["descriptor"]["header"];
        let channels = header["channels"].as_array().unwrap();
        let samples = actual["samples"].as_array().unwrap();
        let expected = case["expected"]["samples"].as_array().unwrap();
        assert_eq!(actual["descriptor"], case["descriptor"], "{id}");
        assert_eq!(samples.len(), expected.len(), "{id}");
        for (index, (sample, expected)) in samples.iter().zip(expected).enumerate() {
            let raw = sample.as_f64().unwrap();
            let channel = &channels[index % channels.len()];
            let physical =
                raw * channel["scale"].as_f64().unwrap() + channel["offset"].as_f64().unwrap();
            assert_eq!(sample.is_f64(), header["encoding"] == "FLOAT32", "{id}");
            assert_eq!(raw, expected.as_f64().unwrap(), "{id}");
            assert_eq!(
                physical,
                case["expected"]["physical_values"][index].as_f64().unwrap(),
                "{id}",
            );
        }
        for (frame, expected) in case["expected"]["times_ns"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let (numerator, denominator) = sampled_frame_time(&actual, &json!(frame)).unwrap();
            assert_eq!(numerator, expected["numerator"].as_str().unwrap(), "{id}");
            assert_eq!(
                denominator.to_string(),
                expected["denominator"].as_str().unwrap(),
                "{id}",
            );
        }
        if case["canonical_encoding"] == true {
            assert_eq!(
                encode_sampled(header, &actual["samples"]).unwrap(),
                bytes,
                "{id}",
            );
        }
    }
}

#[test]
fn exact_time_covers_maximum_anchor_and_rate_terms_with_loss() {
    let mut case = sampled_fixture("v2-loss-gaps-invalid-range-clipped");
    let axis = &mut case["descriptor"]["header"]["time_axis"];
    axis["anchor_ns"] = json!("999999999999999999999999999999999999999");
    axis["sample_rate_hz"] =
        json!({"numerator": 1_000_000_000_000_u64, "denominator": 1_000_000_000_000_u64});
    let capture = json!({"descriptor": case["descriptor"]});
    assert_eq!(
        sampled_frame_time(&capture, &json!(2)).unwrap(),
        (
            "1000000000000000000000000000006999999999000000000000".to_owned(),
            1_000_000_000_000,
        ),
    );
    for invalid in [json!(-1), json!(3), json!(1.0), json!(true)] {
        assert_eq!(
            sampled_frame_time(&capture, &invalid).unwrap_err().code(),
            "INVALID_FRAME",
        );
    }
}

#[test]
fn sampled_encoder_uses_python_float_tokens_and_insertion_order() {
    let mut case = sampled_fixture("v2-empty");
    let header = &mut case["descriptor"]["header"];
    let channel = &mut header["channels"][0];
    channel["scale"] = json!(0.00001);
    channel["offset"] = json!(-0.0);
    channel["range_min"] = json!(-1000000000000000.0);
    channel["range_max"] = json!(10000000000000000.0);
    channel["unit"] = json!("µV");
    let encoded = encode_sampled(header, &json!([])).unwrap();
    assert!(header_bytes(&encoded).contains(
        r#""unit":"µV","scale":1e-05,"offset":-0.0,"range_min":-1000000000000000.0,"range_max":1e+16"#
    ));
    header["channels"][0]["offset"] = json!(f64::from_bits(0x430a8e5672bc7312));
    let encoded = encode_sampled(header, &json!([])).unwrap();
    assert!(header_bytes(&encoded).contains(r#""offset":934356417220194.2"#));
}

#[test]
fn sampled_decoder_compares_descriptor_numbers_and_objects_like_python() {
    let mut case = sampled_fixture("v2-int16-interleaved-negative-calibration");
    let bytes = payload(&case);
    let descriptor = &mut case["descriptor"];
    descriptor["payload_byte_count"] = json!(768.0);
    let header = descriptor["header"].as_object_mut().unwrap();
    let generation = header.shift_remove("generation").unwrap();
    header.insert(
        "generation".into(),
        json!(generation.as_u64().unwrap() as f64),
    );
    descriptor["header"]["channels"][0]["offset"] = json!(0.0);
    descriptor["header"]["time_axis"]["sample_rate_hz"] =
        json!({"numerator": 3.0, "denominator": 2.0});
    let actual = decode_sampled(&bytes, descriptor).unwrap();
    assert_eq!(actual["descriptor"], *descriptor);
    assert_eq!(
        sampled_frame_time(&actual, &json!(0)).unwrap(),
        ("27021597764222979369".to_owned(), 3),
    );
}

#[test]
fn sampled_decoder_rejects_objects_disguised_as_arbitrary_precision_numbers() {
    let mut case = sampled_fixture("v2-empty");
    let original = payload(&case);
    let header = header_bytes(&original).replace(
        r#""scale":0.01"#,
        r#""scale":{"$serde_json::private::Number":"0.01"}"#,
    );
    let bytes = sampled_payload_with_header(&mut case, &header);
    assert_eq!(
        decode_sampled(&bytes, &case["descriptor"])
            .unwrap_err()
            .code(),
        "CAPTURE_CORRUPT",
    );
}

#[test]
fn sampled_encoder_normalizes_extreme_float_exponents_before_validation() {
    let case = sampled_fixture("v2-empty");
    let original = &case["descriptor"]["header"];
    let tiny: Value = serde_json::from_str("1.01e-9223372036854775807").unwrap();

    let mut header = original.clone();
    header["channels"][0]["scale"] = tiny.clone();
    assert_eq!(
        encode_sampled(&header, &json!([])).unwrap_err().code(),
        "INVALID_CAPTURE",
    );

    let mut header = original.clone();
    header["generation"] = tiny;
    assert_eq!(
        encode_sampled(&header, &json!([])).unwrap_err().code(),
        "INVALID_CAPTURE",
    );

    let mut header = original.clone();
    header["channels"][0]["offset"] = serde_json::from_str("-1.01e-9223372036854775807").unwrap();
    let encoded = encode_sampled(&header, &json!([])).unwrap();
    header["channels"][0]["offset"] = json!(-0.0);
    assert_eq!(encoded, encode_sampled(&header, &json!([])).unwrap());
}

#[test]
fn sampled_decoder_checks_extreme_zero_tokens_before_normalizing_for_schema() {
    let original = sampled_fixture("v2-empty");
    let bytes = payload(&original);
    let text = header_bytes(&bytes);

    let mut case = original.clone();
    let header = text.replace(r#""offset":0"#, r#""offset":0.00e-9223372036854775807"#);
    let bytes = sampled_payload_with_header(&mut case, &header);
    assert_eq!(
        decode_sampled(&bytes, &case["descriptor"])
            .unwrap_err()
            .code(),
        "CAPTURE_CORRUPT",
    );

    let mut case = original;
    let header = text.replace(r#""offset":0"#, r#""offset":0.00e-100000000"#);
    let bytes = sampled_payload_with_header(&mut case, &header);
    assert_eq!(
        decode_sampled(&bytes, &case["descriptor"]).unwrap()["samples"],
        json!([]),
    );
}

#[test]
fn encoders_reject_wrong_raw_value_types_and_bounds() {
    for invalid in [json!(true), json!(1.0), json!(2), json!("x")] {
        let samples = json!([{"signal": "D", "time_ns": 0, "value": invalid}]);
        assert_eq!(
            encode_digital(&samples, &json!(0)).unwrap_err().code(),
            "INVALID_CAPTURE",
        );
    }
    for (id, invalid) in [
        ("v2-int16-interleaved-negative-calibration", json!(32768)),
        ("v2-int16-interleaved-negative-calibration", json!(-1.0)),
        ("v2-uint16-extrema", json!(-1)),
        ("v2-float32-extrema", json!(1e39)),
        ("v2-float32-extrema", json!(true)),
    ] {
        let mut case = sampled_fixture(id);
        case["expected"]["samples"][0] = invalid;
        assert_eq!(
            encode_sampled(&case["descriptor"]["header"], &case["expected"]["samples"])
                .unwrap_err()
                .code(),
            "INVALID_CAPTURE",
        );
    }
}
