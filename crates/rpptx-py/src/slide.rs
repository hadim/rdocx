use oxml_py_support::{ContentPath, PathSeg};
use pyo3::PyClass;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PySlice, PyTuple};
use smallvec::smallvec;

use crate::dml::{FillTarget, PyFillFormat};
use crate::normalize_index;
use crate::presentation::{PyComment, PyPresentation};
use crate::replacement_count_to_pyerr;
use crate::shape::{PyPlaceholderCollection, PyShapeCollection};
use crate::validate_path;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PySlideMaster>()?;
    module.add_class::<PySlideMasterCollection>()?;
    module.add_class::<PyTheme>()?;
    module.add_class::<PyThemeFonts>()?;
    module.add_class::<PyThemeFontSet>()?;
    module.add_class::<PyHeaderFooter>()?;
    module.add_class::<PySlideTransition>()?;
    module.add_class::<PySlideLayout>()?;
    module.add_class::<PySlideLayoutCollection>()?;
    module.add_class::<PySlide>()?;
    module.add_class::<PySlideCollection>()?;
    module.add_class::<PyBackground>()?;
    Ok(())
}

#[pyclass(name = "SlideLayout")]
pub struct PySlideLayout {
    pub(crate) presentation: Py<PyPresentation>,
    pub(crate) index: usize,
    path: ContentPath,
}

impl PySlideLayout {
    fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        let presentation = self.presentation.borrow(py);
        validate_path(
            py,
            &presentation,
            &self.path,
            "slide layout",
            &format!(".slide_layouts[{}]", self.index),
        )?;
        presentation
            .current_layout_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("slide layout owner is absent"))
    }
}

#[pymethods]
impl PySlideLayout {
    #[getter]
    fn name(&self, py: Python<'_>) -> PyResult<Option<String>> {
        let index = self.validate(py)?;
        Ok(self
            .presentation
            .borrow(py)
            .inner
            .layout_name(index)
            .map(str::to_owned))
    }

    /// Two handles are equal when they name the same layout of one presentation.
    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        other
            .extract::<PyRef<'_, PySlideLayout>>()
            .is_ok_and(|other| {
                other.presentation.is(&self.presentation)
                    && other.path.owner_generation() == self.path.owner_generation()
            })
    }
}

#[pyclass(name = "SlideLayoutCollection")]
pub struct PySlideLayoutCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PySlideLayoutCollection {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "slide layout collection",
            ".slide_layouts",
        )?;
        Ok(self.presentation.borrow(py).inner.layout_count())
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PySlideLayout>> {
        Py::new(
            py,
            PySlideLayout {
                presentation: self.presentation.clone_ref(py),
                index,
                path: self.presentation.borrow(py).capture_layout(index)?,
            },
        )
    }
}

