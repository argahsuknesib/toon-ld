//! TOON-LD Python Bindings
//!
//! Python bindings for TOON-LD serializer/parser using PyO3.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Convert JSON-LD string to TOON-LD format.
///
/// Args:
///     json_str: A JSON or JSON-LD formatted string.
///
/// Returns:
///     TOON-LD formatted string.
///
/// Raises:
///     ValueError: If the input is not valid JSON.
///
/// Example:
///     >>> import toon_ld
///     >>> json_str = '{"name": "Alice", "age": 30}'
///     >>> toon_str = toon_ld.convert_jsonld_to_toonld(json_str)
///     >>> print(toon_str)
///     age: 30
///     name: Alice
#[pyfunction]
fn convert_jsonld_to_toonld(json_str: &str) -> PyResult<String> {
    toon_core::jsonld_to_toonld(json_str).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Convert TOON-LD string to JSON-LD format.
///
/// Args:
///     toon_str: A TOON-LD formatted string.
///
/// Returns:
///     JSON-LD formatted string (pretty-printed).
///
/// Raises:
///     ValueError: If the input is not valid TOON-LD.
///
/// Example:
///     >>> import toon_ld
///     >>> toon_str = "name: Alice\nage: 30"
///     >>> json_str = toon_ld.convert_toonld_to_jsonld(toon_str)
///     >>> print(json_str)
///     {
///       "age": 30,
///       "name": "Alice"
///     }
#[pyfunction]
fn convert_toonld_to_jsonld(toon_str: &str) -> PyResult<String> {
    toon_core::toonld_to_jsonld(toon_str).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Validate a TOON-LD string.
///
/// Args:
///     toon_str: A TOON-LD formatted string.
///
/// Returns:
///     True if the string is valid TOON-LD, False otherwise.
///
/// Example:
///     >>> import toon_ld
///     >>> toon_ld.validate_toon("name: Alice")
///     True
#[pyfunction]
fn validate_toonld(toon_str: &str) -> bool {
    toon_core::toonld_to_jsonld(toon_str).is_ok()
}

/// Validate a JSON string.
///
/// Args:
///     json_str: A JSON formatted string.
///
/// Returns:
///     True if the string is valid JSON, False otherwise.
///
/// Example:
///     >>> import toon_ld
///     >>> toon_ld.validate_json('{"name": "Alice"}')
///     True
///     >>> toon_ld.validate_json('{"name": }')
///     False
#[pyfunction]
fn validate_json(json_str: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(json_str).is_ok()
}

/// Parse TOON-LD string to a Python dictionary.
///
/// Args:
///     toon_str: A TOON-LD formatted string.
///
/// Returns:
///     Python dictionary representing the parsed data.
///
/// Raises:
///     ValueError: If the input is not valid TOON-LD.
///
/// Example:
///     >>> import toon_ld
///     >>> data = toon_ld.parse_toonld("name: Alice\nage: 30")
///     >>> data['name']
///     'Alice'
#[pyfunction]
fn parse_toonld(py: Python<'_>, toon_str: &str) -> PyResult<PyObject> {
    let parser = toon_core::ToonParser::new();
    let value = parser
        .parse(toon_str)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    json_value_to_py(py, &value)
}

/// Serialize a Python dictionary to TOON-LD format.
///
/// Args:
///     data: A Python dictionary or JSON-serializable object.
///
/// Returns:
///     TOON-LD formatted string.
///
/// Raises:
///     ValueError: If the input cannot be serialized.
///
/// Example:
///     >>> import toon_ld
///     >>> data = {"name": "Alice", "age": 30}
///     >>> toon_str = toon_ld.serialize_to_toonld(data)
///     >>> print(toon_str)
///     age: 30
///     name: Alice
#[pyfunction]
fn serialize_to_toonld(py: Python<'_>, data: PyObject) -> PyResult<String> {
    let value = py_to_json_value(py, &data)?;
    let context = toon_core::JsonLdContext::from_value(&value);
    let serializer = toon_core::ToonSerializer::new().with_context(context);
    serializer
        .serialize(&value)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Convert a serde_json::Value to a Python object
fn json_value_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.into_py(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_py(py)),
        serde_json::Value::Array(arr) => {
            let list = pyo3::types::PyList::empty_bound(py);
            for item in arr {
                list.append(json_value_to_py(py, item)?)?;
            }
            Ok(list.into())
        }
        serde_json::Value::Object(obj) => {
            let dict = pyo3::types::PyDict::new_bound(py);
            for (key, val) in obj {
                dict.set_item(key, json_value_to_py(py, val)?)?;
            }
            Ok(dict.into())
        }
    }
}

/// Convert a Python object to serde_json::Value
fn py_to_json_value(py: Python<'_>, obj: &PyObject) -> PyResult<serde_json::Value> {
    // Check for None
    if obj.is_none(py) {
        return Ok(serde_json::Value::Null);
    }

    // Check for bool (must be before int since bool is a subclass of int in Python)
    if let Ok(b) = obj.extract::<bool>(py) {
        return Ok(serde_json::Value::Bool(b));
    }

    // Check for int
    if let Ok(i) = obj.extract::<i64>(py) {
        return Ok(serde_json::Value::Number(i.into()));
    }

    // Check for float
    if let Ok(f) = obj.extract::<f64>(py) {
        return serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| PyValueError::new_err("Invalid float value (NaN or Infinity)"));
    }

    // Check for string
    if let Ok(s) = obj.extract::<String>(py) {
        return Ok(serde_json::Value::String(s));
    }

    // Check for list
    if let Ok(list) = obj.downcast_bound::<pyo3::types::PyList>(py) {
        let mut arr = Vec::with_capacity(list.len());
        for item in list.iter() {
            arr.push(py_to_json_value(py, &item.into())?);
        }
        return Ok(serde_json::Value::Array(arr));
    }

    // Check for dict
    if let Ok(dict) = obj.downcast_bound::<pyo3::types::PyDict>(py) {
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str = key
                .extract::<String>()
                .map_err(|_| PyValueError::new_err("Dictionary keys must be strings"))?;
            map.insert(key_str, py_to_json_value(py, &value.into())?);
        }
        return Ok(serde_json::Value::Object(map));
    }

    // Check for tuple (treat as list)
    if let Ok(tuple) = obj.downcast_bound::<pyo3::types::PyTuple>(py) {
        let mut arr = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            arr.push(py_to_json_value(py, &item.into())?);
        }
        return Ok(serde_json::Value::Array(arr));
    }

    Err(PyValueError::new_err(format!(
        "Cannot convert Python type to JSON: {:?}",
        obj.bind(py).get_type().name()
    )))
}

