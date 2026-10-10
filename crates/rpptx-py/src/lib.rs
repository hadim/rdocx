//! Python bindings for the rpptx facade.

mod dml;
mod layout;
mod presentation;
mod shape;
mod slide;
mod table;
mod text;

use pyo3::exceptions::{PyAttributeError, PyIndexError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

use oxml_py_support::{ContentPath, PathSeg, RevisionCounter};
use presentation::{
    PyComment, PyCommentAuthor, PyCommentReply, PyCoreProperties, PyPresentation, PySection,
    PyValidationIssue,
};

pub(crate) fn normalize_index(index: isize, len: usize, kind: &str) -> PyResult<usize> {
    let normalized = if index < 0 {
        len as isize + index
    } else {
        index
    };
    if normalized < 0 || normalized >= len as isize {
        return Err(PyIndexError::new_err(format!("{kind} index out of range")));
    }
    Ok(normalized as usize)
}

fn public_exception_type<'py>(py: Python<'py>, class_name: &str) -> PyResult<Bound<'py, PyType>> {
    py.import("rpptx")
        .and_then(|module| module.getattr(class_name))
        .and_then(|class| class.cast_into::<PyType>().map_err(Into::into))
}

fn public_error(py: Python<'_>, class_name: &str, message: String) -> PyErr {
    match public_exception_type(py, class_name) {
        Ok(class) => PyErr::from_type(class, (message,)),
        Err(_) => PyRuntimeError::new_err(message),
    }
}

/// A counted replacement that matched a different number of times than the
/// caller expected, worded like `rpptx replace --expect`.
pub(crate) fn replacement_count_to_pyerr(
    py: Python<'_>,
    placeholder: &str,
    expected: usize,
    found: usize,
) -> PyErr {
    let message = format!("expected {expected} replacement(s) of \"{placeholder}\", found {found}");
    match public_exception_type(py, "ReplacementCountError") {
        Ok(class) => PyErr::from_type(class, (message, expected, found)),
        Err(_) => PyRuntimeError::new_err(message),
    }
}

/// python-pptx names that rpptx spells differently or does not have, by
/// class and attribute, and the rpptx way to do the same.
const DIVERGENCES: &[(&str, &str, &str)] = &[
    (
        "SlideCollection",
        "_sldIdLst",
        "use prs.slides.remove(slide) to delete a slide and prs.slides.move(old_index, new_index) to reorder slides",
    ),
    (
        "NotesTextFrame",
        "paragraphs",
        "notes are read and written as plain text here: use notes_text_frame.text or slide.notes_text",
    ),
    (
        "NotesTextFrame",
        "add_paragraph",
        "append a line to notes_text_frame.text, for example text_frame.text += '\\nNext point'",
    ),
];

/// The lxml handles python-pptx scripts reach for, which have no rpptx
/// counterpart: raw XML is read and replaced instead.
const LXML_NAMES: &[&str] = &[
    "_element", "element", "_sp", "_txBody", "_sld", "_r", "_p", "_tbl", "_tc",
];

/// The handles that read and replace their own XML.
const XML_OWNERS: &[&str] = &["Shape", "TextFrame", "Slide", "SlideLayout"];

/// An `AttributeError` for `class.name` that names the rpptx way when
/// python-pptx spells it differently.
pub(crate) fn missing_attribute(class: &str, name: &str) -> PyErr {
    let hint = DIVERGENCES
        .iter()
        .find(|(owner, attribute, _)| *owner == class && *attribute == name)
        .map(|(.., hint)| (*hint).to_owned())
        .or_else(|| {
            LXML_NAMES.contains(&name).then(|| {
                if XML_OWNERS.contains(&class) {
                    let handle = match class {
                        "TextFrame" => "text_frame".to_owned(),
                        "SlideLayout" => "slide_layout".to_owned(),
                        other => other.to_lowercase(),
                    };
                    format!(
                        "rpptx has no lxml element: read {handle}.xml and write {handle}.replace_xml(xml)"
                    )
                } else {
                    "rpptx has no lxml element: read shape.xml or shape.text_frame.xml and write them back with replace_xml(xml)".to_owned()
                }
            })
        });
    match hint {
        Some(hint) => PyAttributeError::new_err(format!(
            "'{class}' object has no attribute '{name}': {hint}"
        )),
        None => PyAttributeError::new_err(format!("'{class}' object has no attribute '{name}'")),
    }
}

