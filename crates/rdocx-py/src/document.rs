use std::collections::BTreeMap;
use std::path::PathBuf;

use oxml_py_support::{PathSeg, RevisionCounter, StaleElementError};
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyList, PyTuple};
use smallvec::smallvec;

use crate::paragraph::{PyParagraph, PyParagraphCollection};
use crate::rdocx_to_pyerr;
use crate::table::{PyTable, PyTableCollection};

#[pyclass(name = "RunPosition", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PyRunPosition {
    pub body_index: usize,
    pub run_index: usize,
}

#[pymethods]
impl PyRunPosition {
    #[new]
    #[pyo3(signature = (*, body_index, run_index))]
    fn new(body_index: usize, run_index: usize) -> Self {
        Self {
            body_index,
            run_index,
        }
    }
}

impl From<PyRunPosition> for rdocx::RunPosition {
    fn from(value: PyRunPosition) -> Self {
        Self {
            body_index: value.body_index,
            run_index: value.run_index,
        }
    }
}

#[pyclass(name = "RunRange", frozen, eq, skip_from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PyRunRange {
    start: PyRunPosition,
    end: PyRunPosition,
}

#[pymethods]
impl PyRunRange {
    #[new]
    #[pyo3(signature = (*, start, end))]
    fn new(start: PyRef<'_, PyRunPosition>, end: PyRef<'_, PyRunPosition>) -> Self {
        Self {
            start: *start,
            end: *end,
        }
    }

    #[getter]
    fn start(&self) -> PyRunPosition {
        self.start
    }

    #[getter]
    fn end(&self) -> PyRunPosition {
        self.end
    }
}

impl From<PyRunRange> for rdocx::RunRange {
    fn from(value: PyRunRange) -> Self {
        Self {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

#[pyclass(name = "Comment", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyComment {
    pub id: i32,
    pub author: Option<String>,
    pub initials: Option<String>,
    pub date: Option<String>,
    pub text: String,
    pub parent_id: Option<i32>,
    pub resolved: bool,
}

#[pymethods]
impl PyComment {
    #[new]
    #[pyo3(signature = (*, id, author, initials, date, text, parent_id, resolved))]
    fn new(
        id: i32,
        author: Option<String>,
        initials: Option<String>,
        date: Option<String>,
        text: String,
        parent_id: Option<i32>,
        resolved: bool,
    ) -> Self {
        Self {
            id,
            author,
            initials,
            date,
            text,
            parent_id,
            resolved,
        }
    }
}

#[pyclass(
    name = "ComparisonDiagnostic",
    frozen,
    get_all,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyComparisonDiagnostic {
    pub location: String,
    pub message: String,
}

#[pymethods]
impl PyComparisonDiagnostic {
    #[new]
    #[pyo3(signature = (*, location, message))]
    fn new(location: String, message: String) -> Self {
        Self { location, message }
    }
}

#[pyclass(name = "BoundingBox", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, Copy, PartialEq)]
pub struct PyBoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[pymethods]
impl PyBoundingBox {
    #[new]
    #[pyo3(signature = (*, x, y, width, height))]
    fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[pyclass(name = "LayoutFragment", frozen, eq, skip_from_py_object)]
#[derive(Clone, Copy, PartialEq)]
pub struct PyLayoutFragment {
    body_index: usize,
    physical_page: usize,
    displayed_page: usize,
    bounds: PyBoundingBox,
}

#[pymethods]
impl PyLayoutFragment {
    #[new]
    #[pyo3(signature = (*, body_index, physical_page, displayed_page, bounds))]
    fn new(
        body_index: usize,
        physical_page: usize,
        displayed_page: usize,
        bounds: PyRef<'_, PyBoundingBox>,
    ) -> Self {
        Self {
            body_index,
            physical_page,
            displayed_page,
            bounds: *bounds,
        }
    }

    #[getter]
    fn body_index(&self) -> usize {
        self.body_index
    }

    #[getter]
    fn physical_page(&self) -> usize {
        self.physical_page
    }

    #[getter]
    fn displayed_page(&self) -> usize {
        self.displayed_page
    }

    #[getter]
    fn bounds(&self) -> PyBoundingBox {
        self.bounds
    }
}

#[pyclass(name = "LayoutPage", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, Copy, PartialEq)]
pub struct PyLayoutPage {
    pub page_number: usize,
    pub displayed_page_number: usize,
    pub width: f64,
    pub height: f64,
}

#[pymethods]
impl PyLayoutPage {
    #[new]
    #[pyo3(signature = (*, page_number, displayed_page_number, width, height))]
    fn new(page_number: usize, displayed_page_number: usize, width: f64, height: f64) -> Self {
        Self {
            page_number,
            displayed_page_number,
            width,
            height,
        }
    }
}

#[pyclass(name = "TocRebuildReport", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyTocRebuildReport {
    entry_count: usize,
    bookmark_count: usize,
    diagnostics: Vec<String>,
}

#[pyclass(
    name = "LayoutBackedFieldUpdateReport",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyLayoutBackedFieldUpdateReport {
    page_fields: usize,
    num_pages_fields: usize,
    page_reference_fields: usize,
    diagnostics: Vec<String>,
}

#[pyclass(name = "Revision", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyRevision {
    pub id: i32,
    pub author: String,
    pub timestamp: Option<String>,
    pub kind: String,
}

#[pymethods]
impl PyRevision {
    #[new]
    #[pyo3(signature = (*, id, author, timestamp, kind))]
    fn new(id: i32, author: String, timestamp: Option<String>, kind: String) -> Self {
        Self {
            id,
            author,
            timestamp,
            kind,
        }
    }
}

fn revision_kind_name(kind: rdocx::RevisionKind) -> &'static str {
    match kind {
        rdocx::RevisionKind::Insertion => "insertion",
        rdocx::RevisionKind::Deletion => "deletion",
        rdocx::RevisionKind::MoveFrom => "move_from",
        rdocx::RevisionKind::MoveTo => "move_to",
        rdocx::RevisionKind::RunPropertyChange => "run_property_change",
        rdocx::RevisionKind::ParagraphPropertyChange => "paragraph_property_change",
        rdocx::RevisionKind::TablePropertyChange => "table_property_change",
        rdocx::RevisionKind::SectionPropertyChange => "section_property_change",
    }
}

#[pyclass(name = "Story", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyStory {
    pub kind: String,
    pub part_name: String,
    pub owner_index: usize,
}

