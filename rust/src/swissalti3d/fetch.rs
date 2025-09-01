use pyo3::prelude::*;

fn example_function(x: i32) -> i32 {
    x - 42
}

#[pyfunction(name = "example_function_fetch")]
pub fn example_function_python_wrapper(x: i32) -> PyResult<i32> {
    Ok(example_function(x))
}

#[pymodule(name = "fetch")]
pub fn fetch_module(_py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    parent.add_function(wrap_pyfunction!(example_function_python_wrapper, parent)?)?;
    Ok(())
}
