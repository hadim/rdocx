use oxml_py_support::{ContentPath, PathSeg};
use pyo3::exceptions::{PyIndexError, PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PySlice};

use crate::document::PyDocument;
use crate::paragraph::{ParagraphLocation, paragraph_location};
use crate::{normalize_index, stale_to_pyerr};

fn path_indices(path: &ContentPath) -> PyResult<(ParagraphLocation, usize)> {
    let run = path.segs.iter().find_map(|segment| match segment {
        PathSeg::Run(index) => Some(*index),
        _ => None,
    });
    run.map(|run| paragraph_location(path).map(|paragraph| (paragraph, run)))
        .transpose()?
        .ok_or_else(|| PyRuntimeError::new_err("run path is incomplete"))
}

#[pyclass(name = "Run")]
pub struct PyRun {
    document: Py<PyDocument>,
    path: ContentPath,
    run_path: rdocx::AcceptedRunPath,
    related: Option<rdocx::ContentLocation>,
}

impl PyRun {
    pub(crate) fn new(
        document: Py<PyDocument>,
        path: ContentPath,
        run_path: rdocx::AcceptedRunPath,
        related: Option<rdocx::ContentLocation>,
    ) -> Self {
        Self {
            document,
            path,
            run_path,
            related,
        }
    }

    pub(crate) fn validate(&self, py: Python<'_>) -> PyResult<(ParagraphLocation, usize)> {
        let document = self.document.borrow(py);
        self.path
            .validate_scopes(
                &document.revisions,
                "run",
                "Re-fetch it with paragraph.runs[i].",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        let (_, index) = path_indices(&self.path)?;
        Ok((
            match &self.related {
                Some(location) => ParagraphLocation::Related(location.clone()),
                None => paragraph_location(&self.path)?,
            },
            index,
        ))
    }

    pub(crate) fn belongs_to(&self, py: Python<'_>, document: &Py<PyDocument>) -> bool {
        self.document.bind(py).is(document.bind(py))
    }

    /// Apply one edit to this run where it lives, in the body or in a cell.
    fn edit(&self, py: Python<'_>, edit: impl FnOnce(&mut rdocx::run::Run<'_>)) -> PyResult<()> {
        let (location, _) = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        location
            .edit(&mut candidate, |paragraph| {
                paragraph.edit_run(&self.run_path, edit)
            })
            .map_err(|error| crate::rdocx_to_pyerr(py, error))?;
        document.inner = candidate;
        Ok(())
    }
}