#[pymethods]
impl PyStory {
    #[new]
    #[pyo3(signature = (*, kind, part_name, owner_index))]
    fn new(kind: String, part_name: String, owner_index: usize) -> Self {
        Self {
            kind,
            part_name,
            owner_index,
        }
    }
}

#[pyclass(name = "StoryItem", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyStoryItem {
    story: PyStory,
    kind: String,
    index_path: Vec<usize>,
    direct_body_index: Option<usize>,
    text: Option<String>,
    xml: Vec<u8>,
    revision: u64,
}

#[pymethods]
impl PyStoryItem {
    #[new]
    #[pyo3(signature = (*, story, kind, index_path, text, xml=None, direct_body_index=None, revision=0))]
    fn new(
        story: PyRef<'_, PyStory>,
        kind: String,
        index_path: Vec<usize>,
        text: Option<String>,
        xml: Option<&[u8]>,
        direct_body_index: Option<usize>,
        revision: u64,
    ) -> Self {
        Self {
            story: story.clone(),
            kind,
            index_path,
            direct_body_index,
            text,
            xml: xml.unwrap_or_default().to_vec(),
            revision,
        }
    }

    #[getter]
    fn story(&self) -> PyStory {
        self.story.clone()
    }

    #[getter]
    fn kind(&self) -> &str {
        &self.kind
    }

    #[getter]
    fn index_path<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.index_path.iter().copied())
    }

    #[getter]
    fn direct_body_index(&self) -> Option<usize> {
        self.direct_body_index
    }

    #[getter]
    fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    #[getter]
    fn xml<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.xml)
    }

    #[getter]
    fn revision(&self) -> u64 {
        self.revision
    }
}

#[pyclass(name = "StoryRunPosition", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyStoryRunPosition {
    item: PyStoryItem,
    run_index: usize,
}

#[pymethods]
impl PyStoryRunPosition {
    #[new]
    #[pyo3(signature = (*, item, run_index))]
    fn new(item: PyRef<'_, PyStoryItem>, run_index: usize) -> Self {
        Self {
            item: item.clone(),
            run_index,
        }
    }

    #[getter]
    fn item(&self) -> PyStoryItem {
        self.item.clone()
    }

    #[getter]
    fn run_index(&self) -> usize {
        self.run_index
    }
}

#[pyclass(name = "StoryRunRange", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyStoryRunRange {
    start: PyStoryRunPosition,
    end: PyStoryRunPosition,
}

#[pymethods]
impl PyStoryRunRange {
    #[new]
    #[pyo3(signature = (*, start, end))]
    fn new(start: PyRef<'_, PyStoryRunPosition>, end: PyRef<'_, PyStoryRunPosition>) -> Self {
        Self {
            start: start.clone(),
            end: end.clone(),
        }
    }

    #[getter]
    fn start(&self) -> PyStoryRunPosition {
        self.start.clone()
    }

    #[getter]
    fn end(&self) -> PyStoryRunPosition {
        self.end.clone()
    }
}

#[pyclass(name = "ContentFragment", frozen, skip_from_py_object)]
pub struct PyContentFragment {
    inner: rdocx::ContentFragment,
}

#[pymethods]
impl PyContentFragment {
    #[getter]
    fn kind(&self) -> &'static str {
        story_item_kind_name(self.inner.kind())
    }
}

#[pyclass(name = "Hyperlink", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyHyperlink {
    story: PyStory,
    index_path: Vec<usize>,
    text: String,
    url: Option<String>,
    anchor: Option<String>,
    relationship_id: Option<String>,
}

#[pymethods]
impl PyHyperlink {
    #[new]
    #[pyo3(signature = (*, story, index_path, text, url, anchor, relationship_id))]
    fn new(
        story: PyRef<'_, PyStory>,
        index_path: Vec<usize>,
        text: String,
        url: Option<String>,
        anchor: Option<String>,
        relationship_id: Option<String>,
    ) -> Self {
        Self {
            story: story.clone(),
            index_path,
            text,
            url,
            anchor,
            relationship_id,
        }
    }

    #[getter]
    fn story(&self) -> PyStory {
        self.story.clone()
    }

    #[getter]
    fn index_path<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.index_path.iter().copied())
    }

    #[getter]
    fn text(&self) -> &str {
        &self.text
    }

    #[getter]
    fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    #[getter]
    fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    #[getter]
    fn relationship_id(&self) -> Option<&str> {
        self.relationship_id.as_deref()
    }
}

#[pyclass(name = "HeaderFooterVariant", frozen, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyHeaderFooterVariant {
    section_index: usize,
    kind: String,
    variant: String,
    story: Option<PyStory>,
    source_section: Option<usize>,
    inherited: bool,
}

#[pymethods]
impl PyHeaderFooterVariant {
    #[new]
    #[pyo3(signature = (*, section_index, kind, variant, story, source_section, inherited))]
    fn new(
        section_index: usize,
        kind: String,
        variant: String,
        story: Option<PyRef<'_, PyStory>>,
        source_section: Option<usize>,
        inherited: bool,
    ) -> Self {
        Self {
            section_index,
            kind,
            variant,
            story: story.map(|value| value.clone()),
            source_section,
            inherited,
        }
    }

    #[getter]
    fn section_index(&self) -> usize {
        self.section_index
    }

    #[getter]
    fn kind(&self) -> &str {
        &self.kind
    }

    #[getter]
    fn variant(&self) -> &str {
        &self.variant
    }

    #[getter]
    fn story(&self) -> Option<PyStory> {
        self.story.clone()
    }

    #[getter]
    fn source_section(&self) -> Option<usize> {
        self.source_section
    }

    #[getter]
    fn inherited(&self) -> bool {
        self.inherited
    }
}

#[pyclass(name = "Section", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PySection {
    pub ordinal: usize,
    pub is_final: bool,
    pub orientation: Option<String>,
    pub page_width: Option<i64>,
    pub page_height: Option<i64>,
    pub margin_top: Option<i64>,
    pub margin_right: Option<i64>,
    pub margin_bottom: Option<i64>,
    pub margin_left: Option<i64>,
    pub gutter: Option<i64>,
    pub column_count: Option<u32>,
    pub column_spacing: Option<i64>,
    pub page_number_start: Option<u32>,
    pub header_distance: Option<i64>,
    pub footer_distance: Option<i64>,
    pub different_first_page: Option<bool>,
    pub break_type: Option<String>,
}