#[pymethods]
impl PySlideLayoutCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        sequence_item(py, key, self.len(py)?, "slide layout", |index| {
            self.item(py, index)
        })
    }

    /// Returns the zero-based index of a layout of this presentation.
    fn index(&self, py: Python<'_>, slide_layout: &Bound<'_, PyAny>) -> PyResult<usize> {
        self.len(py)?;
        let layout = slide_layout.extract::<PyRef<'_, PySlideLayout>>()?;
        if !layout.presentation.is(&self.presentation) {
            return Err(PyValueError::new_err(
                "layout not in this SlideLayouts collection",
            ));
        }
        layout.validate(py)
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PySlideLayoutIterator>> {
        self.len(py)?;
        Py::new(
            py,
            PySlideLayoutIterator {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PySlideLayoutIterator {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    index: usize,
}

#[pymethods]
impl PySlideLayoutIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PySlideLayout>>> {
        let collection =
            PySlideLayoutCollection::new(self.presentation.clone_ref(py), self.path.clone());
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Slide")]
pub struct PySlide {
    pub(crate) presentation: Py<PyPresentation>,
    pub(crate) path: ContentPath,
}

impl PySlide {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    pub(crate) fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        let presentation = self.presentation.borrow(py);
        validate_path(py, &presentation, &self.path, "slide", "")?;
        presentation
            .current_path(&self.path)
            .ok_or_else(|| PyIndexError::new_err("owner is absent"))?
            .segs
            .iter()
            .find_map(|segment| match segment {
                PathSeg::Slide(index) => Some(*index),
                _ => None,
            })
            .ok_or_else(|| PyIndexError::new_err("slide index is missing"))
    }
}

#[pymethods]
impl PySlide {
    #[getter]
    fn transition(&self, py: Python<'_>) -> PyResult<PySlideTransition> {
        self.validate(py)?;
        Ok(PySlideTransition {
            presentation: self.presentation.clone_ref(py),
            path: self.path.clone(),
        })
    }
    #[getter]
    fn header_footer(&self, py: Python<'_>) -> PyResult<Py<PyHeaderFooter>> {
        self.validate(py)?;
        Py::new(
            py,
            PyHeaderFooter {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
            },
        )
    }

    /// Two current handles are equal when they name the same slide of one
    /// presentation. Like python-pptx `Slide`, a handle is not hashable.
    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        other.extract::<PyRef<'_, PySlide>>().is_ok_and(|other| {
            other.presentation.is(&self.presentation)
                && other.path.owner_generation() == self.path.owner_generation()
        })
    }

    #[getter]
    fn shapes(&self, py: Python<'_>) -> PyResult<Py<PyShapeCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyShapeCollection::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }

    #[getter]
    fn placeholders(&self, py: Python<'_>) -> PyResult<Py<PyPlaceholderCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyPlaceholderCollection::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }

    #[getter]
    fn notes_text(&self, py: Python<'_>) -> PyResult<Option<String>> {
        let index = self.validate(py)?;
        Ok(self
            .presentation
            .borrow(py)
            .inner
            .slide(index)
            .and_then(|slide| slide.notes_text()))
    }

    /// Replaces the speaker notes, creating the notes slide when absent.
    #[setter]
    fn set_notes_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        let index = self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .set_notes_text(index, text)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.text_changed_owners(&self.path)?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "Slide.notes_text",
            &survivors,
            false,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }

    /// Replaces literal text on this slide, and in its speaker notes when
    /// `notes` is true, and returns the count.
    ///
    /// The contract is `Presentation.try_replace_text` restricted to this
    /// slide: with `expect`, a count that differs raises and leaves the
    /// presentation and its revision as they were, and the revision advances
    /// once only when something was replaced. The native call works on a
    /// copy of this slide alone.
    #[pyo3(signature = (placeholder, replacement, *, expect = None, notes = true))]
    fn try_replace_text(
        &self,
        py: Python<'_>,
        placeholder: &str,
        replacement: &str,
        expect: Option<usize>,
        notes: bool,
    ) -> PyResult<usize> {
        let index = self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        let inner = &mut candidate;
        let count = py
            .detach(|| inner.try_replace_slide_text(index, placeholder, replacement, notes, expect))
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        if let Some(expected) = expect.filter(|&expected| expected != count) {
            return Err(replacement_count_to_pyerr(py, placeholder, expected, count));
        }
        if count > 0 {
            let survivors = presentation.replaced_text_owners(&candidate)?;
            let prepared = presentation.prepare_owners(
                &candidate,
                "Slide.try_replace_text",
                &survivors,
                false,
                false,
                false,
            )?;
            presentation.publish_owners(candidate, prepared)?;
        }
        Ok(count)
    }

    #[getter]
    fn slide_layout(&self, py: Python<'_>) -> PyResult<Py<PySlideLayout>> {
        let index = self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        let layout = presentation
            .inner
            .slide_layout_index(index)
            .ok_or_else(|| {
                crate::rpptx_value_to_pyerr(
                    py,
                    format!("slide {index} has no layout reachable through the slide masters"),
                )
            })?;
        let path = presentation.capture_layout(layout)?;
        drop(presentation);
        Py::new(
            py,
            PySlideLayout {
                presentation: self.presentation.clone_ref(py),
                index: layout,
                path,
            },
        )
    }

    /// Moves the slide to another layout of this presentation.
    ///
    /// Placeholders the new layout does not place keep the geometry they
    /// inherited, and the revision advances once.
    #[setter]
    fn set_slide_layout(&self, py: Python<'_>, layout: &Bound<'_, PyAny>) -> PyResult<()> {
        let index = self.validate(py)?;
        let layout = layout.extract::<PyRef<'_, PySlideLayout>>()?;
        if !layout.presentation.is(&self.presentation) {
            return Err(PyValueError::new_err(
                "slide layout is not in this presentation",
            ));
        }
        let layout_index = layout.validate(py)?;
        if self.presentation.borrow(py).inner.slide_layout_index(index) == Some(layout_index) {
            return Ok(());
        }
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .set_slide_layout(index, layout_index)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.unchanged_owners()?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "Slide.slide_layout",
            &survivors,
            false,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }

    #[getter]
    fn hidden(&self, py: Python<'_>) -> PyResult<bool> {
        let index = self.validate(py)?;
        Ok(self
            .presentation
            .borrow(py)
            .inner
            .slide(index)
            .is_some_and(|slide| slide.hidden()))
    }

    #[setter]
    fn set_hidden(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let index = self.validate(py)?;
        self.presentation
            .borrow_mut(py)
            .inner
            .slide_mut(index)
            .ok_or_else(|| PyIndexError::new_err(format!("slide index {index} is out of range")))?
            .set_hidden(value);
        Ok(())
    }

    #[getter]
    fn background(&self, py: Python<'_>) -> PyResult<Py<PyBackground>> {
        self.validate(py)?;
        Py::new(
            py,
            PyBackground {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
            },
        )
    }

    /// Whether the slide inherits its background from its layout and master.
    #[getter]
    fn follow_master_background(&self, py: Python<'_>) -> PyResult<bool> {
        let index = self.validate(py)?;
        Ok(!self
            .presentation
            .borrow(py)
            .inner
            .slide(index)
            .is_some_and(|slide| slide.has_explicit_background()))
    }

    #[setter]
    fn set_follow_master_background(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let index = self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let explicit = presentation
            .inner
            .slide(index)
            .is_some_and(|slide| slide.has_explicit_background());
        let mut slide = presentation
            .inner
            .slide_mut(index)
            .ok_or_else(|| PyIndexError::new_err(format!("slide index {index} is out of range")))?;
        if value {
            slide.remove_background();
        } else if !explicit {
            slide
                .set_background(rpptx::Fill::NoFill(rpptx::NoFill::default()))
                .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        }
        Ok(())
    }

    #[getter]
    fn comments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let index = self.validate(py)?;
        let presentation = self.presentation.borrow(py);
        PyTuple::new(
            py,
            presentation
                .inner
                .comments(index)
                .unwrap_or_default()
                .iter()
                .map(PyComment::from),
        )
    }

    // PyO3 exposes these as named Python arguments, so a Rust options wrapper
    // would only hide the public signature from this boundary.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (*, id, author_id, created, text, shape_id=None, text_start=None, text_length=None))]
    fn add_comment(
        &self,
        id: String,
        author_id: String,
        created: String,
        text: &str,
        shape_id: Option<u32>,
        text_start: Option<usize>,
        text_length: Option<usize>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let index = self.validate(py)?;
        let comment = rpptx::Comment::new(id, author_id, created, text)
            .map_err(|error| crate::rpptx_value_to_pyerr(py, error.to_string()))?;
        if text_start.is_some() != text_length.is_some()
            || (text_start.is_some() && shape_id.is_none())
        {
            return Err(PyValueError::new_err(
                "text_start and text_length require each other and shape_id",
            ));
        }
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        match shape_id {
            Some(shape_id) => candidate.add_comment_at_shape(
                index,
                comment,
                shape_id,
                text_start.zip(text_length),
            ),
            None => candidate.add_comment(index, comment),
        }
        .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.unchanged_owners()?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "Slide.add_comment",
            &survivors,
            false,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }

    #[pyo3(signature = (comment_id, *, id, author_id, created, text))]
    fn reply_to_comment(
        &self,
        comment_id: &str,
        id: String,
        author_id: String,
        created: String,
        text: &str,
        py: Python<'_>,
    ) -> PyResult<()> {
        let index = self.validate(py)?;
        let reply = rpptx::CommentReply::new(id, author_id, created, text)
            .map_err(|error| crate::rpptx_value_to_pyerr(py, error.to_string()))?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .reply_to_comment(index, comment_id, reply)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.unchanged_owners()?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "Slide.reply_to_comment",
            &survivors,
            false,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }

    /// Marks one comment thread resolved. A reply id is an unknown id.
    fn resolve_comment(&self, comment_id: &str, py: Python<'_>) -> PyResult<()> {
        let index = self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .resolve_comment(index, comment_id)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.unchanged_owners()?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "Slide.resolve_comment",
            &survivors,
            false,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }

    /// Removes one comment thread with its replies, or one reply.
    fn remove_comment(&self, comment_id: &str, py: Python<'_>) -> PyResult<()> {
        let index = self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .remove_comment(index, comment_id)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.unchanged_owners()?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "Slide.remove_comment",
            &survivors,
            false,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }

    fn move_comment(&self, from_: usize, to: usize, py: Python<'_>) -> PyResult<()> {
        let index = self.validate(py)?;
        if from_ == to {
            return Ok(());
        }
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .move_comment(index, from_, to)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.unchanged_owners()?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "Slide.move_comment",
            &survivors,
            false,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }

    fn move_reply(
        &self,
        comment_id: &str,
        from_: usize,
        to: usize,
        py: Python<'_>,
    ) -> PyResult<()> {
        let index = self.validate(py)?;
        if from_ == to {
            return Ok(());
        }
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .move_reply(index, comment_id, from_, to)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.unchanged_owners()?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "Slide.move_reply",
            &survivors,
            false,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }
}

#[pyclass(name = "SlideCollection")]
pub struct PySlideCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PySlideCollection {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "slide collection",
            ".slides",
        )?;
        Ok(self.presentation.borrow(py).inner.len())
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PySlide>> {
        let path = self
            .presentation
            .borrow(py)
            .capture_path(smallvec![PathSeg::Slide(index)], None)?;
        Py::new(py, PySlide::new(self.presentation.clone_ref(py), path))
    }
}

