use crate::capture;
use crate::hardware_v2::{FILE_DESCRIPTOR_SET, LEGACY_FILE_DESCRIPTOR_SET};
use crate::validation::{Registry, ValidationError};
use crate::wire_json;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use serde_json::Value;
use std::path::Path;

fn capture_input(input: &str, error: capture::CaptureError) -> PyResult<Value> {
    wire_json::parse(input.as_bytes()).map_err(|_| capture_error(error))
}

fn capture_error(error: capture::CaptureError) -> PyErr {
    PyValueError::new_err(error.code())
}

#[pyclass]
struct Contracts {
    registry: Registry,
}

#[pymethods]
impl Contracts {
    #[new]
    fn new(root: &str) -> PyResult<Self> {
        let registry = Registry::load(Path::new(root))
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        Ok(Self { registry })
    }

    fn validate(&self, name: &str, profile: &str, value: &str) -> PyResult<String> {
        let value = wire_json::parse(value.as_bytes())
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        self.registry
            .validate(name, profile, &value)
            .map(|value| value.to_string())
            .map_err(|error| match error {
                ValidationError::Invalid(issue) => PyValueError::new_err(
                    serde_json::to_string(&issue).expect("serializable issue"),
                ),
                other => PyValueError::new_err(other.to_string()),
            })
    }
}

#[pyfunction]
fn encode_digital<'py>(
    py: Python<'py>,
    samples: &str,
    duration_ns: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let invalid = capture::CaptureError::InvalidCapture;
    let samples = capture_input(samples, invalid)?;
    let duration_ns = capture_input(duration_ns, invalid)?;
    let bytes = capture::encode_digital(&samples, &duration_ns).map_err(capture_error)?;
    Ok(PyBytes::new(py, &bytes))
}

#[pyfunction]
fn decode_digital(payload: &[u8]) -> PyResult<String> {
    capture::decode_digital(payload)
        .map(|value| value.to_string())
        .map_err(capture_error)
}

#[pyfunction]
fn encode_sampled<'py>(
    py: Python<'py>,
    header: &str,
    samples: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let invalid = capture::CaptureError::InvalidCapture;
    let header = capture_input(header, invalid)?;
    let samples = capture_input(samples, invalid)?;
    let bytes = capture::encode_sampled(&header, &samples).map_err(capture_error)?;
    Ok(PyBytes::new(py, &bytes))
}

#[pyfunction]
fn decode_sampled(payload: &[u8], descriptor: &str) -> PyResult<String> {
    let descriptor = capture_input(descriptor, capture::CaptureError::CaptureCorrupt)?;
    capture::decode_sampled(payload, &descriptor)
        .map(|value| value.to_string())
        .map_err(capture_error)
}

#[pyfunction]
fn sampled_frame_time(capture: &str, frame: &str) -> PyResult<(String, u64)> {
    let invalid = capture::CaptureError::InvalidFrame;
    let capture = capture_input(capture, invalid)?;
    let frame = capture_input(frame, invalid)?;
    capture::sampled_frame_time(&capture, &frame).map_err(capture_error)
}

#[pyfunction]
fn hardware_descriptor(py: Python<'_>, legacy: bool) -> Bound<'_, PyBytes> {
    let descriptor = if legacy {
        LEGACY_FILE_DESCRIPTOR_SET
    } else {
        FILE_DESCRIPTOR_SET
    };
    PyBytes::new(py, descriptor)
}

#[pymodule]
fn cirnet_contracts_native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Contracts>()?;
    module.add_function(wrap_pyfunction!(encode_digital, module)?)?;
    module.add_function(wrap_pyfunction!(decode_digital, module)?)?;
    module.add_function(wrap_pyfunction!(encode_sampled, module)?)?;
    module.add_function(wrap_pyfunction!(decode_sampled, module)?)?;
    module.add_function(wrap_pyfunction!(sampled_frame_time, module)?)?;
    module.add_function(wrap_pyfunction!(hardware_descriptor, module)?)?;
    Ok(())
}
