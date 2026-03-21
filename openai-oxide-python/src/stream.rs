use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

#[pyclass]
pub struct PyResponseStream {
    pub receiver: std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Result<String, String>>>>,
}

#[pymethods]
impl PyResponseStream {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        let receiver = self.receiver.clone();
        let fut = future_into_py(py, async move {
            let mut rx = receiver.lock().await;
            match rx.recv().await {
                Some(Ok(s)) => Ok(s),
                Some(Err(e)) => Err(pyo3::exceptions::PyRuntimeError::new_err(e)),
                None => Err(pyo3::exceptions::PyStopAsyncIteration::new_err("")),
            }
        })?;
        Ok(Some(fut))
    }
}