#[pymethods]
impl PySlideCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        sequence_item(py, key, self.len(py)?, "slide", |index| {
            self.item(py, index)
        })
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PySlideIterator>> {
        self.len(py)?;
        Py::new(
            py,
            PySlideIterator {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                index: 0,
            },
        )
    }

    fn add_slide(&self, py: Python<'_>, layout: &Bound<'_, PyAny>) -> PyResult<Py<PySlide>> {
        let layout = layout.extract::<PyRef<'_, PySlideLayout>>()?;
        self.len(py)?;
        if !layout.presentation.is(&self.presentation) {
            return Err(PyValueError::new_err(
                "layout belongs to a different presentation",
            ));
        }
        let layout_index = layout.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let index = presentation.inner.len();
        let mut candidate = presentation.inner.clone();
        candidate
            .add_slide(layout_index)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation.unchanged_owners()?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "SlideCollection.add_slide",
            &survivors,
            true,
            false,
            false,
        )?;
        let path = presentation.capture_prepared_path(
            &prepared,
            smallvec![PathSeg::Slide(index)],
            None,
        )?;
        let result = Py::new(py, PySlide::new(self.presentation.clone_ref(py), path))?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(result)
    }

    /// Imports a slide of another presentation, using this presentation's theme.
    ///
    /// The copy takes `layout` when given. Otherwise a slide of this
    /// presentation keeps its own layout, and a slide of another presentation
    /// takes this presentation's first layout named like its layout. It is
    /// inserted before `index` as by `list.insert`, counted from the end when
    /// negative, or appended when that is `None`. An index outside the
    /// collection raises `IndexError`.
    #[pyo3(signature = (slide, layout=None, index=None))]
    fn import_slide(
        &self,
        py: Python<'_>,
        slide: &Bound<'_, PyAny>,
        layout: Option<&Bound<'_, PyAny>>,
        index: Option<isize>,
    ) -> PyResult<Py<PySlide>> {
        let len = self.len(py)?;
        let slide = slide.extract::<PyRef<'_, PySlide>>()?;
        let source_index = slide.validate(py)?;
        let layout_index = match layout {
            Some(layout) => {
                let layout = layout.extract::<PyRef<'_, PySlideLayout>>()?;
                if !layout.presentation.is(&self.presentation) {
                    return Err(PyValueError::new_err(
                        "layout is not a layout of this presentation",
                    ));
                }
                Some(layout.validate(py)?)
            }
            None => None,
        };
        let inserted = match index {
            None => len,
            Some(index) => {
                let normalized = if index < 0 {
                    len as isize + index
                } else {
                    index
                };
                if normalized < 0 || normalized > len as isize {
                    return Err(PyIndexError::new_err("slide index out of range"));
                }
                normalized as usize
            }
        };
        let source = slide.presentation.clone_ref(py);
        drop(slide);
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        let result = if source.is(&self.presentation) {
            let layout_index =
                layout_index.or_else(|| presentation.inner.slide_layout_index(source_index));
            candidate
                .import_slide(
                    &presentation.inner,
                    source_index,
                    layout_index,
                    Some(inserted),
                )
                .map(|_| ())
        } else {
            let source = source.borrow(py);
            candidate
                .import_slide(&source.inner, source_index, layout_index, Some(inserted))
                .map(|_| ())
        };
        result.map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors =
            presentation.slide_owner_edit(crate::presentation::SlideOwnerEdit::Insert(inserted))?;
        // Native import targets an existing destination layout and never edits
        // its layout or slide-master inventories.
        let prepared = presentation.prepare_owners(
            &candidate,
            "SlideCollection.import_slide",
            &survivors,
            true,
            false,
            false,
        )?;
        let path = presentation.capture_prepared_path(
            &prepared,
            smallvec![PathSeg::Slide(inserted)],
            None,
        )?;
        let result = Py::new(py, PySlide::new(self.presentation.clone_ref(py), path))?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(result)
    }

    /// Removes one slide of this presentation with its notes and owned media.
    fn remove(&self, py: Python<'_>, slide: &Bound<'_, PyAny>) -> PyResult<()> {
        self.len(py)?;
        let slide = slide.extract::<PyRef<'_, PySlide>>()?;
        if !slide.presentation.is(&self.presentation) {
            return Err(PyValueError::new_err("slide is not in this collection"));
        }
        let index = slide.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .remove_slide(index)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors =
            presentation.slide_owner_edit(crate::presentation::SlideOwnerEdit::Remove(index))?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "SlideCollection.remove",
            &survivors,
            true,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }

    /// Duplicates one slide of this presentation with its notes right after
    /// the source, and returns the new slide.
    fn duplicate(&self, py: Python<'_>, slide: &Bound<'_, PyAny>) -> PyResult<Py<PySlide>> {
        self.len(py)?;
        let slide = slide.extract::<PyRef<'_, PySlide>>()?;
        if !slide.presentation.is(&self.presentation) {
            return Err(PyValueError::new_err("slide is not in this collection"));
        }
        let index = slide.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .duplicate_slide(index)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation
            .slide_owner_edit(crate::presentation::SlideOwnerEdit::Insert(index + 1))?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "SlideCollection.duplicate",
            &survivors,
            true,
            false,
            false,
        )?;
        let path = presentation.capture_prepared_path(
            &prepared,
            smallvec![PathSeg::Slide(index + 1)],
            None,
        )?;
        let result = Py::new(py, PySlide::new(self.presentation.clone_ref(py), path))?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(result)
    }

    /// Moves the slide at `from_` so that it ends up at index `to`.
    #[pyo3(name = "move")]
    fn move_slide(&self, py: Python<'_>, from_: isize, to: isize) -> PyResult<()> {
        let len = self.len(py)?;
        let from_ = normalize_index(from_, len, "slide")?;
        let to = normalize_index(to, len, "slide")?;
        if from_ == to {
            return Ok(());
        }
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        candidate
            .move_slide(from_, to)
            .map_err(|error| crate::rpptx_to_pyerr(py, error))?;
        let survivors = presentation
            .slide_owner_edit(crate::presentation::SlideOwnerEdit::Move { from: from_, to })?;
        let prepared = presentation.prepare_owners(
            &candidate,
            "SlideCollection.move",
            &survivors,
            true,
            false,
            false,
        )?;
        presentation.publish_owners(candidate, prepared)?;
        Ok(())
    }
}