/// TOON-LD Python module
///
/// High-performance serializer/parser for TOON-LD (Token-Oriented Object Notation for Linked Data).
#[pymodule]
fn toon_ld(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert_jsonld_to_toonld, m)?)?;
    m.add_function(wrap_pyfunction!(convert_toonld_to_jsonld, m)?)?;
    m.add_function(wrap_pyfunction!(validate_toonld, m)?)?;
    m.add_function(wrap_pyfunction!(validate_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_toonld, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_to_toonld, m)?)?;

    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_jsonld_to_toonld() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let result = toon_core::jsonld_to_toonld(json);
        assert!(result.is_ok());
        let toon = result.unwrap();
        assert!(toon.contains("name: Alice"));
        assert!(toon.contains("age: 30"));
    }

    #[test]
    fn test_convert_toonld_to_jsonld() {
        let toon = "name: Alice\nage: 30";
        let result = toon_core::toonld_to_jsonld(toon);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"Alice\""));
    }

    #[test]
    fn test_validate_functions() {
        assert!(validate_toonld("name: Alice"));
        assert!(validate_json(r#"{"name": "Alice"}"#));
        assert!(!validate_json(r#"{"name": }"#));
    }

    #[test]
    fn test_tabular_array() {
        let json = r#"{
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]
        }"#;
        let toon = toon_core::jsonld_to_toonld(json).unwrap();
        assert!(toon.contains("users[2]"));
    }
}
