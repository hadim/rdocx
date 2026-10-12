use std::path::PathBuf;

use oxml_py_support::RevisionCounter;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyList, PyString, PyTuple};
use smallvec::smallvec;

use crate::layout::PyTextFrameLayout;
use crate::shape::length;
use crate::slide::{
    PySlideCollection, PySlideLayoutCollection, PySlideMaster, PySlideMasterCollection,
};
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
    owners: Vec<LiveOwner>,
    slides: oxml_py_support::RevisionState,
    layouts: oxml_py_support::RevisionState,
    masters: oxml_py_support::RevisionState,
}

impl PyPresentation {
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
                .map_err(|error| rpptx_to_pyerr(py, error))
                .and_then(Self::from_presentation_owned),
            None => rpptx::Presentation::new()
                .map_err(|error| rpptx_to_pyerr(py, error))
                .and_then(Self::from_presentation_owned),
        }
    }

    #[staticmethod]
    fn from_bytes(bytes: &[u8], py: Python<'_>) -> PyResult<Self> {
        rpptx::Presentation::from_bytes(bytes)
            .map_err(|error| rpptx_to_pyerr(py, error))
            .and_then(Self::from_presentation_owned)
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

    /// Explicit authoring transaction. Existing content handles keep their paths.
    #[pyo3(signature=(*,fonts=None,font_dir=None,font_dirs=None))]
    fn refresh_autofit<'py>(
        &mut self,
        py: Python<'py>,
        fonts: Option<Vec<(String, Bound<'py, PyBytes>)>>,
        font_dir: Option<PathBuf>,
        font_dirs: Option<Bound<'py, PyAny>>,
    ) -> PyResult<crate::layout::PyAutofitRefreshResult> {
        let files = collect_caller_fonts(py, fonts, font_dir, font_dirs)?.unwrap_or_default();
        let refs = files
            .iter()
            .map(|f| (f.family.as_str(), f.data.as_slice()))
            .collect::<Vec<_>>();
        let result = py
            .detach(|| self.inner.refresh_autofit(&refs))
            .map_err(|e| rpptx_to_pyerr(py, e))?;
        Ok(crate::layout::PyAutofitRefreshResult::from(&result))
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
    /// were. Both paths prepare owner retirement before publication. The revision
    /// advances once only when something was replaced.
    #[pyo3(signature = (placeholder, replacement, *, expect = None))]
    fn try_replace_text(
        &mut self,
        py: Python<'_>,
        placeholder: &str,
        replacement: &str,
        expect: Option<usize>,
    ) -> PyResult<usize> {
        self.replace_text_candidate(
            py,
            placeholder,
            replacement,
            expect,
            "Presentation.try_replace_text",
        )
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
        self.replace_text_candidate(
            py,
            placeholder,
            replacement,
            expect,
            "Presentation.replace_text",
        )
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
        let mut candidate = self.inner.clone();
        candidate
            .add_comment_author(author)
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        let survivors = self.unchanged_owners()?;
        let prepared = self.prepare_owners(
            &candidate,
            "Presentation.add_comment_author",
            &survivors,
            false,
            false,
            false,
        )?;
        self.publish_owners(candidate, prepared)?;
        Ok(())
    }

    #[pyo3(signature = (slide_number=true, footer=None, date=None, hide_on_title=true, date_format="datetime1", *, date_cache=None))]
    fn set_header_footer(
        mut slf: PyRefMut<'_, Self>,
        slide_number: bool,
        footer: Option<String>,
        date: Option<String>,
        hide_on_title: bool,
        date_format: &str,
        date_cache: Option<String>,
    ) -> PyResult<()> {
        let py = slf.py();
        let presentation = &mut *slf;
        let settings = crate::slide::header_footer_settings(
            slide_number,
            footer,
            date,
            date_format,
            date_cache,
        );
        let mut candidate = presentation.inner.clone();
        if candidate
            .set_header_footer(&settings, hide_on_title)
            .map_err(|e| rpptx_to_pyerr(py, e))?
        {
            let proofs = presentation.footer_owners(&candidate)?;
            let prepared = presentation.prepare_owners(
                &candidate,
                "Presentation.set_header_footer",
                &proofs,
                false,
                false,
                false,
            )?;
            presentation.publish_owners(candidate, prepared)?;
        }
        Ok(())
    }

    #[getter]
    fn slide_number_start(&self, py: Python<'_>) -> PyResult<Option<u32>> {
        self.inner
            .slide_number_start()
            .map_err(|e| rpptx_to_pyerr(py, e))
    }

    #[setter]
    fn set_slide_number_start(&mut self, py: Python<'_>, value: Option<u32>) -> PyResult<()> {
        if self
            .inner
            .slide_number_start()
            .map_err(|e| rpptx_to_pyerr(py, e))?
            == value
        {
            return Ok(());
        }
        let mut candidate = self.inner.clone();
        candidate
            .set_slide_number_start(value)
            .map_err(|e| rpptx_to_pyerr(py, e))?;
        let proofs = self.unchanged_owners()?;
        let prepared = self.prepare_owners(
            &candidate,
            "Presentation.slide_number_start",
            &proofs,
            false,
            false,
            false,
        )?;
        self.publish_owners(candidate, prepared)
    }

    #[getter]
    fn slide_masters(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PySlideMasterCollection>> {
        let path = slf
            .borrow(py)
            .revisions
            .capture_scopes(smallvec![], &[slf.borrow(py).masters.clone()])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        Py::new(py, PySlideMasterCollection::new(slf, path))
    }

    #[getter]
    fn slide_master(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PySlideMaster>> {
        let path = slf
            .borrow(py)
            .revisions
            .capture_scopes(smallvec![], &[slf.borrow(py).masters.clone()])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        PySlideMasterCollection::new(slf, path).item(py, 0)
    }

    #[getter]
    fn slide_layouts(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PySlideLayoutCollection>> {
        let path = slf
            .borrow(py)
            .revisions
            .capture_scopes(smallvec![], &[slf.borrow(py).layouts.clone()])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        Py::new(py, PySlideLayoutCollection::new(slf, path))
    }

    #[getter]
    fn slides(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PySlideCollection>> {
        let path = slf.borrow(py).capture_path(smallvec![], None)?;
        Py::new(py, PySlideCollection::new(slf, path))
    }
}

// Physical addresses are current lookup locations, not lifetime identities.
#[derive(Clone, Copy)]
pub(crate) enum SlideOwnerEdit {
    Insert(usize),
    Remove(usize),
    Move { from: usize, to: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PhysicalOwner {
    Slide(u32),
    Shape {
        slide: u32,
        id: Option<u32>,
        kind: rpptx::ShapeKind,
    },
    Cell {
        slide: u32,
        table: Option<u32>,
        row: usize,
        column: usize,
    },
    Layout(String),
}

#[derive(Clone, Debug)]
struct OwnerAddress {
    physical: PhysicalOwner,
    path: smallvec::SmallVec<[oxml_py_support::PathSeg; 5]>,
    layout_index: Option<usize>,
    table: bool,
}

#[derive(Clone, Debug)]
struct LiveOwner {
    address: OwnerAddress,
    owner: oxml_py_support::RevisionState,
    text: oxml_py_support::RevisionState,
    grid: Option<oxml_py_support::RevisionState>,
}

/// An operation's explicit survival proof. Omitted old generations are deleted.
/// A matching native ID never constructs this proof automatically.
pub(crate) struct OwnerSurvival {
    pub(crate) generation: u64,
    pub(crate) destination: smallvec::SmallVec<[oxml_py_support::PathSeg; 5]>,
    pub(crate) layout_index: Option<usize>,
    pub(crate) text_changed: bool,
    pub(crate) grid_changed: bool,
}

pub(crate) struct PreparedOwners {
    revision: oxml_py_support::PreparedRevision,
    owners: Vec<LiveOwner>,
    slides: oxml_py_support::RevisionState,
    layouts: oxml_py_support::RevisionState,
    masters: oxml_py_support::RevisionState,
}

fn owner_addresses(inner: &rpptx::Presentation) -> PyResult<Vec<OwnerAddress>> {
    use oxml_py_support::PathSeg;
    fn children(
        shape: rpptx::ShapeRef<'_>,
        slide: u32,
        path: smallvec::SmallVec<[PathSeg; 5]>,
        out: &mut Vec<OwnerAddress>,
    ) -> PyResult<()> {
        // AlternateContent remains an opaque physical owner. Its fallback
        // children cannot become mutable addresses by traversing this inventory.
        out.try_reserve(1).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner inventory allocation failed")
        })?;
        out.push(OwnerAddress {
            physical: PhysicalOwner::Shape {
                slide,
                id: shape.non_visual_id(),
                kind: shape.kind(),
            },
            path: path.clone(),
            layout_index: None,
            table: shape.table().is_some(),
        });
        if let Some(table) = shape.table() {
            for row in 0..table.row_count() {
                for column in 0..table.column_count() {
                    if table.cell(row, column).is_none() {
                        continue;
                    }
                    let mut cell_path = path.clone();
                    cell_path.try_reserve(2).map_err(|_| {
                        pyo3::exceptions::PyMemoryError::new_err(
                            "cell owner path allocation failed",
                        )
                    })?;
                    cell_path.extend([PathSeg::Row(row), PathSeg::Cell(column)]);
                    out.try_reserve(1).map_err(|_| {
                        pyo3::exceptions::PyMemoryError::new_err("cell inventory allocation failed")
                    })?;
                    out.push(OwnerAddress {
                        physical: PhysicalOwner::Cell {
                            slide,
                            table: shape.non_visual_id(),
                            row,
                            column,
                        },
                        path: cell_path,
                        layout_index: None,
                        table: false,
                    });
                }
            }
        }
        if shape.kind() == rpptx::ShapeKind::Group {
            for (index, child) in shape.children().enumerate() {
                let mut nested = path.clone();
                nested.try_reserve(1).map_err(|_| {
                    pyo3::exceptions::PyMemoryError::new_err("owner path allocation failed")
                })?;
                nested.push(PathSeg::Shape(index));
                children(child, slide, nested, out)?;
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    for (index, slide) in inner.slides().enumerate() {
        out.try_reserve(1).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner inventory allocation failed")
        })?;
        out.push(OwnerAddress {
            physical: PhysicalOwner::Slide(slide.id()),
            path: smallvec![PathSeg::Slide(index)],
            layout_index: None,
            table: false,
        });
        for (shape_index, shape) in slide.shapes().enumerate() {
            children(
                shape,
                slide.id(),
                smallvec![PathSeg::Slide(index), PathSeg::Shape(shape_index)],
                &mut out,
            )?;
        }
    }
    for index in 0..inner.layout_count() {
        // Originating layout parts provide physical ownership independently of order.
        let part = inner
            .layout_part_name(index)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("layout owner is absent"))?;
        out.try_reserve(1).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner inventory allocation failed")
        })?;
        out.push(OwnerAddress {
            physical: PhysicalOwner::Layout(part.to_owned()),
            path: smallvec![],
            layout_index: Some(index),
            table: false,
        });
    }
    Ok(out)
}

impl PyPresentation {
    /// Fallible construction before any Python handles can escape.
    fn from_presentation_owned(inner: rpptx::Presentation) -> PyResult<Self> {
        use oxml_py_support::RevisionScope as S;
        let addresses = owner_addresses(&inner)?;
        let mut kinds = vec![S::SlideInventory, S::LayoutInventory, S::MasterInventory];
        for address in &addresses {
            kinds.try_reserve(3).map_err(|_| {
                pyo3::exceptions::PyMemoryError::new_err("owner scopes allocation failed")
            })?;
            kinds.extend([S::Owner, S::Text]);
            if address.table {
                kinds.push(S::Grid);
            }
        }
        let mut revisions = oxml_py_support::RevisionCounter::new();
        let mut states = revisions
            .initialize_scopes(&kinds)
            .map_err(oxml_py_support::revision_to_pyerr)?
            .into_iter();
        let slides = states
            .next()
            .expect("three inventory scopes were initialized");
        let layouts = states
            .next()
            .expect("three inventory scopes were initialized");
        let masters = states
            .next()
            .expect("three inventory scopes were initialized");
        let mut owners = Vec::new();
        owners.try_reserve_exact(addresses.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner inventory allocation failed")
        })?;
        for address in addresses {
            let owner = states.next().expect("owner scope was initialized");
            let text = states.next().expect("text scope was initialized");
            let grid = address
                .table
                .then(|| states.next().expect("table grid scope was initialized"));
            owners.push(LiveOwner {
                address,
                owner,
                text,
                grid,
            });
        }
        Ok(Self {
            inner,
            revisions,
            owners,
            slides,
            layouts,
            masters,
        })
    }

    fn owner_at(
        &self,
        path: &[oxml_py_support::PathSeg],
        layout_index: Option<usize>,
    ) -> Option<&LiveOwner> {
        self.owners.iter().find(|owner| {
            owner.address.path.as_slice() == path && owner.address.layout_index == layout_index
        })
    }

    pub(crate) fn current_path(
        &self,
        captured: &oxml_py_support::ContentPath,
    ) -> Option<oxml_py_support::ContentPath> {
        use oxml_py_support::PathSeg;
        if !captured.belongs_to(&self.revisions) {
            return None;
        }
        let Some(generation) = captured.owner_generation() else {
            return Some(captured.clone());
        };
        let owner = self
            .owners
            .iter()
            .find(|owner| owner.owner.generation() == generation)?;
        let mut path = captured.clone();
        let end = match &owner.address.physical {
            PhysicalOwner::Cell { .. } => captured
                .segs
                .iter()
                .rposition(|seg| matches!(seg, PathSeg::Cell(_)))?,
            PhysicalOwner::Shape { .. } => captured
                .segs
                .iter()
                .rposition(|seg| matches!(seg, PathSeg::Shape(_)))?,
            PhysicalOwner::Slide(_) => captured
                .segs
                .iter()
                .position(|seg| matches!(seg, PathSeg::Slide(_)))?,
            PhysicalOwner::Layout(_) => return Some(captured.clone()),
        };
        let suffix = captured.segs.iter().skip(end + 1).copied();
        path.segs = owner.address.path.clone();
        path.segs.extend(suffix);
        Some(path)
    }

    /// ID-based native operations require one qualified current physical owner.
    /// A parsed duplicate ID remains inspectable but cannot select another shape.
    pub(crate) fn checked_shape_id(&self, path: &oxml_py_support::ContentPath) -> PyResult<u32> {
        let current = self
            .current_path(path)
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("shape owner is absent"))?;
        let owner = self.owner_at(&current.segs, None).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err("shape is not a modeled mutable owner")
        })?;
        let PhysicalOwner::Shape {
            slide,
            id: Some(id),
            kind,
        } = &owner.address.physical
        else {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "shape has no qualified id",
            ));
        };
        if *kind == rpptx::ShapeKind::AlternateContent || self.owners.iter().filter(|candidate| {
            matches!(&candidate.address.physical, PhysicalOwner::Shape { slide: other_slide, id: Some(other_id), .. } if other_slide == slide && other_id == id)
        }).count() != 1 {
            return Err(pyo3::exceptions::PyValueError::new_err("shape id is ambiguous or read-only"));
        }
        Ok(*id)
    }

    pub(crate) fn capture_path(
        &self,
        segs: smallvec::SmallVec<[oxml_py_support::PathSeg; 5]>,
        descendants: Option<oxml_py_support::RevisionScope>,
    ) -> PyResult<oxml_py_support::ContentPath> {
        self.capture_owner_path(
            &self.owners,
            &self.slides,
            self.revisions.current(),
            segs,
            descendants,
        )
    }

    pub(crate) fn capture_prepared_path(
        &self,
        prepared: &PreparedOwners,
        segs: smallvec::SmallVec<[oxml_py_support::PathSeg; 5]>,
        descendants: Option<oxml_py_support::RevisionScope>,
    ) -> PyResult<oxml_py_support::ContentPath> {
        self.capture_owner_path(
            &prepared.owners,
            &prepared.slides,
            prepared.revision.committed_serial(),
            segs,
            descendants,
        )
    }

    fn capture_owner_path(
        &self,
        owners: &[LiveOwner],
        slides: &oxml_py_support::RevisionState,
        serial: u64,
        segs: smallvec::SmallVec<[oxml_py_support::PathSeg; 5]>,
        descendants: Option<oxml_py_support::RevisionScope>,
    ) -> PyResult<oxml_py_support::ContentPath> {
        use oxml_py_support::PathSeg;
        let owner_at = |path: &[PathSeg], layout_index: Option<usize>| {
            owners.iter().find(|owner| {
                owner.address.path.as_slice() == path && owner.address.layout_index == layout_index
            })
        };
        if segs.is_empty() {
            let mut path = self
                .revisions
                .capture_scopes(segs, std::slice::from_ref(slides))
                .map_err(oxml_py_support::revision_to_pyerr)?;
            path.revision = serial;
            return Ok(path);
        }
        let physical = segs
            .iter()
            .filter(|seg| matches!(seg, PathSeg::Slide(_) | PathSeg::Shape(_)))
            .copied()
            .collect::<smallvec::SmallVec<[PathSeg; 5]>>();
        let cell = segs.iter().any(|seg| matches!(seg, PathSeg::Row(_)))
            && segs.iter().any(|seg| matches!(seg, PathSeg::Cell(_)));
        let owner_path = if cell {
            segs.iter()
                .filter(|seg| !matches!(seg, PathSeg::Para(_) | PathSeg::Run(_)))
                .copied()
                .collect::<smallvec::SmallVec<[PathSeg; 5]>>()
        } else {
            physical.clone()
        };
        let owner = match owner_at(&owner_path, None) {
            Some(owner) => owner,
            None => {
                // Opaque AlternateContent fallback inspection keeps the existing
                // conservative document scope. It never mints mutable owners or
                // silently rebinds a fallback after an invalidating publication.
                let opaque_parent = owners.iter().any(|owner| {
                    matches!(
                        &owner.address.physical,
                        PhysicalOwner::Shape {
                            kind: rpptx::ShapeKind::AlternateContent,
                            ..
                        }
                    ) && owner_path.starts_with(&owner.address.path)
                        && owner_path.len() > owner.address.path.len()
                });
                if opaque_parent && serial == self.revisions.current() {
                    return Ok(self.revisions.capture(segs));
                }
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "physical owner is absent",
                ));
            }
        };
        let slide_path = physical
            .first()
            .copied()
            .into_iter()
            .collect::<smallvec::SmallVec<[PathSeg; 5]>>();
        let slide = owner_at(&slide_path, None)
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("slide owner is absent"))?;
        let mut scopes = smallvec::SmallVec::<[oxml_py_support::RevisionState; 4]>::new();
        scopes.push(slide.owner.clone());
        if !owner.owner.same_state(&slide.owner) {
            scopes.push(owner.owner.clone());
        }
        if descendants == Some(oxml_py_support::RevisionScope::Grid)
            || segs
                .iter()
                .any(|seg| matches!(seg, PathSeg::Row(_) | PathSeg::Cell(_)))
        {
            let table_owner = owner_at(&physical, None)
                .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("table owner is absent"))?;
            scopes.push(
                table_owner
                    .grid
                    .as_ref()
                    .ok_or_else(|| {
                        pyo3::exceptions::PyValueError::new_err("owner has no table grid")
                    })?
                    .clone(),
            );
        }
        if descendants == Some(oxml_py_support::RevisionScope::Text)
            || segs
                .iter()
                .any(|seg| matches!(seg, PathSeg::Para(_) | PathSeg::Run(_)))
        {
            scopes.push(owner.text.clone());
        }
        let mut path = self
            .revisions
            .capture_scopes(segs, &scopes)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        path.revision = serial;
        Ok(path)
    }

    pub(crate) fn capture_layout(&self, index: usize) -> PyResult<oxml_py_support::ContentPath> {
        let owner = self
            .owner_at(&[], Some(index))
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("layout owner is absent"))?;
        self.revisions
            .capture_scopes(smallvec![], std::slice::from_ref(&owner.owner))
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    pub(crate) fn current_layout_index(
        &self,
        path: &oxml_py_support::ContentPath,
    ) -> Option<usize> {
        let generation = path.owner_generation()?;
        self.owners
            .iter()
            .find(|owner| owner.owner.generation() == generation)?
            .address
            .layout_index
    }

    /// Build a complete candidate inventory, checking all survival proofs.
    /// It never infers that an ID still names the same incarnation.
    pub(crate) fn prepare_owners(
        &self,
        candidate: &rpptx::Presentation,
        operation: &'static str,
        preserved: &[OwnerSurvival],
        slides_changed: bool,
        layouts_changed: bool,
        masters_changed: bool,
    ) -> PyResult<PreparedOwners> {
        use oxml_py_support::RevisionScope as S;
        let next = owner_addresses(candidate)?;
        let fail = |message: &str| pyo3::exceptions::PyValueError::new_err(message.to_owned());
        let mut seen = std::collections::HashSet::new();
        let mut destinations = std::collections::HashSet::new();
        seen.try_reserve(preserved.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("survival identities allocation failed")
        })?;
        destinations.try_reserve(preserved.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("survival destinations allocation failed")
        })?;
        let mut retained = Vec::new();
        retained.try_reserve_exact(preserved.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("survival proof allocation failed")
        })?;
        for proof in preserved {
            if !seen.insert(proof.generation) {
                return Err(fail("duplicate owner survival proof"));
            }
            let old = self
                .owners
                .iter()
                .find(|owner| owner.owner.generation() == proof.generation)
                .ok_or_else(|| fail("survival proof has no current owner"))?;
            let index = next
                .iter()
                .position(|address| {
                    address.path == proof.destination && address.layout_index == proof.layout_index
                })
                .ok_or_else(|| fail("survival destination is absent"))?;
            if !destinations.insert(index) {
                return Err(fail("two owners claim one destination"));
            }
            if old.address.physical != next[index].physical
                || old.address.table != next[index].table
            {
                return Err(fail("survival proof changes physical owner identity"));
            }
            retained.push((index, old, proof));
        }
        let structural = next.len() != self.owners.len()
            || retained.iter().any(|(_, old, proof)| {
                old.address.path != proof.destination
                    || old.address.layout_index != proof.layout_index
            });
        if structural {
            // A mutation cannot disambiguate imported missing/duplicate IDs by
            // merely finding the same number again in a candidate tree.
            for (_, old, _) in &retained {
                if let PhysicalOwner::Shape { slide, id, kind } = &old.address.physical {
                    if *kind == rpptx::ShapeKind::AlternateContent {
                        continue;
                    }
                    let Some(id) = id else {
                        return Err(fail("structural owner proof requires a shape id"));
                    };
                    let original_count = self.owners.iter().filter(|owner| matches!(&owner.address.physical,
                        PhysicalOwner::Shape { slide: at, id: Some(value), .. } if at == slide && value == id)).count();
                    let candidate_count = next.iter().filter(|address| matches!(&address.physical,
                        PhysicalOwner::Shape { slide: at, id: Some(value), .. } if at == slide && value == id)).count();
                    if original_count != 1 || candidate_count != 1 {
                        return Err(fail("structural owner proof has an ambiguous shape id"));
                    }
                }
            }
        }
        let mut affected = Vec::new();
        let mut replacements = Vec::new();
        let affected_capacity = self
            .owners
            .len()
            .checked_mul(3)
            .and_then(|count| count.checked_add(3))
            .ok_or_else(|| {
                pyo3::exceptions::PyOverflowError::new_err("owner retirement count overflow")
            })?;
        affected.try_reserve_exact(affected_capacity).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner retirement allocation failed")
        })?;
        replacements
            .try_reserve_exact(preserved.len().checked_mul(2).ok_or_else(|| {
                pyo3::exceptions::PyOverflowError::new_err("owner replacement count overflow")
            })?)
            .map_err(|_| {
                pyo3::exceptions::PyMemoryError::new_err("owner replacements allocation failed")
            })?;
        for old in &self.owners {
            if !seen.contains(&old.owner.generation()) {
                affected.push(old.owner.clone());
                affected.push(old.text.clone());
                if let Some(grid) = &old.grid {
                    affected.push(grid.clone());
                }
            }
        }
        for (index, old, proof) in &retained {
            if proof.text_changed {
                replacements.push((*index, S::Text, affected.len()));
                affected.push(old.text.clone());
            }
            if proof.grid_changed {
                replacements.push((*index, S::Grid, affected.len()));
                affected.push(
                    old.grid
                        .as_ref()
                        .ok_or_else(|| fail("grid change on non-table"))?
                        .clone(),
                );
            }
        }
        let slides_at = slides_changed.then(|| {
            let index = affected.len();
            affected.push(self.slides.clone());
            index
        });
        let layouts_at = layouts_changed.then(|| {
            let index = affected.len();
            affected.push(self.layouts.clone());
            index
        });
        let masters_at = masters_changed.then(|| {
            let index = affected.len();
            affected.push(self.masters.clone());
            index
        });
        let mut births = Vec::new();
        let mut birth_offsets = Vec::new();
        births
            .try_reserve_exact(next.len().checked_mul(3).ok_or_else(|| {
                pyo3::exceptions::PyOverflowError::new_err("owner birth count overflow")
            })?)
            .map_err(|_| {
                pyo3::exceptions::PyMemoryError::new_err("owner births allocation failed")
            })?;
        birth_offsets.try_reserve_exact(next.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner birth offsets allocation failed")
        })?;
        for (index, address) in next.iter().enumerate() {
            if !destinations.contains(&index) {
                birth_offsets.push((index, births.len()));
                births.extend([S::Owner, S::Text]);
                if address.table {
                    births.push(S::Grid);
                }
            }
        }
        let revision = self
            .revisions
            .prepare_scopes(operation, &affected, &births)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        let mut owners = Vec::new();
        owners.try_reserve_exact(next.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("candidate owner inventory allocation failed")
        })?;
        for (index, address) in next.into_iter().enumerate() {
            if let Some((_, old, _)) = retained.iter().find(|(at, _, _)| *at == index) {
                let mut owner = (**old).clone();
                owner.address = address;
                for (_, scope, at) in replacements.iter().filter(|(at, _, _)| *at == index) {
                    let successor = revision
                        .successor(*at)
                        .expect("affected scope successor exists");
                    match scope {
                        S::Text => owner.text = successor,
                        S::Grid => owner.grid = Some(successor),
                        _ => unreachable!(),
                    }
                }
                owners.push(owner);
            } else {
                let start = birth_offsets
                    .iter()
                    .find(|(at, _)| *at == index)
                    .expect("every unclaimed address has birth scopes")
                    .1;
                let grid = address
                    .table
                    .then(|| revision.born(start + 2).expect("table birth exists"));
                owners.push(LiveOwner {
                    address,
                    owner: revision.born(start).expect("owner birth exists"),
                    text: revision.born(start + 1).expect("text birth exists"),
                    grid,
                });
            }
        }
        let slides = slides_at.map_or_else(
            || self.slides.clone(),
            |at| revision.successor(at).expect("inventory successor exists"),
        );
        let layouts = layouts_at.map_or_else(
            || self.layouts.clone(),
            |at| revision.successor(at).expect("inventory successor exists"),
        );
        let masters = masters_at.map_or_else(
            || self.masters.clone(),
            |at| revision.successor(at).expect("inventory successor exists"),
        );
        Ok(PreparedOwners {
            revision,
            owners,
            slides,
            layouts,
            masters,
        })
    }

    /// No fallible action or Python allocation follows successful token commit.
    pub(crate) fn publish_owners(
        &mut self,
        candidate: rpptx::Presentation,
        prepared: PreparedOwners,
    ) -> PyResult<()> {
        self.revisions
            .commit_scopes(prepared.revision)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        self.inner = candidate;
        self.owners = prepared.owners;
        self.slides = prepared.slides;
        self.layouts = prepared.layouts;
        self.masters = prepared.masters;
        Ok(())
    }
}

