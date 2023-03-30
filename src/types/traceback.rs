// Copyright (c) 2017-present PyO3 Project and Contributors

#[cfg(not(GraalPy))]
use crate::err::error_on_minusone;
use crate::err::PyResult;
use crate::ffi;
use crate::types::PyString;
use crate::{AsPyPointer, PyAny};
#[cfg(GraalPy)]
use std::os::raw::c_char;

/// Represents a Python traceback.
#[repr(transparent)]
pub struct PyTraceback(PyAny);

pyobject_native_type_core!(
    PyTraceback,
    ffi::PyTraceBack_Type,
    #checkfunction=ffi::PyTraceBack_Check
);

impl PyTraceback {
    /// Formats the traceback as a string.
    ///
    /// This does not include the exception type and value. The exception type and value can be
    /// formatted using the `Display` implementation for `PyErr`.
    ///
    /// # Example
    ///
    /// The following code formats a Python traceback and exception pair from Rust:
    ///
    /// ```rust
    /// # use pyo3::{Python, PyResult};
    /// # let result: PyResult<()> =
    /// Python::with_gil(|py| {
    ///     let err = py
    ///         .run("raise Exception('banana')", None, None)
    ///         .expect_err("raise will create a Python error");
    ///
    ///     let traceback = err.traceback(py).expect("raised exception will have a traceback");
    ///     assert_eq!(
    ///         format!("{}{}", traceback.format()?, err),
    ///         "\
    /// Traceback (most recent call last):
    ///   File \"<string>\", line 1, in <module>
    /// Exception: banana\
    /// "
    ///     );
    ///     Ok(())
    /// })
    /// # ;
    /// # result.expect("example failed");
    /// ```
    pub fn format(&self) -> PyResult<String> {
        let py = self.py();
        let string_io = py
            .import(intern!(py, "io"))?
            .getattr(intern!(py, "StringIO"))?
            .call0()?;
        #[cfg(not(GraalPy))]
        let result = unsafe { ffi::PyTraceBack_Print(self.as_ptr(), string_io.as_ptr()) };
        #[cfg(not(GraalPy))]
        error_on_minusone(py, result)?;
        #[cfg(GraalPy)]
        unsafe {
            let py_locals = ffi::PyDict_New();
            ffi::PyDict_SetItem(py_locals, ffi::PyUnicode_FromString("traceback".as_ptr().cast::<c_char>()), self.as_ptr());
            ffi::PyDict_SetItem(py_locals, ffi::PyUnicode_FromString("stringio".as_ptr().cast::<c_char>()), string_io.as_ptr());
            let result: PyResult<&PyAny> = py.from_owned_ptr_or_err(ffi::PyRun_StringFlags(
                "__import__('traceback').print_tb(traceback, file=stringio)".as_ptr().cast::<c_char>(),
                ffi::Py_eval_input,
                ffi::PyEval_GetBuiltins(),
                py_locals,
                std::ptr::null_mut()
            ));
            if let Err(e) = result {
                return Err(e);
            }
        };
        let formatted = string_io
            .getattr(intern!(py, "getvalue"))?
            .call0()?
            .downcast::<PyString>()?
            .to_str()?
            .to_owned();
        Ok(formatted)
    }
}

#[cfg(test)]
mod tests {
    use crate::Python;

    #[test]
    fn format_traceback() {
        Python::with_gil(|py| {
            let err = py
                .run("raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");

            assert_eq!(
                err.traceback(py).unwrap().format().unwrap(),
                "Traceback (most recent call last):\n  File \"<string>\", line 1, in <module>\n"
            );
        })
    }
}
