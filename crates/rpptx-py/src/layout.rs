//! Frozen snapshots of the deterministic slide text layout, in points.

use pyo3::prelude::*;
use pyo3::types::PyTuple;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyBoundingBox>()?;
    module.add_class::<PyTextLineLayout>()?;
    module.add_class::<PyTextFrameLayout>()?;
    module.add_class::<PyAutofitUpdate>()?;
    module.add_class::<PyAutofitRefreshResult>()?;
    Ok(())
}

#[pyclass(name = "BoundingBox", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, Copy, PartialEq)]
pub struct PyBoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[pyclass(name = "TextLineLayout", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct PyTextLineLayout {
    paragraph_index: usize,
    text: String,
    bounds: PyBoundingBox,
    baseline: f64,
    font_size: f64,
}

#[pymethods]
impl PyTextLineLayout {
    #[getter]
    fn paragraph_index(&self) -> usize {
        self.paragraph_index
    }

    #[getter]
    fn text(&self) -> &str {
        &self.text
    }

    #[getter]
    fn bounds(&self) -> PyBoundingBox {
        self.bounds
    }

    #[getter]
    fn baseline(&self) -> f64 {
        self.baseline
    }

    #[getter]
    fn font_size(&self) -> f64 {
        self.font_size
    }
}

#[pyclass(name = "TextFrameLayout", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct PyTextFrameLayout {
    slide_index: usize,
    shape_id: Option<u32>,
    name: Option<String>,
    autofit: &'static str,
    frame: PyBoundingBox,
    usable: PyBoundingBox,
    font_scale: f64,
    height: f64,
    overflow: bool,
    lines: Vec<PyTextLineLayout>,
    diagnostics: Vec<String>,
}

impl From<&rpptx::TextFrameLayout> for PyTextFrameLayout {
    fn from(frame: &rpptx::TextFrameLayout) -> Self {
        let layout = &frame.layout;
        Self {
            slide_index: frame.slide_index,
            diagnostics: frame
                .diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect(),
            shape_id: frame.shape_id,
            name: frame.name.clone(),
            autofit: match frame.autofit {
                rpptx::AutofitMode::None => "none",
                rpptx::AutofitMode::Normal => "normal",
                rpptx::AutofitMode::Shape => "shape",
            },
            frame: PyBoundingBox {
                x: layout.frame.x,
                y: layout.frame.y,
                width: layout.frame.width,
                height: layout.frame.height,
            },
            usable: PyBoundingBox {
                x: layout.usable.x,
                y: layout.usable.y,
                width: layout.usable.width,
                height: layout.usable.height,
            },
            font_scale: layout.font_scale,
            height: layout.height,
            overflow: layout.overflow,
            lines: layout
                .lines
                .iter()
                .map(|line| PyTextLineLayout {
                    paragraph_index: line.paragraph_index,
                    text: line.text.clone(),
                    bounds: PyBoundingBox {
                        x: line.bounds.x,
                        y: line.bounds.y,
                        width: line.bounds.width,
                        height: line.bounds.height,
                    },
                    baseline: line.baseline,
                    font_size: line.font_size,
                })
                .collect(),
        }
    }
}

#[pymethods]
impl PyTextFrameLayout {
    #[getter]
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.diagnostics)
    }

    #[getter]
    fn slide_index(&self) -> usize {
        self.slide_index
    }

    #[getter]
    fn shape_id(&self) -> Option<u32> {
        self.shape_id
    }

    #[getter]
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[getter]
    fn autofit(&self) -> &'static str {
        self.autofit
    }

    #[getter]
    fn frame(&self) -> PyBoundingBox {
        self.frame
    }

    #[getter]
    fn usable(&self) -> PyBoundingBox {
        self.usable
    }

    #[getter]
    fn font_scale(&self) -> f64 {
        self.font_scale
    }

    #[getter]
    fn height(&self) -> f64 {
        self.height
    }

    #[getter]
    fn overflow(&self) -> bool {
        self.overflow
    }

    #[getter]
    fn lines<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.lines.iter().cloned())
    }
}

/// Historical snapshot, never a mutable native owner selector.
#[pyclass(name = "AutofitUpdate", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct PyAutofitUpdate {
    #[pyo3(get)]
    owner_part: String,
    #[pyo3(get)]
    owner_kind: String,
    #[pyo3(get)]
    slide_index: Option<usize>,
    #[pyo3(get)]
    shape_id: Option<u32>,
    #[pyo3(get)]
    cell: Option<(usize, usize)>,
    #[pyo3(get)]
    mode: &'static str,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    font_scale: Option<u32>,
    #[pyo3(get)]
    line_spacing_reduction: Option<u32>,
    #[pyo3(get)]
    point_size: Option<u16>,
    #[pyo3(get)]
    old_bounds: (i64, i64, i64, i64),
    #[pyo3(get)]
    new_bounds: (i64, i64, i64, i64),
    #[pyo3(get)]
    changed: bool,

    diagnostics: Vec<String>,
}

impl From<&rpptx::AutofitUpdate> for PyAutofitUpdate {
    fn from(update: &rpptx::AutofitUpdate) -> Self {
        Self {
            owner_part: update.location.owner_part.clone(),
            owner_kind: update.owner_kind.clone(),
            slide_index: update.slide_index,
            shape_id: update.shape_id,
            cell: update.location.cell,
            mode: match update.mode {
                rpptx::AutofitMode::None => "none",
                rpptx::AutofitMode::Normal => "normal",
                rpptx::AutofitMode::Shape => "shape",
            },
            status: update.status.clone(),
            font_scale: update.font_scale,
            line_spacing_reduction: update.line_spacing_reduction,
            point_size: update.point_size,
            old_bounds: update.old_bounds,
            new_bounds: update.new_bounds,
            changed: update.changed,
            diagnostics: update
                .diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect(),
        }
    }
}

#[pymethods]
impl PyAutofitUpdate {
    #[getter]
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.diagnostics)
    }
}

#[pyclass(name = "AutofitRefreshResult", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq)]
pub struct PyAutofitRefreshResult {
    updates: Vec<PyAutofitUpdate>,
    diagnostics: Vec<String>,
    changed: bool,
}

impl From<&rpptx::AutofitRefreshResult> for PyAutofitRefreshResult {
    fn from(result: &rpptx::AutofitRefreshResult) -> Self {
        Self {
            updates: result.updates.iter().map(PyAutofitUpdate::from).collect(),
            diagnostics: result
                .diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect(),
            changed: result.changed,
        }
    }
}

#[pymethods]
impl PyAutofitRefreshResult {
    #[getter]
    fn updates<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.updates.iter().cloned())
    }
    #[getter]
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.diagnostics)
    }
    #[getter]
    fn changed(&self) -> bool {
        self.changed
    }
}