/// Set `name` through the property the class defines, or raise
/// [`missing_attribute`]: a pyclass without `__dict__` would otherwise say
/// only that the attribute does not exist.
pub(crate) fn set_attribute(
    slf: &Bound<'_, PyAny>,
    class: &str,
    name: &str,
    value: &Bound<'_, PyAny>,
) -> PyResult<()> {
    match slf.get_type().getattr(name) {
        Ok(descriptor) if descriptor.hasattr("__set__")? => {
            descriptor.call_method1("__set__", (slf, value))?;
            Ok(())
        }
        _ => Err(missing_attribute(class, name)),
    }
}

/// The bytes of a `replace_xml` argument: `str`, `bytes` or `bytearray`.
pub(crate) fn raw_xml_argument(xml: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(text) = xml.extract::<String>() {
        return Ok(text.into_bytes());
    }
    xml.extract::<Vec<u8>>()
        .map_err(|_| PyTypeError::new_err("xml must be str or bytes"))
}

/// A refused raw XML replacement is a `ValueError` that says why.
pub(crate) fn raw_xml_error(py: Python<'_>, error: rpptx::Error) -> PyErr {
    match error {
        rpptx::Error::InvalidShapeMutation {
            operation: "replace_xml",
            message,
        } => PyValueError::new_err(message),
        error => rpptx_to_pyerr(py, error),
    }
}

pub(crate) fn recovery_hint(path: &ContentPath, suffix: &str) -> String {
    let mut public_path = String::from("prs");
    let mut pending_row = None;
    for segment in &path.segs {
        match segment {
            PathSeg::Slide(index) => public_path.push_str(&format!(".slides[{index}]")),
            PathSeg::Layout(index) => public_path.push_str(&format!(".slide_layouts[{index}]")),
            PathSeg::Master(index) => public_path.push_str(&format!(".slide_masters[{index}]")),
            PathSeg::Shape(index) => public_path.push_str(&format!(".shapes[{index}]")),
            PathSeg::Body(index) => public_path.push_str(&format!(".body[{index}]")),
            PathSeg::Row(index) => pending_row = Some(*index),
            PathSeg::Cell(index) => {
                if let Some(row) = pending_row.take() {
                    public_path.push_str(&format!(".table.cell({row}, {index})"));
                } else {
                    public_path.push_str(&format!(".table.columns[{index}]"));
                }
            }
            PathSeg::Para(index) => {
                public_path.push_str(&format!(".text_frame.paragraphs[{index}]"));
            }
            PathSeg::Run(index) => public_path.push_str(&format!(".runs[{index}]")),
            // Presentation handles never capture a story segment.
            PathSeg::Story(_) => {}
        }
    }
    if let Some(row) = pending_row {
        public_path.push_str(&format!(".table.rows[{row}]"));
    }
    public_path.push_str(suffix);
    format!("Re-fetch it with {public_path}.")
}

/// How far a structural edit reaches, from the widest scope to the narrowest.
///
/// A handle belongs to the scope of the last step of its path: a slide to
/// `Slides`, a shape to `Shapes`, a table row or cell to `Tables`, a paragraph
/// to `Paragraphs`, and a run to `Runs`. An edit invalidates the handles of
/// its scope and of every narrower one, so replacing a text frame's text
/// invalidates its paragraph and run handles but keeps its shape and slide
/// handles, as python-pptx keeps them. Collections without a path, such as
/// `prs.slides` and `prs.slide_layouts`, read the presentation live and stay
/// valid.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Scope {
    Slides = 0,
    Shapes = 1,
    Tables = 2,
    Paragraphs = 3,
    Runs = 4,
    /// Slide layout and slide master handles. Outside the slide chain: a
    /// slide edit keeps them, and only layout and master edits (remove,
    /// duplicate, theme import, raw XML) retire them.
    Layouts = 5,
}

/// The scopes from [`Scope::Slides`] down to [`Scope::Runs`], which an
/// invalidation of a wider one also retires.
const CHAINED_SCOPES: usize = Scope::Runs as usize + 1;

