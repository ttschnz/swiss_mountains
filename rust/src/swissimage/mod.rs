use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

pub mod cache;
pub mod fetch;

#[pymodule(name = "swissimage")]
pub fn swissimage_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    parent.add_wrapped(wrap_pymodule!(cache::cache_module))?;
    parent.add_wrapped(wrap_pymodule!(fetch::fetch_module))?;

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;

    // rename swissimage_cache to cache
    sys_modules.set_item(
        "swiss_mountains.swissimage.cache",
        parent.getattr("swissimage_cache")?,
    )?;
    parent.setattr("cache", parent.getattr("swissimage_cache")?)?;
    // parent.delattr("swissimage_cache")?;

    // rename swissimage_fetch to fetch
    sys_modules.set_item(
        "swiss_mountains.swissimage.fetch",
        parent.getattr("swissimage_fetch")?,
    )?;
    parent.setattr("fetch", parent.getattr("swissimage_fetch")?)?;
    //parent.delattr("swissimage_fetch")?;

    Ok(())
}
