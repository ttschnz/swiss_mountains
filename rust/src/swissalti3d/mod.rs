use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

pub mod cache;
pub mod fetch;

#[pymodule(name = "swissalti3d")]
pub fn swissalti3d_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    parent.add_wrapped(wrap_pymodule!(cache::cache_module))?;
    parent.add_wrapped(wrap_pymodule!(fetch::fetch_module))?;

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;

    // rename swissalti3d_cache to cache
    sys_modules.set_item(
        "swiss_mountains.swissalti3d.cache",
        parent.getattr("swissalti3d_cache")?,
    )?;
    parent.setattr("cache", parent.getattr("swissalti3d_cache")?)?;
    //parent.delattr("swissalti3d_cache")?;

    // rename swissalti3d_fetch to fetch
    sys_modules.set_item(
        "swiss_mountains.swissalti3d.fetch",
        parent.getattr("swissalti3d_fetch")?,
    )?;
    parent.setattr("fetch", parent.getattr("swissalti3d_fetch")?)?;
    //parent.delattr("swissalti3d_fetch")?;

    Ok(())
}