/// The background of one slide, like python-pptx `_Background`.
#[pyclass(name = "Background")]
pub struct PyBackground {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

#[pymethods]
impl PyBackground {
    /// The slide's own background fill. Reading it never changes the slide.
    #[getter]
    fn fill(&self, py: Python<'_>) -> PyResult<Py<PyFillFormat>> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "background",
            ".background",
        )?;
        Py::new(
            py,
            PyFillFormat::new(
                self.presentation.clone_ref(py),
                self.path.clone(),
                FillTarget::Background,
            ),
        )
    }
}

#[pyclass]
struct PySlideIterator {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    index: usize,
}

#[pymethods]
impl PySlideIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PySlide>>> {
        let collection = PySlideCollection::new(self.presentation.clone_ref(py), self.path.clone());
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

fn sequence_item<T, F>(
    py: Python<'_>,
    key: &Bound<'_, PyAny>,
    len: usize,
    kind: &str,
    mut item: F,
) -> PyResult<Py<PyAny>>
where
    T: PyClass,
    F: FnMut(usize) -> PyResult<Py<T>>,
{
    if let Ok(index) = key.extract::<isize>() {
        return Ok(item(normalize_index(index, len, kind)?)?.into_any());
    }
    if key.is_instance_of::<PySlice>() {
        let (start, stop, step): (isize, isize, isize) =
            key.call_method1("indices", (len,))?.extract()?;
        let items = PyList::empty(py);
        let mut index = start;
        while if step > 0 { index < stop } else { index > stop } {
            items.append(item(index as usize)?)?;
            index += step;
        }
        return Ok(items.into_any().unbind());
    }
    Err(PyTypeError::new_err(format!(
        "{kind} indices must be integers or slices"
    )))
}

#[pyclass(name = "SlideMaster")]
pub struct PySlideMaster {
    presentation: Py<PyPresentation>,
    part_name: String,
    path: ContentPath,
}

impl PySlideMaster {
    fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        let p = self.presentation.borrow(py);
        validate_path(py, &p, &self.path, "slide master", ".slide_masters")?;
        for index in 0..p.inner.slide_master_count() {
            if p.inner
                .slide_master_part_name(index)
                .map_err(|e| crate::rpptx_to_pyerr(py, e))?
                .as_deref()
                == Some(self.part_name.as_str())
            {
                return Ok(index);
            }
        }
        Err(PyIndexError::new_err("slide master owner is absent"))
    }
}

