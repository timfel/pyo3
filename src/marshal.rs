#![cfg(not(Py_LIMITED_API))]

//! Support for the Python `marshal` format.

use crate::ffi;
use crate::types::{PyAny, PyBytes};
use crate::{AsPyPointer, PyResult, Python};
#[cfg(not(GraalPy))]
use crate::FromPyPointer;
use std::os::raw::c_char;
#[cfg(not(GraalPy))]
use std::os::raw::c_int;

/// The current version of the marshal binary format.
pub const VERSION: i32 = 4;

/// Serialize an object to bytes using the Python built-in marshal module.
///
/// The built-in marshalling only supports a limited range of objects.
/// The exact types supported depend on the version argument.
/// The [`VERSION`] constant holds the highest version currently supported.
///
/// See the [Python documentation](https://docs.python.org/3/library/marshal.html) for more details.
///
/// # Examples
/// ```
/// # use pyo3::{marshal, types::PyDict};
/// # pyo3::Python::with_gil(|py| {
/// let dict = PyDict::new(py);
/// dict.set_item("aap", "noot").unwrap();
/// dict.set_item("mies", "wim").unwrap();
/// dict.set_item("zus", "jet").unwrap();
///
/// let bytes = marshal::dumps(py, dict, marshal::VERSION);
/// # });
/// ```
pub fn dumps<'a>(py: Python<'a>, object: &impl AsPyPointer, version: i32) -> PyResult<&'a PyBytes> {
    #[cfg(not(GraalPy))]
    unsafe {
        let bytes = ffi::PyMarshal_WriteObjectToString(object.as_ptr(), version as c_int);
        return FromPyPointer::from_owned_ptr_or_err(py, bytes);
    }
    #[cfg(GraalPy)]
    unsafe {
        let py_locals = ffi::PyDict_New();
        ffi::PyDict_SetItem(py_locals, ffi::PyUnicode_FromString("obj".as_ptr().cast::<c_char>()), object.as_ptr());
        ffi::PyDict_SetItem(py_locals, ffi::PyUnicode_FromString("version".as_ptr().cast::<c_char>()), ffi::PyLong_FromLong(version as i64));
        return py.from_owned_ptr_or_err(ffi::PyRun_StringFlags(
            "__import__('marshal').dumps(obj, version)".as_ptr().cast::<c_char>(),
            ffi::Py_eval_input,
            ffi::PyEval_GetBuiltins(),
            py_locals,
            std::ptr::null_mut()
        ));
    }
}

/// Deserialize an object from bytes using the Python built-in marshal module.
pub fn loads<'a, B>(py: Python<'a>, data: &B) -> PyResult<&'a PyAny>
where
    B: AsRef<[u8]> + ?Sized,
{
    let data = data.as_ref();
    #[cfg(not(GraalPy))]
    unsafe {
        let c_str = data.as_ptr() as *const c_char;
        let object = ffi::PyMarshal_ReadObjectFromString(c_str, data.len() as isize);
        FromPyPointer::from_owned_ptr_or_err(py, object)
    }
    #[cfg(GraalPy)]
    unsafe {
        let py_locals = ffi::PyDict_New();
        ffi::PyDict_SetItem(py_locals, ffi::PyUnicode_FromString("data".as_ptr().cast::<c_char>()), ffi::PyBytes_FromStringAndSize(data.as_ptr() as *const c_char, data.len() as isize));
        return py.from_owned_ptr_or_err(ffi::PyRun_StringFlags(
            "__import__('marshal').loads(data)".as_ptr().cast::<c_char>(),
            ffi::Py_eval_input,
            ffi::PyEval_GetBuiltins(),
            py_locals,
            std::ptr::null_mut()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyDict;

    #[test]
    fn marshal_roundtrip() {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("aap", "noot").unwrap();
            dict.set_item("mies", "wim").unwrap();
            dict.set_item("zus", "jet").unwrap();

            let bytes = dumps(py, dict, VERSION)
                .expect("marshalling failed")
                .as_bytes();
            let deserialized = loads(py, bytes).expect("unmarshalling failed");

            assert!(equal(py, dict, deserialized));
        });
    }

    fn equal(_py: Python<'_>, a: &impl AsPyPointer, b: &impl AsPyPointer) -> bool {
        unsafe { ffi::PyObject_RichCompareBool(a.as_ptr(), b.as_ptr(), ffi::Py_EQ) != 0 }
    }
}