impl Scope {
    const fn name(self) -> &'static str {
        match self {
            Self::Slides => "slide",
            Self::Shapes => "shape",
            Self::Tables => "table",
            Self::Paragraphs => "paragraph",
            Self::Runs => "run",
            Self::Layouts => "layout",
        }
    }

    fn of(path: &ContentPath) -> Option<Self> {
        Some(match path.segs.last()? {
            PathSeg::Slide(_) => Self::Slides,
            // Removing, duplicating, or importing layouts renumbers them.
            PathSeg::Layout(_) | PathSeg::Master(_) => Self::Layouts,
            PathSeg::Shape(_) => Self::Shapes,
            PathSeg::Row(_) | PathSeg::Cell(_) => Self::Tables,
            PathSeg::Body(_) | PathSeg::Para(_) => Self::Paragraphs,
            PathSeg::Run(_) => Self::Runs,
            // A Word header or footer story never appears in a deck path.
            PathSeg::Story(_) => return None,
        })
    }
}

/// One revision counter per [`Scope`], with the call that last advanced it.
#[derive(Debug)]
pub(crate) struct HandleRevisions {
    counters: [RevisionCounter; 6],
    causes: [&'static str; 6],
}

impl HandleRevisions {
    pub(crate) fn new() -> Self {
        Self {
            counters: [RevisionCounter::new(); 6],
            causes: [""; 6],
        }
    }

    /// Invalidates the handles of `scope` and of every narrower scope,
    /// recording `cause`, the public call, for the stale-handle message.
    /// [`Scope::Layouts`] has no narrower scope.
    pub(crate) fn invalidate(&mut self, scope: Scope, cause: &'static str) {
        let end = if scope == Scope::Layouts {
            self.counters.len()
        } else {
            CHAINED_SCOPES
        };
        for level in scope as usize..end {
            self.counters[level].bump();
            self.causes[level] = cause;
        }
    }

    /// Captures `segs` at the revision of the scope its last step belongs to.
    pub(crate) fn capture(&self, segs: smallvec::SmallVec<[PathSeg; 5]>) -> ContentPath {
        let path = ContentPath::new(segs, 0);
        let revision = Scope::of(&path).map_or(0, |scope| self.counters[scope as usize].current());
        ContentPath::new(path.segs, revision)
    }
}

pub(crate) fn validate_path(
    py: Python<'_>,
    presentation: &PyPresentation,
    path: &ContentPath,
    kind: &str,
    suffix: &str,
) -> PyResult<()> {
    let Some(scope) = Scope::of(path) else {
        return Ok(());
    };
    let revisions = &presentation.revisions;
    let current = revisions.counters[scope as usize].current();
    if path.revision == current {
        return Ok(());
    }
    Err(public_error(
        py,
        "StaleElementError",
        format!(
            "{kind} handle was created at {name} revision {captured}, but the {name} revision \
             is now {current} because {cause} renumbered what it points to. {hint}",
            name = scope.name(),
            captured = path.revision,
            cause = revisions.causes[scope as usize],
            hint = recovery_hint(path, suffix),
        ),
    ))
}

pub(crate) fn rpptx_to_pyerr(py: Python<'_>, error: rpptx::Error) -> PyErr {
    let class_name = match &error {
        rpptx::Error::MalformedPart { .. } => "XmlError",
        rpptx::Error::Package(_)
        | rpptx::Error::MissingMainDocument
        | rpptx::Error::MissingRelationship { .. }
        | rpptx::Error::WrongRelationshipType { .. }
        | rpptx::Error::ExternalRelationship { .. }
        | rpptx::Error::MissingPart { .. }
        | rpptx::Error::CorePropertiesPartCollision { .. }
        | rpptx::Error::DuplicateNotesSlides { .. } => "PackageError",
        _ => "RpptxError",
    };
    public_error(py, class_name, error.to_string())
}

pub(crate) fn rpptx_value_to_pyerr(py: Python<'_>, message: String) -> PyErr {
    public_error(py, "RpptxError", message)
}

#[pymodule]
fn _rpptx(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPresentation>()?;
    module.add_class::<PyCommentAuthor>()?;
    module.add_class::<PyComment>()?;
    module.add_class::<PyCommentReply>()?;
    module.add_class::<PyValidationIssue>()?;
    module.add_class::<PyCoreProperties>()?;
    module.add_class::<PySection>()?;
    slide::register(module)?;
    shape::register(module)?;
    dml::register(module)?;
    text::register(module)?;
    table::register(module)?;
    layout::register(module)?;
    Ok(())
}