#[pymethods]
impl PySlideMaster {
    /// The theme this master uses, read when accessed.
    #[getter]
    fn theme(&self, py: Python<'_>) -> PyResult<PyTheme> {
        self.validate(py)?;
        Ok(PyTheme {
            presentation: self.presentation.clone_ref(py),
            part_name: self.part_name.clone(),
            path: self.path.clone(),
        })
    }

    /// The layouts this master owns, as `prs.slide_layouts` members.
    #[getter]
    fn slide_layouts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let index = self.validate(py)?;
        let layouts = self
            .presentation
            .borrow(py)
            .inner
            .master_layouts(index)
            .ok_or_else(|| PyIndexError::new_err("slide master index out of range"))?;
        let collection =
            PySlideLayoutCollection::new(self.presentation.clone_ref(py), self.path.clone());
        let items = layouts
            .into_iter()
            .map(|index| collection.item(py, index))
            .collect::<PyResult<Vec<_>>>()?;
        PyTuple::new(py, items)
    }

    /// Two handles are equal when they name the same master of one presentation.
    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        other
            .extract::<PyRef<'_, PySlideMaster>>()
            .is_ok_and(|other| {
                other.presentation.is(&self.presentation) && other.part_name == self.part_name
            })
    }
}

#[pyclass(name = "SlideMasterCollection")]
pub struct PySlideMasterCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PySlideMasterCollection {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "slide master collection",
            ".slide_masters",
        )?;
        Ok(self.presentation.borrow(py).inner.slide_master_count())
    }

    pub(crate) fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PySlideMaster>> {
        Py::new(
            py,
            PySlideMaster {
                presentation: self.presentation.clone_ref(py),
                part_name: self
                    .presentation
                    .borrow(py)
                    .inner
                    .slide_master_part_name(index)
                    .map_err(|e| crate::rpptx_to_pyerr(py, e))?
                    .ok_or_else(|| PyIndexError::new_err("slide master index out of range"))?,
                path: self.path.clone(),
            },
        )
    }
}

#[pymethods]
impl PySlideMasterCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        sequence_item(py, key, self.len(py)?, "slide master", |index| {
            self.item(py, index)
        })
    }

    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let items = (0..self.len(py)?)
            .map(|index| self.item(py, index))
            .collect::<PyResult<Vec<_>>>()?;
        PyTuple::new(py, items)?
            .into_any()
            .try_iter()
            .map(Bound::into_any)
    }
}