#[pymethods]
impl PySection {
    #[new]
    #[pyo3(signature = (*, ordinal, is_final, orientation, page_width, page_height, margin_top, margin_right, margin_bottom, margin_left, gutter, column_count, column_spacing, page_number_start, header_distance, footer_distance, different_first_page, break_type))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        ordinal: usize,
        is_final: bool,
        orientation: Option<String>,
        page_width: Option<i64>,
        page_height: Option<i64>,
        margin_top: Option<i64>,
        margin_right: Option<i64>,
        margin_bottom: Option<i64>,
        margin_left: Option<i64>,
        gutter: Option<i64>,
        column_count: Option<u32>,
        column_spacing: Option<i64>,
        page_number_start: Option<u32>,
        header_distance: Option<i64>,
        footer_distance: Option<i64>,
        different_first_page: Option<bool>,
        break_type: Option<String>,
    ) -> Self {
        Self {
            ordinal,
            is_final,
            orientation,
            page_width,
            page_height,
            margin_top,
            margin_right,
            margin_bottom,
            margin_left,
            gutter,
            column_count,
            column_spacing,
            page_number_start,
            header_distance,
            footer_distance,
            different_first_page,
            break_type,
        }
    }
}

#[pyclass(name = "Style", frozen, get_all, eq, skip_from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyStyle {
    pub style_id: String,
    pub name: Option<String>,
    pub based_on: Option<String>,
    pub style_type: String,
    pub linked_style: Option<String>,
    pub next_style: Option<String>,
    pub priority: Option<u32>,
    pub auto_redefine: Option<bool>,
    pub hidden: Option<bool>,
    pub semi_hidden: Option<bool>,
    pub unhide_when_used: Option<bool>,
    pub quick_format: Option<bool>,
    pub locked: Option<bool>,
    pub is_default: bool,
}

#[pymethods]
impl PyStyle {
    #[new]
    #[pyo3(signature = (*, style_id, name, based_on, style_type, linked_style, next_style, priority, auto_redefine, hidden, semi_hidden, unhide_when_used, quick_format, locked, is_default))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        style_id: String,
        name: Option<String>,
        based_on: Option<String>,
        style_type: String,
        linked_style: Option<String>,
        next_style: Option<String>,
        priority: Option<u32>,
        auto_redefine: Option<bool>,
        hidden: Option<bool>,
        semi_hidden: Option<bool>,
        unhide_when_used: Option<bool>,
        quick_format: Option<bool>,
        locked: Option<bool>,
        is_default: bool,
    ) -> Self {
        Self {
            style_id,
            name,
            based_on,
            style_type,
            linked_style,
            next_style,
            priority,
            auto_redefine,
            hidden,
            semi_hidden,
            unhide_when_used,
            quick_format,
            locked,
            is_default,
        }
    }
}

#[pymethods]
impl PyTocRebuildReport {
    #[new]
    #[pyo3(signature = (*, entry_count, bookmark_count, diagnostics))]
    fn new(entry_count: usize, bookmark_count: usize, diagnostics: Vec<String>) -> Self {
        Self {
            entry_count,
            bookmark_count,
            diagnostics,
        }
    }

    #[getter]
    fn entry_count(&self) -> usize {
        self.entry_count
    }

    #[getter]
    fn bookmark_count(&self) -> usize {
        self.bookmark_count
    }

    #[getter]
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.diagnostics)
    }

    #[getter]
    fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }
}

#[pymethods]
impl PyLayoutBackedFieldUpdateReport {
    #[new]
    #[pyo3(signature = (*, page_fields, num_pages_fields, page_reference_fields, diagnostics))]
    fn new(
        page_fields: usize,
        num_pages_fields: usize,
        page_reference_fields: usize,
        diagnostics: Vec<String>,
    ) -> Self {
        Self {
            page_fields,
            num_pages_fields,
            page_reference_fields,
            diagnostics,
        }
    }

    #[getter]
    fn page_fields(&self) -> usize {
        self.page_fields
    }

    #[getter]
    fn num_pages_fields(&self) -> usize {
        self.num_pages_fields
    }

    #[getter]
    fn page_reference_fields(&self) -> usize {
        self.page_reference_fields
    }

    #[getter]
    fn updated_count(&self) -> usize {
        self.page_fields + self.num_pages_fields + self.page_reference_fields
    }

    #[getter]
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.diagnostics)
    }

    #[getter]
    fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }
}

#[pyclass(name = "Document")]
pub struct PyDocument {
    pub(crate) inner: rdocx::Document,
    pub(crate) revisions: RevisionCounter,
}

impl PyDocument {
    fn from_document(inner: rdocx::Document) -> Self {
        Self {
            inner,
            revisions: RevisionCounter::new(),
        }
    }

    /// The live native owner that a `Story` snapshot names.
    ///
    /// A snapshot carries no fingerprint, so it resolves by kind, part name
    /// and owner index against the document as it is now.
    fn native_story(&self, py: Python<'_>, story: &PyStory) -> PyResult<rdocx::StoryId> {
        self.inner
            .stories()
            .map_err(|error| rdocx_to_pyerr(py, error))?
            .into_iter()
            .find(|candidate| story_snapshot(candidate) == *story)
            .ok_or_else(|| {
                rdocx_to_pyerr(
                    py,
                    rdocx::Error::Other(format!(
                        "document has no {} story {} at owner index {}",
                        story.kind, story.part_name, story.owner_index
                    )),
                )
            })
    }

    fn body_story(&self, py: Python<'_>) -> PyResult<rdocx::StoryId> {
        self.inner
            .stories()
            .map_err(|error| rdocx_to_pyerr(py, error))?
            .into_iter()
            .find(|story| story.kind() == rdocx::StoryKind::Body)
            .ok_or_else(|| {
                rdocx_to_pyerr(
                    py,
                    rdocx::Error::Other("document body story is missing".to_owned()),
                )
            })
    }

    fn native_location(
        &self,
        py: Python<'_>,
        item: &PyStoryItem,
    ) -> PyResult<rdocx::ContentLocation> {
        let story = self.native_story(py, &item.story)?;
        if item.revision != self.revisions.current() {
            return Err(crate::stale_to_pyerr(
                py,
                StaleElementError {
                    element_kind: "story item".to_owned(),
                    captured_revision: item.revision,
                    current_revision: self.revisions.current(),
                    recovery_hint: "Re-fetch it with document.story_items.".to_owned(),
                },
            ));
        }
        Ok(rdocx::ContentLocation::new(
            story,
            story_item_kind_from_name(&item.kind)?,
            item.index_path.clone(),
        ))
    }

