use super::MAX_SAFE_INTEGER;
use crate::wire_json::RawObject;
use serde::de;
use serde_json::{Map, Number, Value, value::RawValue};

const MAX_DEPTH: usize = 128;

pub(super) fn parse(bytes: &[u8], sampled: bool) -> Option<Value> {
    let raw: &RawValue = serde_json::from_slice(bytes).ok()?;
    strict_value(raw, sampled, 0).ok()
}

fn strict_value(raw: &RawValue, sampled: bool, depth: usize) -> serde_json::Result<Value> {
    let text = raw.get();
    if depth >= MAX_DEPTH {
        return Err(de::Error::custom("nested capture header"));
    }
    match text.as_bytes()[0] {
        b'{' => {
            let RawObject(members) = serde_json::from_str(text)?;
            let mut object = Map::new();
            for (key, raw_child) in members {
                let child = strict_value(raw_child, sampled, depth + 1)?;
                if sampled && integer_member(&key) && !child.is_i64() {
                    return Err(de::Error::custom("expected integer member"));
                }
                if object.insert(key, child).is_some() {
                    return Err(de::Error::custom("duplicate capture header member"));
                }
            }
            Ok(Value::Object(object))
        }
        b'[' => serde_json::from_str::<Vec<&RawValue>>(text)?
            .into_iter()
            .map(|child| strict_value(child, sampled, depth + 1))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        _ => match serde_json::from_str(text)? {
            Value::Number(number) if sampled => checked_sampled_number(number)
                .map(Value::Number)
                .ok_or_else(|| de::Error::custom("unrepresentable capture header number")),
            value => Ok(value),
        },
    }
}

fn integer_member(key: &str) -> bool {
    matches!(
        key,
        "generation"
            | "frame_count"
            | "numerator"
            | "denominator"
            | "lost_frames"
            | "at_frame"
            | "missing_frames"
            | "first_frame"
    )
}

fn checked_sampled_number(number: Number) -> Option<Number> {
    if let Some(value) = number.as_i64() {
        return (value.unsigned_abs() <= MAX_SAFE_INTEGER).then_some(number);
    }
    if !number.is_f64() {
        return None;
    }
    let rounded = Number::from_f64(number.as_f64()?)?;
    // Verify the original wire decimal, then pass only the normalized binary64
    // token downstream to schema validation.
    (decimal(number.as_str())? == decimal(rounded.as_str())?).then_some(rounded)
}

// Compare base-ten digits and powers exactly, just as Decimal does for the NCAP2 header.
#[derive(PartialEq)]
struct Decimal {
    negative: bool,
    digits: String,
    exponent: i64,
}

fn decimal(token: &str) -> Option<Decimal> {
    let negative = token.starts_with('-');
    let unsigned = token.strip_prefix('-').unwrap_or(token);
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, exponent.parse::<i64>().ok()?),
        None => (unsigned, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let exponent = exponent.checked_sub(fraction.len() as i64)?;
    // The reference's 64-bit Decimal parser bounds the coefficient exponent
    // (MIN_ETINY..=MAX_EMAX), including on zero-valued tokens.
    if !(-1_999_999_999_999_999_997..=999_999_999_999_999_999).contains(&exponent) {
        return None;
    }
    let digits = format!("{whole}{fraction}");
    let significant = digits.trim_start_matches('0');
    if significant.is_empty() {
        return Some(Decimal {
            negative: false,
            digits: "0".to_owned(),
            exponent: 0,
        });
    }
    let trimmed = significant.trim_end_matches('0');
    Some(Decimal {
        negative,
        digits: trimmed.to_owned(),
        exponent: exponent.checked_add((significant.len() - trimmed.len()) as i64)?,
    })
}

pub(super) fn encode_sampled_header(header: &Value, max_bytes: usize) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    write_value(&mut bytes, header, 0, max_bytes)?;
    Some(bytes)
}

fn write_value(bytes: &mut Vec<u8>, value: &Value, depth: usize, max_bytes: usize) -> Option<()> {
    if depth >= MAX_DEPTH || bytes.len() >= max_bytes {
        return None;
    }
    match value {
        Value::Object(object) => {
            bytes.push(b'{');
            for (index, (key, child)) in object.iter().enumerate() {
                if index != 0 {
                    bytes.push(b',');
                }
                if key.len() > max_bytes.saturating_sub(bytes.len()) {
                    return None;
                }
                serde_json::to_writer(&mut *bytes, key).ok()?;
                bytes.push(b':');
                write_value(bytes, child, depth + 1, max_bytes)?;
            }
            bytes.push(b'}');
        }
        Value::Array(array) => {
            bytes.push(b'[');
            for (index, child) in array.iter().enumerate() {
                if index != 0 {
                    bytes.push(b',');
                }
                write_value(bytes, child, depth + 1, max_bytes)?;
            }
            bytes.push(b']');
        }
        Value::Number(number) => {
            if number.is_f64() {
                bytes.extend(python_float(number.as_f64()?)?.bytes());
            } else {
                let integer = number.as_i64()?;
                if integer.unsigned_abs() > MAX_SAFE_INTEGER {
                    return None;
                }
                serde_json::to_writer(&mut *bytes, &integer).ok()?;
            }
        }
        Value::String(text) if text.len() > max_bytes - bytes.len() => return None,
        _ => serde_json::to_writer(&mut *bytes, value).ok()?,
    }
    (bytes.len() <= max_bytes).then_some(())
}

fn python_float(value: f64) -> Option<String> {
    if value == 0.0 {
        return Some(
            if value.is_sign_negative() {
                "-0.0"
            } else {
                "0.0"
            }
            .to_owned(),
        );
    }
    let mut buffer = ryu::Buffer::new();
    let Decimal {
        negative,
        digits,
        exponent,
    } = decimal(buffer.format_finite(value))?;
    let scientific_exponent = digits.len() as i64 + exponent - 1;
    let mut text = if negative {
        "-".to_owned()
    } else {
        String::new()
    };
    if !(-4..16).contains(&scientific_exponent) {
        text.push_str(&digits[..1]);
        if digits.len() > 1 {
            text.push('.');
            text.push_str(&digits[1..]);
        }
        text.push_str(&format!("e{scientific_exponent:+03}"));
    } else if scientific_exponent < 0 {
        text.push_str("0.");
        text.extend(std::iter::repeat_n(
            '0',
            (-scientific_exponent - 1) as usize,
        ));
        text.push_str(&digits);
    } else {
        let whole_digits = scientific_exponent as usize + 1;
        if whole_digits >= digits.len() {
            text.push_str(&digits);
            text.extend(std::iter::repeat_n('0', whole_digits - digits.len()));
            text.push_str(".0");
        } else {
            text.push_str(&digits[..whole_digits]);
            text.push('.');
            text.push_str(&digits[whole_digits..]);
        }
    }
    Some(text)
}

pub(super) fn python_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => match (left.is_f64(), right.is_f64()) {
            (false, false) => decimal(left.as_str()) == decimal(right.as_str()),
            (true, true) => left.as_f64() == right.as_f64(),
            (true, false) => float_equals_integer(left, right),
            (false, true) => float_equals_integer(right, left),
        },
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len() && left.iter().zip(right).all(|(a, b)| python_equal(a, b))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .all(|(key, a)| right.get(key).is_some_and(|b| python_equal(a, b)))
        }
        _ => left == right,
    }
}

fn float_equals_integer(float: &Number, integer: &Number) -> bool {
    float.as_f64().is_some_and(|value| {
        value.fract() == 0.0 && decimal(&format!("{value:.0}")) == decimal(integer.as_str())
    })
}