// Existing presentation.rs helpers. No current-owner history is retained.
impl PyPresentation {
    fn replace_text_candidate(
        &mut self,
        py: Python<'_>,
        placeholder: &str,
        replacement: &str,
        expect: Option<usize>,
        operation: &'static str,
    ) -> PyResult<usize> {
        let mut candidate = self.inner.clone();
        let count = py
            .detach(|| candidate.try_replace_text(placeholder, replacement))
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        if let Some(expected) = expect.filter(|expected| *expected != count) {
            return Err(replacement_count_to_pyerr(py, placeholder, expected, count));
        }
        if count > 0 {
            let survivors = self.replaced_text_owners(&candidate)?;
            let prepared =
                self.prepare_owners(&candidate, operation, &survivors, false, false, false)?;
            self.publish_owners(candidate, prepared)?;
        }
        Ok(count)
    }

    /// Footer staging only removes owned footer shapes, appends fresh copies,
    /// and preserves every surviving physical identity. Match that explicit
    /// native guarantee and compare complete text owners before publication.
    pub(crate) fn footer_owners(
        &self,
        candidate: &rpptx::Presentation,
    ) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::ContentPath;
        let next = owner_addresses(candidate)?;
        let mut proofs = Vec::new();
        proofs.try_reserve_exact(self.owners.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("footer owner allocation failed")
        })?;
        for old in &self.owners {
            let matching = next
                .iter()
                .filter(|address| address.physical == old.address.physical)
                .collect::<Vec<_>>();
            let [after] = matching.as_slice() else {
                if matching.is_empty() {
                    continue;
                }
                return Err(PyValueError::new_err("footer owner identity is ambiguous"));
            };
            let before_path = ContentPath::new(old.address.path.clone(), self.revisions.current());
            let after_path = ContentPath::new(after.path.clone(), self.revisions.current());
            let text_changed = match &old.address.physical {
                PhysicalOwner::Shape { .. } => {
                    crate::shape::native_shape_ref_at(&self.inner, &before_path)
                        .and_then(|shape| shape.text_frame())
                        != crate::shape::native_shape_ref_at(candidate, &after_path)
                            .and_then(|shape| shape.text_frame())
                }
                // Footer publication does not change notes or any table cell.
                PhysicalOwner::Slide(_) | PhysicalOwner::Cell { .. } | PhysicalOwner::Layout(_) => {
                    false
                }
            };
            proofs.push(OwnerSurvival {
                generation: old.owner.generation(),
                destination: after.path.clone(),
                layout_index: after.layout_index,
                text_changed,
                grid_changed: false,
            });
        }
        Ok(proofs)
    }

    pub(crate) fn unchanged_owners(&self) -> PyResult<Vec<OwnerSurvival>> {
        let mut proofs = Vec::new();
        proofs.try_reserve_exact(self.owners.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner proofs allocation failed")
        })?;
        proofs.extend(self.owners.iter().map(|owner| OwnerSurvival {
            generation: owner.owner.generation(),
            destination: owner.address.path.clone(),
            layout_index: owner.address.layout_index,
            text_changed: false,
            grid_changed: false,
        }));
        Ok(proofs)
    }

    /// Remap exactly one existing sibling ordinal. The operation must use this
    /// same permutation on its staged native tree before preparing publication.
    pub(crate) fn moved_shape_owners(
        &self,
        parent: &[oxml_py_support::PathSeg],
        from: usize,
        to: usize,
    ) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::PathSeg;
        let mut proofs = self.unchanged_owners()?;
        for proof in &mut proofs {
            if proof.layout_index.is_some() || !proof.destination.starts_with(parent) {
                continue;
            }
            let Some(PathSeg::Shape(index)) = proof.destination.get_mut(parent.len()) else {
                continue;
            };
            *index = if *index == from {
                to
            } else if from < to && *index > from && *index <= to {
                *index - 1
            } else if to < from && *index >= to && *index < from {
                *index + 1
            } else {
                *index
            };
        }
        Ok(proofs)
    }

    pub(crate) fn removed_shape_owners(
        &self,
        parent: &[oxml_py_support::PathSeg],
        removed: usize,
    ) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::PathSeg;
        let mut proofs = self.unchanged_owners()?;
        proofs.retain_mut(|proof| {
            if proof.layout_index.is_some() || !proof.destination.starts_with(parent) {
                return true;
            }
            let Some(PathSeg::Shape(index)) = proof.destination.get_mut(parent.len()) else {
                return true;
            };
            if *index == removed {
                return false;
            }
            if *index > removed {
                *index -= 1;
            }
            true
        });
        Ok(proofs)
    }

    /// Apply the same explicit ordinal operation to every surviving physical
    /// slide descendant. Removed members never contribute a survival proof.
    pub(crate) fn slide_owner_edit(&self, edit: SlideOwnerEdit) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::PathSeg;
        let mut proofs = self.unchanged_owners()?;
        proofs.retain_mut(|proof| {
            let Some(PathSeg::Slide(index)) = proof.destination.first_mut() else {
                return true;
            };
            match edit {
                SlideOwnerEdit::Insert(at) => {
                    if *index >= at {
                        *index += 1;
                    }
                }
                SlideOwnerEdit::Remove(at) => {
                    if *index == at {
                        return false;
                    }
                    if *index > at {
                        *index -= 1;
                    }
                }
                SlideOwnerEdit::Move { from, to } => {
                    *index = if *index == from {
                        to
                    } else if from < to && *index > from && *index <= to {
                        *index - 1
                    } else if to < from && *index >= to && *index < from {
                        *index + 1
                    } else {
                        *index
                    };
                }
            }
            true
        });
        Ok(proofs)
    }

    /// Literal replacement preserves physical owners but can redistribute text
    /// between runs without changing flattened text. Compare accepted bodies,
    /// including opaque facts, before retaining descendant-text generations.
    pub(crate) fn replaced_text_owners(
        &self,
        candidate: &rpptx::Presentation,
    ) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::ContentPath;
        let mut proofs = self.unchanged_owners()?;
        for (owner, proof) in self.owners.iter().zip(&mut proofs) {
            let path = ContentPath::new(owner.address.path.clone(), self.revisions.current());
            proof.text_changed = match &owner.address.physical {
                PhysicalOwner::Slide(_) => {
                    let index = crate::shape::slide_index(&path)?;
                    match (self.inner.slide(index), candidate.slide(index)) {
                        (Some(before), Some(after)) => !before.notes_text_matches(&after),
                        (None, None) => false,
                        _ => true,
                    }
                }
                PhysicalOwner::Shape { .. } => {
                    let before = crate::shape::native_shape_ref_at(&self.inner, &path)
                        .and_then(|shape| shape.text_frame());
                    let after = crate::shape::native_shape_ref_at(candidate, &path)
                        .and_then(|shape| shape.text_frame());
                    before != after
                }
                PhysicalOwner::Cell { row, column, .. } => {
                    let before = crate::shape::native_shape_ref_at(&self.inner, &path)
                        .and_then(|shape| shape.table())
                        .and_then(|table| table.cell(*row, *column))
                        .and_then(|cell| cell.text_frame());
                    let after = crate::shape::native_shape_ref_at(candidate, &path)
                        .and_then(|shape| shape.table())
                        .and_then(|table| table.cell(*row, *column))
                        .and_then(|cell| cell.text_frame());
                    before != after
                }
                PhysicalOwner::Layout(_) => false,
            };
        }
        Ok(proofs)
    }

    pub(crate) fn cell_text_owners(
        &self,
        path: &oxml_py_support::ContentPath,
        first_row: usize,
        last_row: usize,
        first_column: usize,
        last_column: usize,
    ) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::PathSeg;
        let table_path = path
            .segs
            .iter()
            .filter(|seg| matches!(seg, PathSeg::Slide(_) | PathSeg::Shape(_)))
            .copied()
            .collect::<smallvec::SmallVec<[PathSeg; 5]>>();
        let mut proofs = self.unchanged_owners()?;
        for (owner, proof) in self.owners.iter().zip(&mut proofs) {
            if owner.address.layout_index.is_some() || !owner.address.path.starts_with(&table_path)
            {
                continue;
            }
            if let PhysicalOwner::Cell { row, column, .. } = &owner.address.physical {
                proof.text_changed = (*row >= first_row && *row <= last_row)
                    && (*column >= first_column && *column <= last_column);
            }
        }
        Ok(proofs)
    }

    pub(crate) fn text_changed_owners(
        &self,
        path: &oxml_py_support::ContentPath,
    ) -> PyResult<Vec<OwnerSurvival>> {
        let mut proofs = self.unchanged_owners()?;
        let generation = path.owner_generation().ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err("text mutation needs a bound owner")
        })?;
        let proof = proofs
            .iter_mut()
            .find(|proof| proof.generation == generation)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("text owner is absent"))?;
        proof.text_changed = true;
        Ok(proofs)
    }

    pub(crate) fn grid_changed_owners(
        &self,
        path: &oxml_py_support::ContentPath,
    ) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::PathSeg;
        let current = self
            .current_path(path)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("grid owner is absent"))?;
        let table_path = current
            .segs
            .iter()
            .filter(|seg| matches!(seg, PathSeg::Slide(_) | PathSeg::Shape(_)))
            .copied()
            .collect::<smallvec::SmallVec<[PathSeg; 5]>>();
        let table = self
            .owner_at(&table_path, None)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("table owner is absent"))?;
        let mut proofs = self.unchanged_owners()?;
        // Every old physical cell retires on grid restructure. The table
        // survives with a fresh grid generation, and new cells get births.
        proofs.retain(|proof| {
            !(proof.destination.starts_with(&table_path)
                && proof
                    .destination
                    .get(table_path.len())
                    .is_some_and(|seg| matches!(seg, PathSeg::Row(_))))
        });
        proofs
            .iter_mut()
            .find(|proof| proof.generation == table.owner.generation())
            .expect("table inventory supplied survival proof")
            .grid_changed = true;
        Ok(proofs)
    }
}

