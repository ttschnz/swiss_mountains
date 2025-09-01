use pyo3::prelude::*;
use pyo3::wrap_pymodule;

pub mod cache;
pub mod fetch;

#[pymodule(name = "swissalti3d")]
pub fn swissalti3d_module(_py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    parent.add_wrapped(wrap_pymodule!(cache::cache_module))?;
    Ok(())
}