#[pymethods]
impl PyRun {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        let (location, run_index) = self.validate(py)?;
        let document = self.document.borrow(py);
        location
            .read(&document.inner, |paragraph| {
                paragraph.run(run_index).map(|run| run.text())
            })
            .map_err(|error| crate::rdocx_to_pyerr(py, error))?
            .ok_or_else(|| PyIndexError::new_err("run index out of range"))
    }

    #[setter]
    fn set_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        self.edit(py, |run| run.set_text(text))
    }

    #[pyo3(signature = (data, filename, width = None, height = None))]
    fn add_picture(
        &self,
        py: Python<'_>,
        data: &[u8],
        filename: &str,
        width: Option<&Bound<'_, PyAny>>,
        height: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let (location, index) = self.validate(py)?;
        let width = crate::document::checked_section_length(width, "picture width")?
            .map(rdocx::Length::emu);
        let height = crate::document::checked_section_length(height, "picture height")?
            .map(rdocx::Length::emu);
        let mut document = self.document.borrow_mut(py);
        let position = rdocx::StoryRunPosition {
            location: location
                .native(&document.inner)
                .map_err(|error| crate::rdocx_to_pyerr(py, error))?,
            run_index: index + 1,
        };
        let mut candidate = document.inner.clone_for_staging();
        candidate
            .insert_picture_at(&position, data, filename, width, height)
            .map_err(|error| crate::rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Run.add_picture", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(())
    }

    // A tab is appended inside this run, so no run index moves and live
    // handles stay valid.
    fn add_tab(&self, py: Python<'_>) -> PyResult<()> {
        self.edit(py, |run| run.add_tab())
    }

    #[pyo3(signature = (kind = "line"))]
    fn add_break(&self, py: Python<'_>, kind: &str) -> PyResult<()> {
        let kind = match kind {
            "line" => rdocx::BreakKind::Line,
            "page" => rdocx::BreakKind::Page,
            "column" => rdocx::BreakKind::Column,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "break kind must be line, page or column",
                ));
            }
        };
        self.edit(py, |run| run.add_break(kind))
    }

    #[pyo3(signature = (instruction, cached_result = ""))]
    fn add_field(&self, py: Python<'_>, instruction: &str, cached_result: &str) -> PyResult<()> {
        let (location, _) = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        location
            .edit(&mut candidate, |paragraph| {
                let mut added = Ok(());
                paragraph.edit_run(&self.run_path, |run| {
                    added = run.add_field(instruction, cached_result)
                })?;
                added
            })
            .map_err(|error| crate::rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Run.add_field", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(())
    }

    /// Remove this run from its paragraph, keeping the markers around it.
    fn remove(&self, py: Python<'_>) -> PyResult<()> {
        let (location, run_index) = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        location
            .edit(&mut candidate, |paragraph| paragraph.remove_run(run_index))
            .map_err(|error| crate::rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Run.remove", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(())
    }

    #[getter]
    fn font(&self, py: Python<'_>) -> PyResult<Py<crate::formatting::PyFont>> {
        self.validate(py)?;
        Py::new(
            py,
            crate::formatting::PyFont::new(
                self.document.clone_ref(py),
                self.path.clone(),
                self.run_path.clone(),
                self.related.clone(),
            ),
        )
    }

    #[getter]
    fn style_id(&self, py: Python<'_>) -> PyResult<Option<String>> {
        let (location, run_index) = self.validate(py)?;
        Ok(crate::formatting::run_snapshot(py, &self.document, location, run_index)?.style_id)
    }

    #[setter]
    fn set_style_id(&self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
        let (location, _) = self.validate(py)?;
        crate::formatting::apply_run_update(
            py,
            &self.document,
            location,
            &self.run_path,
            crate::formatting::FontUpdate::Style(value),
        )
    }
}

#[pyclass(name = "RunCollection")]
pub struct PyRunCollection {
    document: Py<PyDocument>,
    paragraph_path: ContentPath,
    related: Option<rdocx::ContentLocation>,
}

impl PyRunCollection {
    pub(crate) fn new(
        document: Py<PyDocument>,
        paragraph_path: ContentPath,
        related: Option<rdocx::ContentLocation>,
    ) -> Self {
        Self {
            document,
            paragraph_path,
            related,
        }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<ParagraphLocation> {
        let document = self.document.borrow(py);
        self.paragraph_path
            .validate_scopes(
                &document.revisions,
                "run collection",
                "Re-fetch it with paragraph.runs.",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        match &self.related {
            Some(location) => Ok(ParagraphLocation::Related(location.clone())),
            None => paragraph_location(&self.paragraph_path),
        }
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let location = self.validate(py)?;
        let document = self.document.borrow(py);
        location
            .read(&document.inner, |paragraph| paragraph.run_count())
            .map_err(|error| crate::rdocx_to_pyerr(py, error))
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyRun>> {
        let location = self.validate(py)?;
        let (path, run_path) = {
            let document = self.document.borrow(py);
            let mut segments = self.paragraph_path.segs.clone();
            segments.push(PathSeg::Run(index));
            let run_path = location
                .read(&document.inner, |paragraph| paragraph.run_path(index))
                .map_err(|error| crate::rdocx_to_pyerr(py, error))?
                .ok_or_else(|| PyIndexError::new_err("run index out of range"))?;
            (document.revisions.capture(segments), run_path)
        };
        Py::new(
            py,
            PyRun::new(
                self.document.clone_ref(py),
                path,
                run_path,
                self.related.clone(),
            ),
        )
    }
}

#[pymethods]
impl PyRunCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let len = self.len(py)?;
        if let Ok(index) = key.extract::<isize>() {
            return Ok(self
                .item(py, normalize_index(index, len, "run")?)?
                .into_any());
        }
        if key.is_instance_of::<PySlice>() {
            let (start, stop, step): (isize, isize, isize) =
                key.call_method1("indices", (len,))?.extract()?;
            let items = PyList::empty(py);
            let mut index = start;
            while if step > 0 { index < stop } else { index > stop } {
                items.append(self.item(py, index as usize)?)?;
                index += step;
            }
            return Ok(items.into_any().unbind());
        }
        Err(PyTypeError::new_err(
            "run indices must be integers or slices",
        ))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyRunIterator>> {
        self.validate(py)?;
        Py::new(
            py,
            PyRunIterator {
                document: self.document.clone_ref(py),
                paragraph_path: self.paragraph_path.clone(),
                related: self.related.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyRunIterator {
    document: Py<PyDocument>,
    paragraph_path: ContentPath,
    related: Option<rdocx::ContentLocation>,
    index: usize,
}

#[pymethods]
impl PyRunIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyRun>>> {
        let collection = PyRunCollection::new(
            self.document.clone_ref(py),
            self.paragraph_path.clone(),
            self.related.clone(),
        );
        let len = collection.len(py)?;
        if self.index >= len {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}