/// A read-only view that resolves its original master part on every access.
#[pyclass(name = "Theme")]
pub struct PyTheme {
    presentation: Py<PyPresentation>,
    part_name: String,
    path: ContentPath,
}

impl PyTheme {
    fn current(&self, py: Python<'_>) -> PyResult<rpptx::Theme> {
        let master = PySlideMaster {
            presentation: self.presentation.clone_ref(py),
            part_name: self.part_name.clone(),
            path: self.path.clone(),
        };
        let index = master.validate(py)?;
        self.presentation
            .borrow(py)
            .inner
            .theme(index)
            .map_err(|e| crate::rpptx_to_pyerr(py, e))?
            .ok_or_else(|| PyIndexError::new_err("slide master owner is absent"))
    }
}

#[pymethods]
impl PyTheme {
    #[getter]
    fn name(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.current(py)?.name)
    }

    #[getter]
    fn colors<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let rgb = py.import("rpptx.dml.color")?.getattr("RGBColor")?;
        let colors = PyDict::new(py);
        for (slot, color) in self.current(py)?.colors {
            let value = match color {
                Some(color) => {
                    let [r, g, b] = color.components();
                    rgb.call1((r, g, b))?.unbind()
                }
                None => py.None(),
            };
            colors.set_item(slot, value)?;
        }
        Ok(colors)
    }

    #[getter]
    fn fonts(&self, py: Python<'_>) -> PyResult<PyThemeFonts> {
        let current = self.current(py)?;
        let fonts = |f: rpptx::ThemeFonts| PyThemeFontSet {
            latin: f.latin,
            east_asian: f.east_asian,
            complex_script: f.complex_script,
        };
        Ok(PyThemeFonts {
            major: fonts(current.major_font),
            minor: fonts(current.minor_font),
        })
    }
}

/// The theme's heading (`major`) and body (`minor`) fonts.
#[pyclass(name = "ThemeFonts", frozen, get_all, skip_from_py_object)]
#[derive(Clone)]
pub struct PyThemeFonts {
    major: PyThemeFontSet,
    minor: PyThemeFontSet,
}

/// One theme font collection's typefaces, empty when the theme names none.
#[pyclass(name = "ThemeFontSet", frozen, get_all, skip_from_py_object)]
#[derive(Clone)]
pub struct PyThemeFontSet {
    latin: String,
    east_asian: String,
    complex_script: String,
}

pub(crate) fn header_footer_settings(
    slide_number: bool,
    footer: Option<String>,
    date: Option<String>,
    date_format: &str,
    date_cache: Option<String>,
) -> rpptx::HeaderFooter {
    let date = match date.as_deref() {
        None => rpptx::HeaderFooterDate::Off,
        Some("auto") => rpptx::HeaderFooterDate::Automatic {
            field_type: date_format.into(),
            text: date_cache,
        },
        Some(text) => rpptx::HeaderFooterDate::Fixed(text.into()),
    };
    rpptx::HeaderFooter {
        slide_number,
        footer,
        date,
    }
}

#[pyclass(name = "HeaderFooter")]
pub struct PyHeaderFooter {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyHeaderFooter {
    fn slide_index(&self, py: Python<'_>) -> PyResult<usize> {
        PySlide::new(self.presentation.clone_ref(py), self.path.clone()).validate(py)
    }

    fn current(&self, py: Python<'_>) -> PyResult<(usize, rpptx::HeaderFooter)> {
        let index = self.slide_index(py)?;
        let settings = self
            .presentation
            .borrow(py)
            .inner
            .slide_header_footer(index)
            .ok_or_else(|| PyIndexError::new_err("slide index out of range"))?;
        Ok((index, settings))
    }

    /// Patch the latest owning slide and publish all causal scope changes once.
    fn update(
        &mut self,
        py: Python<'_>,
        change: impl FnOnce(&mut rpptx::HeaderFooter),
    ) -> PyResult<()> {
        let (index, mut settings) = self.current(py)?;
        change(&mut settings);
        let mut p = self.presentation.borrow_mut(py);
        let mut candidate = p.inner.clone();
        if candidate
            .set_slide_header_footer(index, &settings)
            .map_err(|e| crate::rpptx_to_pyerr(py, e))?
        {
            let proofs = p.footer_owners(&candidate)?;
            let prepared = p.prepare_owners(
                &candidate,
                "Slide.header_footer",
                &proofs,
                false,
                false,
                false,
            )?;
            p.publish_owners(candidate, prepared)?;
        }
        Ok(())
    }
}

#[pymethods]
impl PyHeaderFooter {
    #[getter]
    fn date_cache(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(match self.current(py)?.1.date {
            rpptx::HeaderFooterDate::Automatic { text, .. } => text,
            _ => None,
        })
    }

    #[getter]
    fn date_mode(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(match self.current(py)?.1.date {
            rpptx::HeaderFooterDate::Off => "off",
            rpptx::HeaderFooterDate::Fixed(_) => "fixed",
            rpptx::HeaderFooterDate::Automatic { .. } => "automatic",
        })
    }