// Replacement of existing shape.rs move_shape body AFTER range validation.
// Concrete source insertion, not a forwarding wrapper.
// let parent = smallvec![PathSeg::Slide(slide_index)];
// let mut presentation = self.presentation.borrow_mut(py);
// let mut candidate = presentation.inner.clone();
// candidate.move_shape(slide_index, from_, to).map_err(|e| rpptx_to_pyerr(py, e))?;
// let proofs = presentation.moved_shape_owners(&parent, from_, to)?;
// let prepared = presentation.prepare_owners(&candidate, "ShapeCollection.move", &proofs, false, false, false)?;
// presentation.publish_owners(candidate, prepared)
// Preserve the existing native no-op definition for from_==to before serial prep.

// Removal cannot use remove_shape alone as final acceptance. It must first
// detach incoming endpoints and preserve their last resolved geometry in this
// SAME candidate. The final authored removal method returns only after both
// detachment and native structural removal succeed. The owner proof excludes
// the removed subtree and shifts surviving ordinals. Publication is one serial.

// Candidate additions inside existing presentation.rs impl, not a new module.
impl PyPresentation {
    pub(crate) fn grouped_shape_owners(
        &self,
        parent: &[oxml_py_support::PathSeg],
        start: usize,
        count: usize,
    ) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::PathSeg;
        let end = start
            .checked_add(count)
            .ok_or_else(|| pyo3::exceptions::PyOverflowError::new_err("group range overflow"))?;
        if count == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "empty group selection",
            ));
        }
        let mut proofs = self.unchanged_owners()?;
        for proof in &mut proofs {
            if proof.layout_index.is_some() || !proof.destination.starts_with(parent) {
                continue;
            }
            let Some(PathSeg::Shape(index)) = proof.destination.get(parent.len()).copied() else {
                continue;
            };
            if index >= start && index < end {
                proof.destination.try_reserve(1).map_err(|_| {
                    pyo3::exceptions::PyMemoryError::new_err("group path allocation failed")
                })?;
                proof.destination[parent.len()] = PathSeg::Shape(start);
                proof
                    .destination
                    .insert(parent.len() + 1, PathSeg::Shape(index - start));
            } else if index >= end {
                proof.destination[parent.len()] = PathSeg::Shape(index - (count - 1));
            }
        }
        Ok(proofs)
    }
    pub(crate) fn ungrouped_shape_owners(
        &self,
        parent: &[oxml_py_support::PathSeg],
        index: usize,
        count: usize,
    ) -> PyResult<Vec<OwnerSurvival>> {
        use oxml_py_support::PathSeg;
        let mut proofs = self.unchanged_owners()?;
        proofs.retain(|proof| {
            !(proof.destination.starts_with(parent)
                && proof.destination.len() == parent.len() + 1
                && proof.destination[parent.len()] == PathSeg::Shape(index))
        });
        for proof in &mut proofs {
            if proof.layout_index.is_some() || !proof.destination.starts_with(parent) {
                continue;
            }
            let Some(PathSeg::Shape(at)) = proof.destination.get(parent.len()).copied() else {
                continue;
            };
            if at == index {
                let Some(PathSeg::Shape(child)) = proof.destination.get(parent.len() + 1).copied()
                else {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "invalid group child proof",
                    ));
                };
                if child >= count {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "group child proof out of range",
                    ));
                }
                proof.destination[parent.len()] =
                    PathSeg::Shape(index.checked_add(child).ok_or_else(|| {
                        pyo3::exceptions::PyOverflowError::new_err("ungroup index overflow")
                    })?);
                proof.destination.remove(parent.len() + 1);
            } else if at > index {
                proof.destination[parent.len()] = PathSeg::Shape(
                    at.checked_sub(1)
                        .and_then(|v| v.checked_add(count))
                        .ok_or_else(|| {
                            pyo3::exceptions::PyOverflowError::new_err("ungroup index overflow")
                        })?,
                );
            }
        }
        Ok(proofs)
    }
}

pub(crate) fn collect_caller_fonts<'py>(
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