    fn body_location(&self, py: Python<'_>, index: usize) -> PyResult<rdocx::ContentLocation> {
        let content_count = self.inner.content_count();
        if index > content_count {
            return Err(PyIndexError::new_err("content index out of range"));
        }
        let story = self.body_story(py)?;
        if index == content_count {
            return Ok(rdocx::ContentLocation::end(story));
        }
        for item in self
            .inner
            .story_items(&story)
            .map_err(|error| rdocx_to_pyerr(py, error))?
        {
            let direct_body_index = item
                .direct_body_index()
                .map_err(|error| rdocx_to_pyerr(py, error))?;
            if direct_body_index == Some(index) && item.location().index_path().len() == 1 {
                return Ok(item.location().clone());
            }
        }
        Err(rdocx_to_pyerr(
            py,
            rdocx::Error::Other(format!(
                "direct body content at index {index} has no checked location"
            )),
        ))
    }

    fn story_item_snapshot(
        &self,
        py: Python<'_>,
        location: &rdocx::ContentLocation,
    ) -> PyResult<PyStoryItem> {
        let item = self
            .inner
            .story_item_snapshots()
            .map_err(|error| rdocx_to_pyerr(py, error))?
            .into_iter()
            .find(|item| item.location() == location)
            .ok_or_else(|| PyIndexError::new_err("inserted story item was not found"))?;
        Ok(PyStoryItem {
            story: story_snapshot(item.location().story()),
            kind: story_item_kind_name(item.location().item_kind()).to_owned(),
            index_path: item.location().index_path().to_vec(),
            direct_body_index: item.direct_body_index(),
            text: item.text().map(str::to_owned),
            xml: item.xml().to_vec(),
            revision: self.revisions.current(),
        })
    }

    fn direct_content_index(
        slf: &Py<Self>,
        py: Python<'_>,
        content: &Bound<'_, PyAny>,
    ) -> PyResult<usize> {
        if let Ok(paragraph) = content.cast::<PyParagraph>() {
            let paragraph = paragraph.borrow();
            if !paragraph.belongs_to(py, slf) {
                return Err(PyValueError::new_err(
                    "content handle belongs to a different document",
                ));
            }
            let paragraph_index = match paragraph.validate(py)? {
                crate::paragraph::ParagraphLocation::Body(index) => index,
                crate::paragraph::ParagraphLocation::Cell { .. } => {
                    return Err(PyValueError::new_err(
                        "content handle is not a direct body child",
                    ));
                }
            };
            return slf
                .borrow(py)
                .inner
                .content_index_of_paragraph(paragraph_index)
                .ok_or_else(|| PyValueError::new_err("content handle is not a direct body child"));
        }
        if let Ok(table) = content.cast::<PyTable>() {
            let table = table.borrow();
            if !table.belongs_to(py, slf) {
                return Err(PyValueError::new_err(
                    "content handle belongs to a different document",
                ));
            }
            let table_index = table.validate(py)?;
            return slf
                .borrow(py)
                .inner
                .content_index_of_table(table_index)
                .ok_or_else(|| PyValueError::new_err("content handle is not a direct body child"));
        }
        Err(PyTypeError::new_err(
            "content must be a Paragraph or Table handle",
        ))
    }

    /// Run a native mutation that reports how many things it changed.
    ///
    /// The GIL is released while it runs, and live handles are staled only
    /// when the count is nonzero.
    fn counted_mutation<F>(&mut self, py: Python<'_>, mutation: F) -> PyResult<usize>
    where
        F: FnOnce(&mut rdocx::Document) -> rdocx::Result<usize> + Send,
    {
        let count = py
            .detach(|| mutation(&mut self.inner))
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        if count > 0 {
            self.revisions.bump();
        }
        Ok(count)
    }
}

fn story_snapshot(story: &rdocx::StoryId) -> PyStory {
    PyStory {
        kind: match story.kind() {
            rdocx::StoryKind::Body => "body",
            rdocx::StoryKind::TableCell => "table_cell",
            rdocx::StoryKind::Header => "header",
            rdocx::StoryKind::Footer => "footer",
            rdocx::StoryKind::Footnote => "footnote",
            rdocx::StoryKind::Endnote => "endnote",
            rdocx::StoryKind::Comment => "comment",
            rdocx::StoryKind::TextBox => "text_box",
            _ => "unknown",
        }
        .to_owned(),
        part_name: story.part_name().to_owned(),
        owner_index: story.owner_index(),
    }
}

fn story_item_kind_name(kind: rdocx::StoryItemKind) -> &'static str {
    match kind {
        rdocx::StoryItemKind::Paragraph => "paragraph",
        rdocx::StoryItemKind::Table => "table",
        rdocx::StoryItemKind::ContentControl => "content_control",
        rdocx::StoryItemKind::Field => "field",
        rdocx::StoryItemKind::Drawing => "drawing",
        rdocx::StoryItemKind::PreservedNode => "preserved_node",
        _ => "unknown",
    }
}

fn story_item_kind_from_name(name: &str) -> PyResult<rdocx::StoryItemKind> {
    match name {
        "paragraph" => Ok(rdocx::StoryItemKind::Paragraph),
        "table" => Ok(rdocx::StoryItemKind::Table),
        "content_control" => Ok(rdocx::StoryItemKind::ContentControl),
        "field" => Ok(rdocx::StoryItemKind::Field),
        "drawing" => Ok(rdocx::StoryItemKind::Drawing),
        "preserved_node" => Ok(rdocx::StoryItemKind::PreservedNode),
        _ => Err(PyValueError::new_err(format!(
            "unsupported story item kind {name:?}"
        ))),
    }
}

fn section_snapshot(section: rdocx::SectionRef<'_>) -> PySection {
    let (page_width, page_height) = section.page_size().map_or((None, None), |(width, height)| {
        (Some(width.to_emu()), Some(height.to_emu()))
    });
    let (margin_top, margin_right, margin_bottom, margin_left) =
        section
            .margins()
            .map_or((None, None, None, None), |(top, right, bottom, left)| {
                (
                    Some(top.to_emu()),
                    Some(right.to_emu()),
                    Some(bottom.to_emu()),
                    Some(left.to_emu()),
                )
            });
    let (column_count, column_spacing) =
        section.columns().map_or((None, None), |(count, spacing)| {
            (Some(count), Some(spacing.to_emu()))
        });
    let (header_distance, footer_distance) = section
        .header_footer_distance()
        .map_or((None, None), |(header, footer)| {
            (Some(header.to_emu()), Some(footer.to_emu()))
        });
    PySection {
        ordinal: section.ordinal(),
        is_final: section.is_final(),
        orientation: section.orientation().map(|value| value.to_str().to_owned()),
        page_width,
        page_height,
        margin_top,
        margin_right,
        margin_bottom,
        margin_left,
        gutter: section.gutter().map(rdocx::Length::to_emu),
        column_count,
        column_spacing,
        page_number_start: section.page_number_start(),
        header_distance,
        footer_distance,
        different_first_page: section.different_first_page(),
        break_type: section.break_type().map(|value| value.to_str().to_owned()),
    }
}

