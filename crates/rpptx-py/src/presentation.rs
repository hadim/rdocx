use std::path::PathBuf;

use oxml_py_support::RevisionCounter;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyList, PyString, PyTuple};
use smallvec::smallvec;

use crate::layout::PyTextFrameLayout;
use crate::shape::length;
use crate::slide::{PySlideCollection, PySlideLayoutCollection};
use crate::{replacement_count_to_pyerr, rpptx_to_pyerr, rpptx_value_to_pyerr};

/// The bundled 16:9 slide size, paired with the first dimension set on a deck
/// that has no `p:sldSz`.
const DEFAULT_SLIDE_SIZE: (i64, i64) = (12_192_000, 6_858_000);

#[pyclass(name = "CommentAuthor", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyCommentAuthor {
    pub id: String,
    pub name: String,
    pub initials: Option<String>,
    pub user_id: String,
    pub provider_id: String,
}

impl From<&rpptx::CommentAuthor> for PyCommentAuthor {
    fn from(author: &rpptx::CommentAuthor) -> Self {
        Self {
            id: author.id.clone(),
            name: author.name.clone(),
            initials: author.initials.clone(),
            user_id: author.user_id.clone(),
            provider_id: author.provider_id.clone(),
        }
    }
}

#[pyclass(name = "CommentReply", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyCommentReply {
    pub id: String,
    pub author_id: String,
    pub status: Option<String>,
    pub created: String,
    pub text: String,
}

impl From<&rpptx::CommentReply> for PyCommentReply {
    fn from(reply: &rpptx::CommentReply) -> Self {
        Self {
            id: reply.id.clone(),
            author_id: reply.author_id.clone(),
            status: reply.status.clone(),
            created: reply.created.clone(),
            text: reply.text(),
        }
    }
}

#[pyclass(name = "Comment", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyComment {
    id: String,
    author_id: String,
    status: Option<String>,
    created: String,
    text: String,
    replies: Vec<PyCommentReply>,
}

impl From<&rpptx::Comment> for PyComment {
    fn from(comment: &rpptx::Comment) -> Self {
        Self {
            id: comment.id.clone(),
            author_id: comment.author_id.clone(),
            status: comment.status.clone(),
            created: comment.created.clone(),
            text: comment.text(),
            replies: comment.replies().iter().map(Into::into).collect(),
        }
    }
}

#[pymethods]
impl PyComment {
    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    #[getter]
    fn author_id(&self) -> &str {
        &self.author_id
    }

    #[getter]
    fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    #[getter]
    fn created(&self) -> &str {
        &self.created
    }

    #[getter]
    fn text(&self) -> &str {
        &self.text
    }

    #[getter]
    fn replies<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.replies.iter().cloned())
    }
}

/// One issue `Presentation::validate` reports, with the snake_case variant
/// name and the line `rpptx validate` prints for it.
#[pyclass(name = "ValidationIssue", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyValidationIssue {
    pub kind: &'static str,
    pub message: String,
}

impl From<&rpptx::ValidationIssue> for PyValidationIssue {
    fn from(issue: &rpptx::ValidationIssue) -> Self {
        use rpptx::ValidationIssue as Issue;
        let kind = match issue {
            Issue::DuplicateShapeId { .. } => "duplicate_shape_id",
            Issue::SlideIdOutOfRange { .. } => "slide_id_out_of_range",
            Issue::DuplicateSlideId { .. } => "duplicate_slide_id",
            Issue::MissingContentTypeOverride { .. } => "missing_content_type_override",
            Issue::DanglingRelationship { .. } => "dangling_relationship",
            Issue::UnreachableRelationshipTarget { .. } => "unreachable_relationship_target",
            Issue::EmptyTextBody { .. } => "empty_text_body",
            Issue::DuplicatePlaceholderIdx { .. } => "duplicate_placeholder_idx",
            Issue::OrphanMedia { .. } => "orphan_media",
            Issue::CustomShowReference { .. } => "custom_show_reference",
            Issue::MissingLayoutRel { .. } => "missing_layout_rel",
            Issue::MissingThemeRel { .. } => "missing_theme_rel",
        };
        Self {
            kind,
            message: format!("{issue:?}"),
        }
    }
}

