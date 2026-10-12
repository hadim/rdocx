use std::path::PathBuf;

use oxml_py_support::RevisionCounter;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};
use smallvec::smallvec;

use crate::layout::PyTextFrameLayout;
use crate::shape::{image_bytes, length};
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

/// The caller's explicit authorization and exact license identity for one
/// embedded font face. Neither has a default, so authorization is always a
/// deliberate argument.
#[pyclass(
    name = "FontEmbeddingLicense",
    frozen,
    get_all,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyFontEmbeddingLicense {
    pub authorized: bool,
    pub identity: String,
}

#[pymethods]
impl PyFontEmbeddingLicense {
    #[new]
    #[pyo3(signature = (*, authorized, identity))]
    fn new(authorized: bool, identity: String) -> Self {
        Self {
            authorized,
            identity,
        }
    }
}

/// One embedded face as the package stores it.
#[pyclass(name = "EmbeddedFontFace", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyEmbeddedFontFace {
    kind: &'static str,
    relationship_id: String,
    part_name: Option<String>,
    format: &'static str,
    stored_len: usize,
    data: Option<std::sync::Arc<[u8]>>,
}

impl From<rpptx::EmbeddedFontFace> for PyEmbeddedFontFace {
    fn from(face: rpptx::EmbeddedFontFace) -> Self {
        Self {
            kind: embedded_font_kind_name(face.kind),
            relationship_id: face.relationship_id,
            part_name: face.part_name,
            format: match face.format {
                rpptx::EmbeddedFontFormat::Eot => "eot",
                rpptx::EmbeddedFontFormat::CompressedEot => "compressed_eot",
                rpptx::EmbeddedFontFormat::Sfnt => "sfnt",
                rpptx::EmbeddedFontFormat::Unknown => "unknown",
            },
            stored_len: face.stored_len,
            data: face.data,
        }
    }
}

#[pymethods]
impl PyEmbeddedFontFace {
    #[getter]
    fn kind(&self) -> &str {
        self.kind
    }

    #[getter]
    fn relationship_id(&self) -> &str {
        &self.relationship_id
    }

    #[getter]
    fn part_name(&self) -> Option<&str> {
        self.part_name.as_deref()
    }

    #[getter]
    fn format(&self) -> &str {
        self.format
    }

    #[getter]
    fn stored_len(&self) -> usize {
        self.stored_len
    }

    /// The TrueType or OpenType bytes, or `None` when the stored form is
    /// compressed or unrecognized.
    #[getter]
    fn data<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.data.as_deref().map(|data| PyBytes::new(py, data))
    }
}

/// One embedded font entry and its faces in schema order.
#[pyclass(name = "EmbeddedTypeface", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyEmbeddedTypeface {
    typeface: String,
    panose: Option<String>,
    pitch_family: Option<i8>,
    charset: Option<i8>,
    faces: Vec<PyEmbeddedFontFace>,
}

impl From<rpptx::EmbeddedTypeface> for PyEmbeddedTypeface {
    fn from(font: rpptx::EmbeddedTypeface) -> Self {
        Self {
            typeface: font.typeface,
            panose: font.panose,
            pitch_family: font.pitch_family,
            charset: font.charset,
            faces: font.faces.into_iter().map(Into::into).collect(),
        }
    }
}

#[pymethods]
impl PyEmbeddedTypeface {
    #[getter]
    fn typeface(&self) -> &str {
        &self.typeface
    }

    #[getter]
    fn panose(&self) -> Option<&str> {
        self.panose.as_deref()
    }

    #[getter]
    fn pitch_family(&self) -> Option<i8> {
        self.pitch_family
    }

    #[getter]
    fn charset(&self) -> Option<i8> {
        self.charset
    }

    #[getter]
    fn faces<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.faces.iter().cloned())
    }
}

fn embedded_font_kind_name(kind: rpptx::EmbeddedFontKind) -> &'static str {
    match kind {
        rpptx::EmbeddedFontKind::Regular => "regular",
        rpptx::EmbeddedFontKind::Bold => "bold",
        rpptx::EmbeddedFontKind::Italic => "italic",
        rpptx::EmbeddedFontKind::BoldItalic => "bold_italic",
    }
}

fn embedded_font_kind(name: &str) -> PyResult<rpptx::EmbeddedFontKind> {
    rpptx::EmbeddedFontKind::ALL
        .into_iter()
        .find(|kind| embedded_font_kind_name(*kind) == name)
        .ok_or_else(|| {
            PyValueError::new_err(format!(
                "font face kind must be regular, bold, italic or bold_italic, not {name:?}"
            ))
        })
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
}

