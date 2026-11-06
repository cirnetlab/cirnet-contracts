use super::{ValidationIssue, child};
use num_bigint::BigInt;
use serde_json::{Number, Value};
use std::{borrow::Cow, cmp::Ordering};

fn integral_token(number: &Number) -> bool {
    !number.as_str().contains(['.', 'e', 'E'])
}

pub(super) fn lexical_integer(value: &Value) -> Option<BigInt> {
    let number = value.as_number()?;
    integral_token(number).then(|| integer(number))
}

fn nonfinite(value: &Value) -> bool {
    value
        .as_number()
        .is_some_and(|number| !integral_token(number) && number.as_f64().is_none())
}

/// The schema view compares integral binary64 values against exact bounds.
/// The output view preserves the distinction between integer and float tokens.
#[derive(Clone, Copy)]
pub(super) enum NumberView {
    Schema,
    Output,
}

/// Match the legacy JSON reader's numeric model and preserve exact integer tokens.
pub(super) fn normalize_binary64<'a>(
    value: &'a Value,
    path: &str,
    view: NumberView,
) -> Result<Cow<'a, Value>, String> {
    let mut normalized = Cow::Borrowed(value);
    match value {
        Value::Number(number) if !integral_token(number) => {
            let rounded = number.as_f64().ok_or_else(|| path.to_owned())?;
            let converted = if matches!(view, NumberView::Schema) && rounded.fract() == 0.0 {
                // Formatting the exact binary64 integer also handles exponents
                // beyond machine integer widths and underflow to signed zero.
                let integer = integer_from_float(rounded);
                serde_json::from_str(&integer.to_string()).expect("decimal integer")
            } else {
                Number::from_f64(rounded).expect("finite binary64")
            };
            if *number != converted {
                normalized = Cow::Owned(Value::Number(converted));
            }
        }
        Value::Object(entries) => {
            for (key, value) in entries {
                if let Cow::Owned(value) = normalize_binary64(value, &child(path, key), view)? {
                    normalized.to_mut()[key] = value;
                }
            }
        }
        Value::Array(entries) => {
            for (index, value) in entries.iter().enumerate() {
                if let Cow::Owned(value) = normalize_binary64(value, &child(path, index), view)? {
                    normalized.to_mut()[index] = value;
                }
            }
        }
        _ => {}
    }
    Ok(normalized)
}

pub(super) fn check_finite(
    value: &Value,
    path: &str,
    profile: &str,
) -> Result<(), ValidationIssue> {
    match value {
        Value::Number(_) if nonfinite(value) => {
            return Err(ValidationIssue::new(
                "NONFINITE_NUMBER",
                path,
                "Nonfinite numeric value",
                profile,
            ));
        }
        Value::Object(entries) => {
            for (key, value) in entries {
                check_finite(value, &child(path, key), profile)?;
            }
        }
        Value::Array(entries) => {
            for (index, value) in entries.iter().enumerate() {
                check_finite(value, &child(path, index), profile)?;
            }
        }
        _ => {}
    }
    Ok(())
}

// Preserve arbitrary precision integer tokens. JSON decimal tokens use the
// finite binary64 values observed by the legacy JSON reader. Compare an
// integer and a binary64 through its exact IEEE mantissa and exponent.
pub(super) fn compare(left: &Value, right: &Value) -> Ordering {
    let (Some(left), Some(right)) = (left.as_number(), right.as_number()) else {
        return Ordering::Equal;
    };
    match (integral_token(left), integral_token(right)) {
        (true, true) => integer(left).cmp(&integer(right)),
        (true, false) => compare_int_float(left, float(right)),
        (false, true) => compare_int_float(right, float(left)).reverse(),
        (false, false) => float(left)
            .partial_cmp(&float(right))
            .unwrap_or(Ordering::Equal),
    }
}

fn float(number: &Number) -> f64 {
    number.as_str().parse::<f64>().unwrap_or(f64::NAN)
}

fn integer(number: &Number) -> BigInt {
    number.as_str().parse().expect("integer JSON token")
}

fn compare_int_float(number: &Number, float: f64) -> Ordering {
    if float.is_infinite() {
        return if float.is_sign_positive() {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    let (significand, exponent) = float_parts(float);
    let integer = integer(number);
    if exponent >= 0 {
        integer.cmp(&(significand << exponent as usize))
    } else {
        (integer << (-exponent) as usize).cmp(&significand)
    }
}

fn float_parts(float: f64) -> (BigInt, i32) {
    let bits = float.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1 << 52) - 1);
    let (significand, exponent) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), exponent - 1075)
    };
    let significand = BigInt::from(significand);
    (
        if float.is_sign_negative() {
            -significand
        } else {
            significand
        },
        exponent,
    )
}

pub(super) fn exact_integer(value: &Value) -> BigInt {
    let number = value.as_number().expect("integer-validated value");
    if integral_token(number) {
        integer(number)
    } else {
        integer_from_float(float(number))
    }
}

fn integer_from_float(float: f64) -> BigInt {
    let (significand, exponent) = float_parts(float);
    if exponent >= 0 {
        significand << exponent as usize
    } else {
        significand >> (-exponent) as usize
    }
}

pub(super) fn compare_count(count: usize, value: &Value) -> Ordering {
    compare(&Value::from(count), value)
}