fn style_snapshot(style: rdocx::style::Style<'_>) -> PyStyle {
    PyStyle {
        style_id: style.style_id().to_owned(),
        name: style.name().map(str::to_owned),
        based_on: style.based_on().map(str::to_owned),
        style_type: style.style_type().to_str().to_owned(),
        linked_style: style.linked_style().map(str::to_owned),
        next_style: style.next_style().map(str::to_owned),
        priority: style.priority(),
        auto_redefine: style.auto_redefine(),
        hidden: style.hidden(),
        semi_hidden: style.semi_hidden(),
        unhide_when_used: style.unhide_when_used(),
        quick_format: style.quick_format(),
        locked: style.locked(),
        is_default: style.is_default(),
    }
}

#[pymethods]
impl PyDocument {
    #[new]
    #[pyo3(signature = (path = None))]
    fn new(path: Option<PathBuf>, py: Python<'_>) -> PyResult<Self> {
        match path {
            Some(path) => rdocx::Document::open(path)
                .map(Self::from_document)
                .map_err(|error| rdocx_to_pyerr(py, error)),
            None => Ok(Self::from_document(rdocx::Document::new())),
        }
    }

    #[staticmethod]
    fn open(path: PathBuf, py: Python<'_>) -> PyResult<Self> {
        rdocx::Document::open(path)
            .map(Self::from_document)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[staticmethod]
    fn from_bytes(bytes: &[u8], py: Python<'_>) -> PyResult<Self> {
        rdocx::Document::from_bytes(bytes)
            .map(Self::from_document)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    fn save(&mut self, path: PathBuf, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.inner.save(path))
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[pyo3(name = "to_bytes")]
    fn serialize<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        py.detach(|| self.inner.to_bytes())
            .map(|bytes| PyBytes::new(py, &bytes))
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[getter]
    fn update_fields_on_open(&self) -> Option<bool> {
        self.inner.update_fields_on_open()
    }

    #[setter]
    fn set_update_fields_on_open(&mut self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.inner
            .set_update_fields_on_open(value)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    fn image_data<'py>(
        &self,
        py: Python<'py>,
        relationship_id: &str,
    ) -> Option<Bound<'py, PyBytes>> {
        self.inner
            .image_data(relationship_id)
            .map(|bytes| PyBytes::new(py, &bytes))
    }

    fn replace_image(
        &mut self,
        py: Python<'_>,
        relationship_id: &str,
        data: &[u8],
    ) -> PyResult<()> {
        // No content moves, so live handles stay valid.
        self.inner
            .replace_image(relationship_id, data)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    fn replace_image_for_story(
        &mut self,
        py: Python<'_>,
        story: PyRef<'_, PyStory>,
        relationship_id: &str,
        data: &[u8],
    ) -> PyResult<()> {
        let story = self.native_story(py, &story)?;
        self.inner
            .replace_image_for_story(&story, relationship_id, data)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    fn split_run(
        &mut self,
        py: Python<'_>,
        body_index: usize,
        run_index: usize,
        character_offset: usize,
    ) -> PyResult<usize> {
        let before = self
            .inner
            .paragraph(body_index)
            .map(|paragraph| paragraph.run_count());
        let boundary = self
            .inner
            .split_run(body_index, run_index, character_offset)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        let after = self
            .inner
            .paragraph(body_index)
            .map(|paragraph| paragraph.run_count());
        if before != after {
            self.revisions.bump();
        }
        Ok(boundary)
    }

    fn to_pdf<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        py.detach(|| self.inner.to_pdf())
            .map(|bytes| PyBytes::new(py, &bytes))
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[pyo3(signature = (page_index, dpi = 150.0))]
    fn render_page_to_png<'py>(
        &self,
        py: Python<'py>,
        page_index: usize,
        dpi: f64,
    ) -> PyResult<Option<Bound<'py, PyBytes>>> {
        py.detach(|| self.inner.render_page_to_png(page_index, dpi))
            .map(|bytes| bytes.map(|bytes| PyBytes::new(py, &bytes)))
            .map_err(|error| rdocx_to_pyerr(py, error))
    }

    #[pyo3(signature = (dpi = 150.0))]
    fn render_all_pages<'py>(&self, py: Python<'py>, dpi: f64) -> PyResult<Bound<'py, PyList>> {
        let pages = py
            .detach(|| self.inner.render_all_pages(dpi))
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        PyList::new(py, pages.iter().map(|page| PyBytes::new(py, page)))
    }

    #[pyo3(signature = (*, dpi = 150.0, format = "png", quality = 90, transparent = false, pages = None))]
    fn render_pages(
        &self,
        py: Python<'_>,
        dpi: f64,
        format: &str,
        quality: u8,
        transparent: bool,
        pages: Option<Vec<usize>>,
    ) -> PyResult<Py<PyAny>> {
        let rendered = py
            .detach(|| {
                let format = parse_raster_format(format, quality, transparent)?;
                let selected = match pages {
                    Some(pages) => pages,
                    None => (0..self.inner.layout()?.layout.pages.len()).collect(),
                };
                self.inner
                    .render_pages(&selected, rdocx::RasterOptions { dpi, format })
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        match rendered {
            rdocx::RasterOutput::SeparatePages(pages) => {
                let list = PyList::new(py, pages.iter().map(|page| PyBytes::new(py, page)))?;
                Ok(list.into_any().unbind())
            }
            rdocx::RasterOutput::MultiPageTiff(tiff) => {
                Ok(PyBytes::new(py, &tiff).into_any().unbind())
            }
        }
    }

    fn compare<'py>(
        &mut self,
        edited: &PyDocument,
        author: &str,
        timestamp: &str,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyTuple>> {
        let (diagnostics, changed) = py
            .detach(|| {
                let before = self.inner.to_bytes()?;
                let diagnostics = self.inner.compare(&edited.inner, author, timestamp)?;
                let changed = self.inner.to_bytes()? != before;
                Ok::<_, rdocx::Error>((diagnostics, changed))
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        if changed {
            self.revisions.bump();
        }
        PyTuple::new(
            py,
            diagnostics.into_iter().map(|item| PyComparisonDiagnostic {
                location: item.location,
                message: item.message,
            }),
        )
    }

    #[getter]
    fn comments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(
            py,
            self.inner.comments().into_iter().map(|comment| PyComment {
                id: comment.id(),
                author: comment.author().map(str::to_owned),
                initials: comment.initials().map(str::to_owned),
                date: comment.date().map(str::to_owned),
                text: comment.text(),
                parent_id: comment.parent_id(),
                resolved: comment.resolved(),
            }),
        )
    }

    #[getter]
    fn sections<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let sections = self
            .inner
            .sections()
            .map(section_snapshot)
            .collect::<Vec<_>>();
        PyTuple::new(py, sections)
    }

    #[getter]
    fn styles<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.inner.styles().into_iter().map(style_snapshot))
    }

    #[getter]
    fn stories<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let stories = self
            .inner
            .stories()
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        PyTuple::new(py, stories.iter().map(story_snapshot))
    }

    #[getter]
    fn story_items<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self
            .inner
            .story_item_snapshots()
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        let snapshots = items
            .into_iter()
            .map(|item| PyStoryItem {
                story: story_snapshot(item.location().story()),
                kind: story_item_kind_name(item.location().item_kind()).to_owned(),
                index_path: item.location().index_path().to_vec(),
                direct_body_index: item.direct_body_index(),
                text: item.text().map(str::to_owned),
                xml: item.xml().to_vec(),
                revision: self.revisions.current(),
            })
            .collect::<Vec<_>>();
        PyTuple::new(py, snapshots)
    }

    #[getter]
    fn header_footer_variants<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let mut snapshots = Vec::new();
        for section_index in 0..self.inner.section_count() {
            for (kind, kind_name) in [
                (rdocx::HeaderFooterKind::Header, "header"),
                (rdocx::HeaderFooterKind::Footer, "footer"),
            ] {
                for variant in [
                    rdocx::HdrFtrType::Default,
                    rdocx::HdrFtrType::First,
                    rdocx::HdrFtrType::Even,
                ] {
                    let resolved = self
                        .inner
                        .section_story(section_index, kind, variant)
                        .map_err(|error| rdocx_to_pyerr(py, error))?;
                    snapshots.push(PyHeaderFooterVariant {
                        section_index,
                        kind: kind_name.to_owned(),
                        variant: variant.to_str().to_owned(),
                        story: resolved.as_ref().map(|value| story_snapshot(value.story())),
                        source_section: resolved.as_ref().map(rdocx::SectionStory::source_section),
                        inherited: resolved.is_some_and(|value| value.is_inherited()),
                    });
                }
            }
        }
        PyTuple::new(py, snapshots)
    }

    #[getter]
    fn hyperlinks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let links = self
            .inner
            .story_link_snapshots()
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        let snapshots = links
            .into_iter()
            .map(|(location, link)| PyHyperlink {
                story: story_snapshot(location.story()),
                index_path: location.index_path().to_vec(),
                text: link.text,
                url: link.url,
                anchor: link.anchor,
                relationship_id: link.rel_id,
            })
            .collect::<Vec<_>>();
        PyTuple::new(py, snapshots)
    }

    fn set_header(&mut self, text: &str) {
        self.inner.set_header(text);
        self.revisions.bump();
    }

    fn set_footer(&mut self, text: &str) {
        self.inner.set_footer(text);
        self.revisions.bump();
    }

    fn set_story_text(
        &mut self,
        py: Python<'_>,
        item: PyRef<'_, PyStoryItem>,
        text: &str,
    ) -> PyResult<()> {
        let location = self.native_location(py, &item)?;
        py.detach(|| self.inner.set_story_text(&location, text))
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        self.revisions.bump();
        Ok(())
    }

    fn add_hyperlink_to_story(
        &mut self,
        py: Python<'_>,
        story: PyRef<'_, PyStory>,
        text: &str,
        url: &str,
    ) -> PyResult<()> {
        let story = self.native_story(py, &story)?;
        py.detach(|| self.inner.add_hyperlink_to_story(&story, text, url))
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        self.revisions.bump();
        Ok(())
    }

    #[pyo3(signature = (data, filename, width=None, height=None, *, after=None))]
    fn add_picture(
        &mut self,
        py: Python<'_>,
        data: &[u8],
        filename: &str,
        width: Option<i64>,
        height: Option<i64>,
        after: Option<PyRef<'_, PyStoryItem>>,
    ) -> PyResult<PyStoryItem> {
        let after = after
            .as_deref()
            .map(|item| self.native_location(py, item))
            .transpose()?;
        let story = match after.as_ref() {
            Some(location) => location.story().clone(),
            None => self.body_story(py)?,
        };
        let location = py
            .detach(|| {
                self.inner.insert_picture_to_story(
                    &story,
                    after.as_ref(),
                    data,
                    filename,
                    width.map(rdocx::Length::emu),
                    height.map(rdocx::Length::emu),
                )
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        self.revisions.bump();
        self.story_item_snapshot(py, &location)
    }

    #[pyo3(signature = (range, *, author, text, initials = None, date = None))]
    fn add_comment(
        &mut self,
        range: &Bound<'_, PyAny>,
        author: &str,
        text: &str,
        initials: Option<&str>,
        date: Option<&str>,
        py: Python<'_>,
    ) -> PyResult<i32> {
        let id = if let Ok(range) = range.cast::<PyRunRange>() {
            self.inner
                .add_comment_with_date((*range.borrow()).into(), author, initials, text, date)
        } else if let Ok(range) = range.cast::<PyStoryRunRange>() {
            let range = range.borrow();
            let start = rdocx::StoryRunPosition {
                location: self.native_location(py, &range.start.item)?,
                run_index: range.start.run_index,
            };
            let end = rdocx::StoryRunPosition {
                location: self.native_location(py, &range.end.item)?,
                run_index: range.end.run_index,
            };
            self.inner.add_story_comment_with_date(
                rdocx::StoryRunRange { start, end },
                author,
                initials,
                text,
                date,
            )
        } else {
            return Err(PyTypeError::new_err(
                "range must be a RunRange or StoryRunRange",
            ));
        }
        .map_err(|error| rdocx_to_pyerr(py, error))?;
        self.revisions.bump();
        Ok(id)
    }

    #[pyo3(signature = (parent_id, *, author, text, date = None))]
    fn reply_to(
        &mut self,
        parent_id: i32,
        author: &str,
        text: &str,
        date: Option<&str>,
        py: Python<'_>,
    ) -> PyResult<i32> {
        let id = self
            .inner
            .reply_to_with_date(parent_id, author, text, date)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        self.revisions.bump();
        Ok(id)
    }

    #[pyo3(signature = (id, *, resolved = true))]
    fn resolve_comment(&mut self, id: i32, resolved: bool, py: Python<'_>) -> PyResult<bool> {
        let updated = self
            .inner
            .resolve_comment(id, resolved)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        if updated {
            self.revisions.bump();
        }
        Ok(updated)
    }

    fn remove_comment(&mut self, id: i32, py: Python<'_>) -> PyResult<bool> {
        let removed = self
            .inner
            .remove_comment(id)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        if removed {
            self.revisions.bump();
        }
        Ok(removed)
    }

    fn layout<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let fragments = py
            .detach(|| {
                let layout = self.inner.layout_deterministic()?;
                let mut fragments = Vec::new();
                for body_index in 0..self.inner.content_count() {
                    let Some(body_fragments) = layout.body_layout_fragments(body_index) else {
                        continue;
                    };
                    fragments.extend(body_fragments.iter().map(|fragment| PyLayoutFragment {
                        body_index,
                        physical_page: fragment.physical_page,
                        displayed_page: fragment.displayed_page,
                        bounds: PyBoundingBox {
                            x: fragment.x,
                            y: fragment.y,
                            width: fragment.width,
                            height: fragment.height,
                        },
                    }));
                }
                Ok::<_, rdocx::Error>(fragments)
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        PyTuple::new(py, fragments)
    }

    fn layout_page(&self, page_index: usize, py: Python<'_>) -> PyResult<Option<PyLayoutPage>> {
        py.detach(|| {
            let layout = self.inner.layout_deterministic()?;
            Ok(layout
                .layout
                .pages
                .get(page_index)
                .map(|page| PyLayoutPage {
                    page_number: page.page_number,
                    displayed_page_number: page.displayed_page_number,
                    width: page.width,
                    height: page.height,
                }))
        })
        .map_err(|error| rdocx_to_pyerr(py, error))
    }

    fn rebuild_toc(&mut self, py: Python<'_>) -> PyResult<PyTocRebuildReport> {
        let (report, changed) = py
            .detach(|| {
                let before = self.inner.to_bytes()?;
                let report = self.inner.rebuild_toc()?;
                let changed = self.inner.to_bytes()? != before;
                Ok::<_, rdocx::Error>((report, changed))
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        if changed {
            self.revisions.bump();
        }
        Ok(PyTocRebuildReport {
            entry_count: report.entry_count,
            bookmark_count: report.bookmark_count,
            diagnostics: report.diagnostics,
        })
    }

    #[getter(revisions)]
    fn revision_snapshots<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(
            py,
            self.inner
                .revisions()
                .into_iter()
                .map(|revision| PyRevision {
                    id: revision.id(),
                    author: revision.author().to_owned(),
                    timestamp: revision.timestamp().map(str::to_owned),
                    kind: revision_kind_name(revision.kind()).to_owned(),
                }),
        )
    }

    fn accept_all(&mut self, py: Python<'_>) -> PyResult<usize> {
        self.counted_mutation(py, rdocx::Document::accept_all)
    }

    fn reject_all(&mut self, py: Python<'_>) -> PyResult<usize> {
        self.counted_mutation(py, rdocx::Document::reject_all)
    }

    fn accept_revisions_by_author(&mut self, py: Python<'_>, author: &str) -> PyResult<usize> {
        self.counted_mutation(py, |document| document.accept_revisions_by_author(author))
    }

    fn reject_revisions_by_author(&mut self, py: Python<'_>, author: &str) -> PyResult<usize> {
        self.counted_mutation(py, |document| document.reject_revisions_by_author(author))
    }

    #[pyo3(signature = (*, start, end))]
    fn accept_revisions_in_date_range(
        &mut self,
        py: Python<'_>,
        start: &str,
        end: &str,
    ) -> PyResult<usize> {
        self.counted_mutation(py, |document| {
            document.accept_revisions_in_date_range(start, end)
        })
    }

    #[pyo3(signature = (*, start, end))]
    fn reject_revisions_in_date_range(
        &mut self,
        py: Python<'_>,
        start: &str,
        end: &str,
    ) -> PyResult<usize> {
        self.counted_mutation(py, |document| {
            document.reject_revisions_in_date_range(start, end)
        })
    }

    fn accept_revision_id(&mut self, py: Python<'_>, id: i32) -> PyResult<usize> {
        self.counted_mutation(py, |document| document.accept_revision_id(id))
    }

    fn reject_revision_id(&mut self, py: Python<'_>, id: i32) -> PyResult<usize> {
        self.counted_mutation(py, |document| document.reject_revision_id(id))
    }

    fn try_replace_text(
        &mut self,
        py: Python<'_>,
        placeholder: &str,
        replacement: &str,
    ) -> PyResult<usize> {
        self.counted_mutation(py, |document| {
            document.try_replace_text(placeholder, replacement)
        })
    }

    fn replace_all_regex(
        &mut self,
        py: Python<'_>,
        patterns: Vec<(String, String)>,
    ) -> PyResult<usize> {
        self.counted_mutation(py, |document| document.replace_all_regex(&patterns))
    }

    #[pyo3(signature = (
        *,
        now = None,
        file_name = None,
        file_path = None,
        merge_fields = None,
        included_text = None,
        merge_record_number = None,
        merge_sequence_number = None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn update_fields(
        &mut self,
        py: Python<'_>,
        now: Option<&Bound<'_, PyAny>>,
        file_name: Option<String>,
        file_path: Option<String>,
        merge_fields: Option<BTreeMap<String, String>>,
        included_text: Option<BTreeMap<String, String>>,
        merge_record_number: Option<u32>,
        merge_sequence_number: Option<u32>,
    ) -> PyResult<usize> {
        // Read field by field: the abi3 build has no datetime accessors, and
        // the wall-clock values are used as given.
        let now = now
            .map(|now| {
                Ok::<_, PyErr>(rdocx::FieldDateTime {
                    year: now.getattr("year")?.extract()?,
                    month: now.getattr("month")?.extract()?,
                    day: now.getattr("day")?.extract()?,
                    hour: now.getattr("hour")?.extract()?,
                    minute: now.getattr("minute")?.extract()?,
                    second: now.getattr("second")?.extract()?,
                })
            })
            .transpose()?;
        let context = rdocx::FieldEvaluationContext {
            now,
            file_name,
            file_path,
            merge_fields: merge_fields.unwrap_or_default(),
            included_text: included_text.unwrap_or_default(),
            merge_record_number,
            merge_sequence_number,
        };
        self.counted_mutation(py, |document| document.update_fields(&context))
    }

    fn update_page_fields(&mut self, py: Python<'_>) -> PyResult<usize> {
        let updated = py
            .detach(|| self.inner.update_page_fields())
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        if updated != 0 {
            self.revisions.bump();
        }
        Ok(updated)
    }

    fn update_layout_backed_fields(
        &mut self,
        py: Python<'_>,
    ) -> PyResult<PyLayoutBackedFieldUpdateReport> {
        let report = py
            .detach(|| self.inner.update_layout_backed_fields())
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        if report.updated_count() != 0 {
            self.revisions.bump();
        }
        Ok(PyLayoutBackedFieldUpdateReport {
            page_fields: report.page_fields,
            num_pages_fields: report.num_pages_fields,
            page_reference_fields: report.page_reference_fields,
            diagnostics: report.diagnostics,
        })
    }

    #[getter]
    fn paragraphs(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PyParagraphCollection>> {
        Py::new(py, PyParagraphCollection::new(slf))
    }

    #[getter]
    fn tables(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<PyTableCollection>> {
        Py::new(py, PyTableCollection::new(slf))
    }

    fn add_paragraph(slf: Py<Self>, py: Python<'_>, text: &str) -> PyResult<Py<PyParagraph>> {
        let (index, path) = {
            let mut document = slf.borrow_mut(py);
            let index = document.inner.paragraph_count();
            document.inner.add_paragraph(text);
            document.revisions.bump();
            let path = document
                .revisions
                .capture(smallvec![PathSeg::Body(0), PathSeg::Para(index)]);
            (index, path)
        };
        debug_assert!(matches!(path.segs.last(), Some(PathSeg::Para(i)) if *i == index));
        Py::new(py, PyParagraph::new(slf, path))
    }

    #[pyo3(signature = (rows, cols))]
    fn add_table(slf: Py<Self>, py: Python<'_>, rows: usize, cols: usize) -> PyResult<Py<PyTable>> {
        let (index, path) = {
            let mut document = slf.borrow_mut(py);
            let index = document.inner.table_count();
            document.inner.add_table(rows, cols);
            document.revisions.bump();
            let path = document.revisions.capture(smallvec![PathSeg::Body(index)]);
            (index, path)
        };
        debug_assert!(matches!(path.segs.last(), Some(PathSeg::Body(i)) if *i == index));
        Py::new(py, PyTable::new(slf, path))
    }

    fn remove_content(&mut self, index: usize) -> bool {
        let removed = self.inner.remove_content(index);
        if removed {
            self.revisions.bump();
        }
        removed
    }

    fn find_content_index(
        slf: Py<Self>,
        py: Python<'_>,
        content: &Bound<'_, PyAny>,
    ) -> PyResult<usize> {
        if let Ok(text) = content.extract::<String>() {
            return slf
                .borrow(py)
                .inner
                .find_content_index(&text)
                .ok_or_else(|| PyValueError::new_err("text was not found in body content"));
        }
        Self::direct_content_index(&slf, py, content)
    }

    fn find_content_indices<'py>(
        &self,
        py: Python<'py>,
        text: &str,
    ) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.inner.find_content_indices(text))
    }

    fn insert_paragraph(
        slf: Py<Self>,
        py: Python<'_>,
        index: usize,
        text: &str,
    ) -> PyResult<Py<PyParagraph>> {
        let path = {
            let mut document = slf.borrow_mut(py);
            if index > document.inner.content_count() {
                return Err(PyIndexError::new_err("content index out of range"));
            }
            document.inner.insert_paragraph(index, text);
            let paragraph = document
                .inner
                .paragraph_index_of_content(index)
                .expect("an inserted body paragraph has a paragraph index");
            document.revisions.bump();
            document
                .revisions
                .capture(smallvec![PathSeg::Body(0), PathSeg::Para(paragraph)])
        };
        Py::new(py, PyParagraph::new(slf, path))
    }

    fn pop_content(slf: Py<Self>, py: Python<'_>, index: usize) -> PyResult<PyContentFragment> {
        let location = slf.borrow(py).body_location(py, index)?;
        if index == slf.borrow(py).inner.content_count() {
            return Err(PyIndexError::new_err("content index out of range"));
        }
        let fragment = slf
            .borrow_mut(py)
            .inner
            .remove_content_at(&location)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        slf.borrow_mut(py).revisions.bump();
        Ok(PyContentFragment { inner: fragment })
    }

    fn insert_content(
        slf: Py<Self>,
        py: Python<'_>,
        destination: usize,
        fragment: PyRef<'_, PyContentFragment>,
    ) -> PyResult<()> {
        let location = slf.borrow(py).body_location(py, destination)?;
        let fragment = fragment.inner.clone();
        slf.borrow_mut(py)
            .inner
            .insert_content(&location, fragment)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        slf.borrow_mut(py).revisions.bump();
        Ok(())
    }

    fn clone_content(
        slf: Py<Self>,
        py: Python<'_>,
        source: &Bound<'_, PyAny>,
        destination: usize,
    ) -> PyResult<()> {
        let source_index = Self::direct_content_index(&slf, py, source)?;
        let (source, destination) = {
            let document = slf.borrow(py);
            (
                document.body_location(py, source_index)?,
                document.body_location(py, destination)?,
            )
        };
        slf.borrow_mut(py)
            .inner
            .clone_content(&source, &destination)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        slf.borrow_mut(py).revisions.bump();
        Ok(())
    }

    fn move_content(
        slf: Py<Self>,
        py: Python<'_>,
        source: &Bound<'_, PyAny>,
        destination: usize,
    ) -> PyResult<()> {
        let source_index = Self::direct_content_index(&slf, py, source)?;
        let (source, destination) = {
            let document = slf.borrow(py);
            (
                document.body_location(py, source_index)?,
                document.body_location(py, destination)?,
            )
        };
        slf.borrow_mut(py)
            .inner
            .move_content(&source, &destination)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        slf.borrow_mut(py).revisions.bump();
        Ok(())
    }
}

fn parse_raster_format(
    format: &str,
    quality: u8,
    transparent: bool,
) -> rdocx::Result<rdocx::RasterFormat> {
    match format {
        "png" => Ok(rdocx::RasterFormat::Png {
            transparent_background: transparent,
        }),
        "jpg" | "jpeg" => Ok(rdocx::RasterFormat::Jpeg { quality }),
        "tif" | "tiff" => Ok(rdocx::RasterFormat::Tiff),
        other => Err(rdocx::Error::Other(format!(
            "unknown raster format {other:?}, expected png, jpeg, or tiff"
        ))),
    }
}