impl PyPresentation {
    fn from_presentation(inner: rpptx::Presentation) -> Self {
        Self {
            inner,
            revisions: RevisionCounter::new(),
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

    fn to_pdf<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        py.detach(|| self.inner.to_pdf_deterministic())
            .map(|bytes| PyBytes::new(py, &bytes))
            .map_err(|error| rpptx_to_pyerr(py, error))
    }

    #[pyo3(signature = (slide_index, dpi = 150.0))]
    fn render_slide_to_png<'py>(
        &self,
        py: Python<'py>,
        slide_index: usize,
        dpi: f64,
    ) -> PyResult<Option<Bound<'py, PyBytes>>> {
        py.detach(|| self.inner.slide_png_deterministic(slide_index, dpi))
            .map(|bytes| bytes.map(|bytes| PyBytes::new(py, &bytes)))
            .map_err(|error| rpptx_to_pyerr(py, error))
    }

    #[pyo3(signature = (dpi = 150.0))]
    fn render_all_slides<'py>(&self, py: Python<'py>, dpi: f64) -> PyResult<Bound<'py, PyList>> {
        let slides = py
            .detach(|| self.inner.slide_pngs_deterministic(dpi))
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        PyList::new(py, slides.iter().map(|slide| PyBytes::new(py, slide)))
    }

    #[pyo3(signature = (*, width_factor = 1.0))]
    fn text_layout<'py>(
        &self,
        py: Python<'py>,
        width_factor: f64,
    ) -> PyResult<Bound<'py, PyTuple>> {
        let frames = py
            .detach(|| self.inner.text_layout_deterministic(width_factor))
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        PyTuple::new(py, frames.iter().map(PyTextFrameLayout::from))
    }

    fn to_notes_pdf<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        py.detach(|| self.inner.to_notes_pdf_deterministic())
            .map(|bytes| PyBytes::new(py, &bytes))
            .map_err(|error| rpptx_to_pyerr(py, error))
    }

    #[pyo3(signature = (dpi = 150.0))]
    fn render_all_notes<'py>(&self, py: Python<'py>, dpi: f64) -> PyResult<Bound<'py, PyList>> {
        let notes = py
            .detach(|| self.inner.notes_page_pngs_deterministic(dpi))
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        PyList::new(py, notes.iter().map(|page| PyBytes::new(py, page)))
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
    fn embedded_fonts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(
            py,
            self.inner
                .embedded_fonts()
                .into_iter()
                .map(PyEmbeddedTypeface::from),
        )
    }

    /// Embeds one face of a TrueType or OpenType font from a path, bytes, or
    /// a binary file-like object. The license states the identity and
    /// authorizes the embedding explicitly.
    #[pyo3(signature = (typeface, font, kind = "regular", *, license))]
    fn embed_font(
        &mut self,
        py: Python<'_>,
        typeface: &str,
        font: &Bound<'_, PyAny>,
        kind: &str,
        license: PyRef<'_, PyFontEmbeddingLicense>,
    ) -> PyResult<()> {
        let kind = embedded_font_kind(kind)?;
        let (data, _) = image_bytes(font)?;
        let font = rpptx::EmbeddedFont::new(
            kind,
            data,
            rpptx::FontEmbeddingLicense::new(license.authorized, license.identity.clone()),
        );
        drop(license);
        py.detach(|| self.inner.embed_font(typeface, font))
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        self.revisions.bump();
        Ok(())
    }

    /// Removes one embedded face and returns it, or `None` when absent.
    fn remove_embedded_font(
        &mut self,
        py: Python<'_>,
        typeface: &str,
        kind: &str,
    ) -> PyResult<Option<PyEmbeddedFontFace>> {
        let kind = embedded_font_kind(kind)?;
        let removed = self
            .inner
            .remove_embedded_font(typeface, kind)
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        if removed.is_some() {
            self.revisions.bump();
        }
        Ok(removed.map(Into::into))
    }

    /// Removes one embedded font with every face and returns it, or `None`.
    fn remove_font(
        &mut self,
        py: Python<'_>,
        typeface: &str,
    ) -> PyResult<Option<PyEmbeddedTypeface>> {
        let removed = self
            .inner
            .remove_font(typeface)
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        if removed.is_some() {
            self.revisions.bump();
        }
        Ok(removed.map(Into::into))
    }

    #[getter]
    fn embed_true_type_fonts(&self) -> bool {
        self.inner.embed_true_type_fonts()
    }

    #[setter(embed_true_type_fonts)]
    fn set_embed_true_type_fonts(&mut self, value: bool) {
        self.inner.set_embed_true_type_fonts(value);
    }

    #[getter]
    fn save_subset_fonts(&self) -> bool {
        self.inner.save_subset_fonts()
    }

    #[setter(save_subset_fonts)]
    fn set_save_subset_fonts(&mut self, value: bool) {
        self.inner.set_save_subset_fonts(value);
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
