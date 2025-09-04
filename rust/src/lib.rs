use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

mod swissalti3d;
mod swissimage;
pub mod terrain_viz;
mod utils;

/// An example module implemented in Rust using PyO3.
#[pymodule(name = "swiss_mountains")]
fn swiss_mountains_module(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(swissalti3d::swissalti3d_module))?;
    m.add_wrapped(wrap_pymodule!(swissimage::swissimage_module))?;

    // Inserting to sys.modules allows importing submodules nicely from Python
    // e.g. from swiss_mountains.swissalti3d import example_function

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;

    sys_modules.set_item("swiss_mountains.swissalti3d", m.getattr("swissalti3d")?)?;
    sys_modules.set_item("swiss_mountains.swissimage", m.getattr("swissimage")?)?;

    Ok(())
}