    #[getter]
    fn source(&self, py: Python<'_>) -> PyResult<&'static str> {
        let current = self.current(py)?.1;
        if current.slide_number
            || current.footer.is_some()
            || !matches!(current.date, rpptx::HeaderFooterDate::Off)
        {
            return Ok("slide");
        }
        self.policy_source(py)
    }

    /// Legal inherited policy is independent of materialized slide-owned content.
    #[getter]
    fn policy_source(&self, py: Python<'_>) -> PyResult<&'static str> {
        let index = self.slide_index(py)?;
        Ok(self
            .presentation
            .borrow(py)
            .inner
            .slide_header_footer_policy(index)
            .map_err(|e| crate::rpptx_to_pyerr(py, e))?
            .map_or("absent", |(source, _)| source))
    }

    #[getter]
    fn slide_number_policy(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        let index = self.slide_index(py)?;
        Ok(self
            .presentation
            .borrow(py)
            .inner
            .slide_header_footer_policy(index)
            .map_err(|e| crate::rpptx_to_pyerr(py, e))?
            .and_then(|(_, flags)| flags.slide_number))
    }

    #[getter]
    fn footer_policy(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        let index = self.slide_index(py)?;
        Ok(self
            .presentation
            .borrow(py)
            .inner
            .slide_header_footer_policy(index)
            .map_err(|e| crate::rpptx_to_pyerr(py, e))?
            .and_then(|(_, flags)| flags.footer))
    }

    #[getter]
    fn date_policy(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        let index = self.slide_index(py)?;
        Ok(self
            .presentation
            .borrow(py)
            .inner
            .slide_header_footer_policy(index)
            .map_err(|e| crate::rpptx_to_pyerr(py, e))?
            .and_then(|(_, flags)| flags.date))
    }

    #[getter]
    fn footer_present(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.current(py)?.1.footer.is_some())
    }

    #[getter]
    fn date_present(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(!matches!(
            self.current(py)?.1.date,
            rpptx::HeaderFooterDate::Off
        ))
    }

    #[getter]
    fn slide_number_present(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.current(py)?.1.slide_number)
    }

    /// Whether the slide owns a slide-number placeholder.
    #[getter]
    fn slide_number(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.current(py)?.1.slide_number)
    }

    #[setter]
    fn set_slide_number(&mut self, py: Python<'_>, value: bool) -> PyResult<()> {
        self.update(py, |settings| settings.slide_number = value)
    }

    /// The footer text, `None` without a footer placeholder.
    #[getter]
    fn footer(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.current(py)?.1.footer)
    }

    #[setter]
    fn set_footer(&mut self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.update(py, |settings| settings.footer = value)
    }

    /// `None`, the fixed date text, or `"auto"` for a date field.
    #[getter]
    fn date(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(match self.current(py)?.1.date {
            rpptx::HeaderFooterDate::Off => None,
            rpptx::HeaderFooterDate::Fixed(text) => Some(text),
            rpptx::HeaderFooterDate::Automatic { .. } => Some("auto".to_owned()),
        })
    }

    /// Sets `None`, fixed text, or `"auto"` for a field PowerPoint refreshes,
    /// retaining a compatible cache in the current `date_format`, else `datetime1`.
    #[setter]
    fn set_date(&mut self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        let date_format = self
            .date_format(py)?
            .unwrap_or_else(|| "datetime1".to_owned());
        let date = header_footer_settings(false, None, value, &date_format, None).date;
        self.update(py, |settings| settings.date = date)
    }

    /// The date field type, such as `datetime1`, `None` for a fixed or no date.
    #[getter]
    fn date_format(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(match self.current(py)?.1.date {
            rpptx::HeaderFooterDate::Automatic { field_type, .. } => Some(field_type),
            _ => None,
        })
    }

    #[setter]
    fn set_date_format(&mut self, py: Python<'_>, value: &str) -> PyResult<()> {
        if self.date_format(py)?.is_none() {
            return Err(PyValueError::new_err(
                "the slide has no date field, set header_footer.date = \"auto\" first",
            ));
        }
        let date = header_footer_settings(false, None, Some("auto".to_owned()), value, None).date;
        self.update(py, |settings| settings.date = date)
    }
}

/// One slide's transition, read and written live (rpptx extension).
///
/// `type` is one of `fade`, `push`, `wipe`, `split`, `cover`, `uncover`,
/// `cut`, `zoom`, or `None`. Another producer's effect reads as its element
/// name and cannot be assigned. Durations are in seconds.
#[pyclass(name = "SlideTransition")]
pub struct PySlideTransition {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PySlideTransition {
    fn current(&self, py: Python<'_>) -> PyResult<(usize, rpptx::SlideTransition)> {
        let index =
            PySlide::new(self.presentation.clone_ref(py), self.path.clone()).validate(py)?;
        let transition = self
            .presentation
            .borrow(py)
            .inner
            .slide(index)
            .ok_or_else(|| PyIndexError::new_err("slide index out of range"))?
            .transition()
            .unwrap_or_default();
        Ok((index, transition))
    }