#[pyclass(name = "Presentation")]
pub struct PyPresentation {
    pub(crate) inner: rpptx::Presentation,
    pub(crate) revisions: RevisionCounter,
    pub(crate) text_revisions: RevisionCounter,
}

impl PyPresentation {
    fn from_presentation(inner: rpptx::Presentation) -> Self {
        Self {
            inner,
            revisions: RevisionCounter::new(),
            text_revisions: RevisionCounter::new(),
        }
    }

    fn set_slide_size(
        &mut self,
        py: Python<'_>,
        width: Option<i64>,
        height: Option<i64>,
    ) -> PyResult<()> {
        let (current_width, current_height) = self
            .inner
            .slide_size()
            .map_or(DEFAULT_SLIDE_SIZE, |(width, height)| (width.0, height.0));
        self.inner
            .set_slide_size(
                rpptx::Emu(width.unwrap_or(current_width)),
                rpptx::Emu(height.unwrap_or(current_height)),
            )
            .map_err(|error| rpptx_to_pyerr(py, error))
    }
}

#[pymethods]
impl PyPresentation {
    #[new]
    #[pyo3(signature = (path = None))]
    fn new(path: Option<PathBuf>, py: Python<'_>) -> PyResult<Self> {
        match path {
            Some(path) => rpptx::Presentation::open(path)
                .map(Self::from_presentation)
                .map_err(|error| rpptx_to_pyerr(py, error)),
            None => rpptx::Presentation::new()
                .map(Self::from_presentation)
                .map_err(|error| rpptx_to_pyerr(py, error)),
        }
    }

    #[staticmethod]
    fn from_bytes(bytes: &[u8], py: Python<'_>) -> PyResult<Self> {
        rpptx::Presentation::from_bytes(bytes)
            .map(Self::from_presentation)
            .map_err(|error| rpptx_to_pyerr(py, error))
    }

    fn save(&self, path: PathBuf, py: Python<'_>) -> PyResult<()> {
        self.inner
            .save(path)
            .map_err(|error| rpptx_to_pyerr(py, error))
    }