    fn update(
        &self,
        py: Python<'_>,
        change: impl FnOnce(&mut rpptx::SlideTransition) -> PyResult<()>,
    ) -> PyResult<()> {
        let (index, mut transition) = self.current(py)?;
        change(&mut transition)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        if candidate
            .set_slide_transition(index, Some(&transition))
            .map_err(|e| crate::rpptx_to_pyerr(py, e))?
        {
            let proofs = presentation.footer_owners(&candidate)?;
            let prepared = presentation.prepare_owners(
                &candidate,
                "Slide.transition",
                &proofs,
                false,
                false,
                false,
            )?;
            presentation.publish_owners(candidate, prepared)?;
        }
        Ok(())
    }
}

fn seconds_to_ms(name: &str, value: Option<f64>) -> PyResult<Option<u64>> {
    value
        .map(|seconds| {
            if seconds.is_finite() && seconds >= 0.0 && seconds * 1000.0 < u64::MAX as f64 {
                Ok((seconds * 1000.0).trunc() as u64)
            } else {
                Err(PyValueError::new_err(format!(
                    "{name} must be finite, nonnegative and fit milliseconds, got {seconds}"
                )))
            }
        })
        .transpose()
}

#[pymethods]
impl PySlideTransition {
    #[getter]
    fn r#type(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.current(py)?.1.kind.map(|kind| kind.name().to_owned()))
    }

    /// Sets the effect, keeping the direction when the new effect takes it.
    #[setter]
    fn set_type(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        let kind = value
            .map(|name| {
                rpptx::TransitionKind::parse(&name).ok_or_else(|| {
                    PyValueError::new_err(format!(
                        "unknown transition type {name:?}, use one of {}",
                        rpptx::TransitionKind::NAMES.join(", ")
                    ))
                })
            })
            .transpose()?;
        self.update(py, |transition| {
            let keeps_direction = transition.direction.is_some_and(|direction| {
                kind.as_ref()
                    .is_some_and(|kind| kind.directions().contains(&direction))
            });
            if !keeps_direction {
                transition.direction = None;
            }
            if kind.is_none() {
                *transition = rpptx::SlideTransition::default();
            }
            transition.kind = kind;
            Ok(())
        })
    }

    /// The direction the incoming slide moves, such as `left` (`dir="l"`,
    /// PowerPoint's "From Right"), `None` for the effect's default.
    #[getter]
    fn direction(&self, py: Python<'_>) -> PyResult<Option<&'static str>> {
        Ok(self
            .current(py)?
            .1
            .direction
            .map(rpptx::TransitionDirection::name))
    }

    #[setter]
    fn set_direction(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        let direction = value
            .map(|name| {
                rpptx::TransitionDirection::parse(&name).ok_or_else(|| {
                    PyValueError::new_err(format!(
                        "unknown transition direction {name:?}, use one of {}",
                        rpptx::TransitionDirection::NAMES.join(", ")
                    ))
                })
            })
            .transpose()?;
        self.update(py, |transition| {
            transition.direction = direction;
            Ok(())
        })
    }

    /// The effect duration in seconds, `None` for the default speed.
    #[getter]
    fn duration(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        Ok(self.current(py)?.1.duration_ms.map(|ms| ms as f64 / 1000.0))
    }

    #[setter]
    fn set_duration(&self, py: Python<'_>, value: Option<f64>) -> PyResult<()> {
        let duration = seconds_to_ms("duration", value)?;
        self.update(py, |transition| {
            transition.duration_ms = duration;
            Ok(())
        })
    }

    /// Whether a click advances the slide show, true by default.
    #[getter]
    fn advance_on_click(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        Ok(self.current(py)?.1.advance_on_click)
    }

    #[setter]
    fn set_advance_on_click(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.update(py, |transition| {
            transition.advance_on_click = value;
            Ok(())
        })
    }

    /// Seconds after which the slide show advances by itself, or `None`.
    #[getter]
    fn advance_after(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        Ok(self
            .current(py)?
            .1
            .advance_after_ms
            .map(|ms| ms as f64 / 1000.0))
    }

    #[setter]
    fn set_advance_after(&self, py: Python<'_>, value: Option<f64>) -> PyResult<()> {
        let after = seconds_to_ms("advance_after", value)?;
        self.update(py, |transition| {
            transition.advance_after_ms = after;
            Ok(())
        })
    }

    /// Gives every slide this slide's transition, as PowerPoint's Apply To All.
    fn apply_to_all(&self, py: Python<'_>) -> PyResult<()> {
        let (_, transition) = self.current(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut candidate = presentation.inner.clone();
        if candidate
            .set_all_transitions(Some(&transition))
            .map_err(|e| crate::rpptx_to_pyerr(py, e))?
        {
            let proofs = presentation.footer_owners(&candidate)?;
            let prepared = presentation.prepare_owners(
                &candidate,
                "SlideTransition.apply_to_all",
                &proofs,
                false,
                false,
                false,
            )?;
            presentation.publish_owners(candidate, prepared)?;
        }
        Ok(())
    }
}