    #[pyo3(name = "to_bytes")]
    fn serialize<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        self.inner
            .to_bytes()
            .map(|bytes| PyBytes::new(py, &bytes))
            .map_err(|error| rpptx_to_pyerr(py, error))
    }

    #[pyo3(signature = (*, fonts = None, font_dir = None, font_dirs = None))]
    fn to_pdf<'py>(
        &self,
        py: Python<'py>,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });

        py.detach(|| match &font_refs {
            None => self.inner.to_pdf_deterministic(),
            Some(fonts) => self.inner.to_pdf_deterministic_with_fonts(fonts),
        })
        .map(|bytes| PyBytes::new(py, &bytes))
        .map_err(|error| rpptx_to_pyerr(py, error))
    }

    #[pyo3(signature = (*, fonts = None, font_dir = None, font_dirs = None))]
    fn to_pdf_with_diagnostics<'py>(
        &self,
        py: Python<'py>,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyTuple>)> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });
        let font_refs = font_refs.as_deref().unwrap_or(&[]);
        let (payload, diagnostics) = py
            .detach(|| self.inner.to_pdf_with_diagnostics(font_refs))
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        let messages = PyTuple::new(py, diagnostics.iter().map(|d| d.message.as_str()))?;
        Ok((PyBytes::new(py, &payload), messages))
    }
    #[pyo3(signature = (slide_index, dpi = 150.0, *, fonts = None, font_dir = None, font_dirs = None))]
    #[allow(clippy::too_many_arguments)]
    fn render_slide_to_png<'py>(
        &self,
        py: Python<'py>,
        slide_index: usize,
        dpi: f64,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Option<Bound<'py, PyBytes>>> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });

        py.detach(|| match &font_refs {
            None => self.inner.slide_png_deterministic(slide_index, dpi),
            Some(fonts) => self
                .inner
                .slide_png_deterministic_with_fonts(slide_index, dpi, fonts),
        })
        .map(|bytes| bytes.map(|bytes| PyBytes::new(py, &bytes)))
        .map_err(|error| rpptx_to_pyerr(py, error))
    }

    #[pyo3(signature = (slide_index, dpi = 150.0, *, fonts = None, font_dir = None, font_dirs = None))]
    #[allow(clippy::too_many_arguments)]
    fn render_slide_to_png_with_diagnostics<'py>(
        &self,
        py: Python<'py>,
        slide_index: usize,
        dpi: f64,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<(Option<Bound<'py, PyBytes>>, Bound<'py, PyTuple>)> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });
        let font_refs = font_refs.as_deref().unwrap_or(&[]);
        let (payload, diagnostics) = py
            .detach(|| {
                self.inner
                    .render_slide_to_png_with_diagnostics(slide_index, dpi, font_refs)
            })
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        let messages = PyTuple::new(py, diagnostics.iter().map(|d| d.message.as_str()))?;
        Ok((payload.map(|bytes| PyBytes::new(py, &bytes)), messages))
    }
    #[pyo3(signature = (dpi = 150.0, *, fonts = None, font_dir = None, font_dirs = None))]
    fn render_all_slides<'py>(
        &self,
        py: Python<'py>,
        dpi: f64,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyList>> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });

        let slides = py
            .detach(|| match &font_refs {
                None => self.inner.slide_pngs_deterministic(dpi),
                Some(fonts) => self.inner.slide_pngs_deterministic_with_fonts(dpi, fonts),
            })
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        PyList::new(py, slides.iter().map(|slide| PyBytes::new(py, slide)))
    }

    #[pyo3(signature = (dpi = 150.0, *, fonts = None, font_dir = None, font_dirs = None))]
    fn render_all_slides_with_diagnostics<'py>(
        &self,
        py: Python<'py>,
        dpi: f64,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<(Bound<'py, PyList>, Bound<'py, PyTuple>)> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });
        let font_refs = font_refs.as_deref().unwrap_or(&[]);
        let (payload, diagnostics) = py
            .detach(|| {
                self.inner
                    .render_all_slides_with_diagnostics(dpi, font_refs)
            })
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        let messages = PyTuple::new(py, diagnostics.iter().map(|d| d.message.as_str()))?;
        Ok((
            PyList::new(py, payload.iter().map(|bytes| PyBytes::new(py, bytes)))?,
            messages,
        ))
    }
    #[pyo3(signature = (*, width_factor = 1.0, fonts = None, font_dir = None, font_dirs = None))]
    fn text_layout<'py>(
        &self,
        py: Python<'py>,
        width_factor: f64,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyTuple>> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });

        let frames = py
            .detach(|| match &font_refs {
                None => self.inner.text_layout_deterministic(width_factor),
                Some(fonts) => self
                    .inner
                    .text_layout_deterministic_with_fonts(width_factor, fonts),
            })
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        PyTuple::new(py, frames.iter().map(PyTextFrameLayout::from))
    }

    #[pyo3(signature = (*, fonts = None, font_dir = None, font_dirs = None))]
    fn to_notes_pdf<'py>(
        &self,
        py: Python<'py>,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });

        py.detach(|| match &font_refs {
            None => self.inner.to_notes_pdf_deterministic(),
            Some(fonts) => self.inner.to_notes_pdf_deterministic_with_fonts(fonts),
        })
        .map(|bytes| PyBytes::new(py, &bytes))
        .map_err(|error| rpptx_to_pyerr(py, error))
    }

    #[pyo3(signature = (dpi = 150.0, *, fonts = None, font_dir = None, font_dirs = None))]
    fn render_all_notes<'py>(
        &self,
        py: Python<'py>,
        dpi: f64,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyList>> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });

        let notes = py
            .detach(|| match &font_refs {
                None => self.inner.notes_page_pngs_deterministic(dpi),
                Some(fonts) => self
                    .inner
                    .notes_page_pngs_deterministic_with_fonts(dpi, fonts),
            })
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        PyList::new(py, notes.iter().map(|page| PyBytes::new(py, page)))
    }

    #[pyo3(signature = (*, fonts = None, font_dir = None, font_dirs = None))]
    fn to_notes_pdf_with_diagnostics<'py>(
        &self,
        py: Python<'py>,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyTuple>)> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });
        let font_refs = font_refs.as_deref().unwrap_or(&[]);
        let (payload, diagnostics) = py
            .detach(|| self.inner.to_notes_pdf_with_diagnostics(font_refs))
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        let messages = PyTuple::new(py, diagnostics.iter().map(|d| d.message.as_str()))?;
        Ok((PyBytes::new(py, &payload), messages))
    }
    #[pyo3(signature = (dpi = 150.0, *, fonts = None, font_dir = None, font_dirs = None))]
    fn render_all_notes_with_diagnostics<'py>(
        &self,
        py: Python<'py>,
        dpi: f64,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<(Bound<'py, PyList>, Bound<'py, PyTuple>)> {
        let caller_fonts = collect_caller_fonts(py, fonts, font_dir, font_dirs)?;
        let font_refs = caller_fonts.as_ref().map(|files| {
            files
                .iter()
                .map(|font| (font.family.as_str(), font.data.as_slice()))
                .collect::<Vec<_>>()
        });
        let font_refs = font_refs.as_deref().unwrap_or(&[]);
        let (payload, diagnostics) = py
            .detach(|| self.inner.render_all_notes_with_diagnostics(dpi, font_refs))
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        let messages = PyTuple::new(py, diagnostics.iter().map(|d| d.message.as_str()))?;
        Ok((
            PyList::new(py, payload.iter().map(|bytes| PyBytes::new(py, bytes)))?,
            messages,
        ))
    }

    /// Replaces literal text in slides and speaker notes and returns the count.
    ///
    /// With `expect`, the replacement runs on a clone, so a count that
    /// differs raises and leaves the presentation and its revision as they
    /// were. Without it, the staged facade call runs in place. The revision
    /// advances once only when something was replaced.
    #[pyo3(signature = (placeholder, replacement, *, expect = None))]
    fn try_replace_text(
        &mut self,
        py: Python<'_>,
        placeholder: &str,
        replacement: &str,
        expect: Option<usize>,
    ) -> PyResult<usize> {
        let count = match expect {
            None => py
                .detach(|| self.inner.try_replace_text(placeholder, replacement))
                .map_err(|error| rpptx_to_pyerr(py, error))?,
            Some(expected) => {
                let (candidate, count) = py
                    .detach(|| {
                        let mut candidate = self.inner.clone();
                        candidate
                            .try_replace_text(placeholder, replacement)
                            .map(|count| (candidate, count))
                    })
                    .map_err(|error| rpptx_to_pyerr(py, error))?;
                if count != expected {
                    return Err(replacement_count_to_pyerr(py, placeholder, expected, count));
                }
                self.inner = candidate;
                count
            }
        };
        if count > 0 {
            self.revisions.bump();
        }
        Ok(count)
    }

    /// Alias for the checked literal replacement operation.
    #[pyo3(signature = (placeholder, replacement, *, expect = None))]
    fn replace_text(
        &mut self,
        py: Python<'_>,
        placeholder: &str,
        replacement: &str,
        expect: Option<usize>,
    ) -> PyResult<usize> {
        self.try_replace_text(py, placeholder, replacement, expect)
    }

    /// Returns every package and PresentationML invariant violation.
    fn validate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let issues = py.detach(|| self.inner.validate());
        PyTuple::new(py, issues.iter().map(PyValidationIssue::from))
    }

    #[getter]
    fn slide_width(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        length(py, self.inner.slide_size().map(|(width, _)| width))
    }

    #[setter(slide_width)]
    fn set_slide_width(&mut self, py: Python<'_>, value: i64) -> PyResult<()> {
        self.set_slide_size(py, Some(value), None)
    }

    #[getter]
    fn slide_height(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        length(py, self.inner.slide_size().map(|(_, height)| height))
    }

    #[setter(slide_height)]
    fn set_slide_height(&mut self, py: Python<'_>, value: i64) -> PyResult<()> {
        self.set_slide_size(py, None, Some(value))
    }

    #[getter]
    fn comment_authors<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(
            py,
            self.inner
                .comment_authors()
                .iter()
                .map(PyCommentAuthor::from),
        )
    }

    #[pyo3(signature = (*, id, name, user_id, provider_id, initials = None))]
    fn add_comment_author(
        &mut self,
        id: String,
        name: String,
        user_id: String,
        provider_id: String,
        initials: Option<&str>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let author = rpptx::CommentAuthor::new(id, name, initials, user_id, provider_id)
            .map_err(|error| rpptx_value_to_pyerr(py, error.to_string()))?;
        self.inner
            .add_comment_author(author)
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        self.revisions.bump();
        Ok(())
    }

    #[getter]
    fn slide_layouts(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PySlideLayoutCollection>> {
        let path = slf.borrow(py).revisions.capture(smallvec![]);
        Py::new(py, PySlideLayoutCollection::new(slf, path))
    }

    #[getter]
    fn slides(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PySlideCollection>> {
        let path = slf.borrow(py).revisions.capture(smallvec![]);
        Py::new(py, PySlideCollection::new(slf, path))
    }
}

fn collect_caller_fonts<'py>(
    py: Python<'py>,
    fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
    font_dir: Option<PathBuf>,
    font_dirs: Option<Bound<'py, PyAny>>,
) -> PyResult<Option<Vec<oxml_layout::FontFile>>> {
    if fonts.is_none() && font_dir.is_none() && font_dirs.is_none() {
        return Ok(None);
    }
    if font_dir.is_some() && font_dirs.is_some() {
        return Err(PyValueError::new_err(
            "font_dir and font_dirs cannot be supplied together",
        ));
    }
    let explicit = fonts
        .unwrap_or_default()
        .into_iter()
        .map(|(family, data)| oxml_layout::FontFile {
            family,
            data: data.as_bytes().to_vec(),
        })
        .collect::<Vec<_>>();
    let directories = match font_dirs {
        Some(paths) => {
            if paths.is_instance_of::<PyString>() || paths.is_instance_of::<PyBytes>() {
                return Err(PyTypeError::new_err(
                    "font_dirs must be a sequence of paths, not a scalar string or bytes",
                ));
            }
            paths
                .extract::<Vec<PathBuf>>()
                .map_err(|_| PyTypeError::new_err("font_dirs must be a sequence of paths"))?
        }
        None => font_dir.into_iter().collect(),
    };
    py.detach(|| {
        for directory in &directories {
            let metadata = std::fs::metadata(directory).map_err(|error| {
                std::io::Error::new(
                    error.kind(),
                    format!("font directory {}: {error}", directory.display()),
                )
            })?;
            if !metadata.is_dir() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    format!("font directory {} is not a directory", directory.display()),
                ));
            }
        }
        Ok::<_, std::io::Error>(())
    })
    .map_err(PyErr::from)?;
    let selected = py
        .detach(|| {
            let mut selected = Vec::new();
            for directory in directories {
                selected.extend(oxml_layout::FontFile::from_dir(&directory)?);
            }
            selected.extend(explicit);
            Ok::<_, oxml_layout::LayoutError>(selected)
        })
        .map_err(|error| rpptx_value_to_pyerr(py, error.to_string()))?;
    Ok((!selected.is_empty()).then_some(selected))
}
