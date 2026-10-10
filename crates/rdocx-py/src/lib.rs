//! Python bindings for the rdocx facade.

mod document;
mod formatting;
mod paragraph;
mod run;
mod story;
mod table;

use pyo3::exceptions::{PyAttributeError, PyIndexError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyType};

use oxml_py_support::StaleElementError;

use document::{
    PyAppProperties, PyContentControl, PyCustomProperties, PyDocumentFragment, PySettings,
    PyValidationReport,
};
use document::{
    PyBookmark, PyBoundingBox, PyComment, PyComparisonDiagnostic, PyContentFragment,
    PyCoreProperties, PyDocument, PyHeaderFooterVariant, PyHyperlink,
    PyLayoutBackedFieldUpdateReport, PyLayoutFragment, PyLayoutPage, PyListLevel, PyPicture,
    PyRevision, PyRunPosition, PyRunRange, PySection, PyStory, PyStoryItem, PyStoryRunPosition,
    PyStoryRunRange, PyStyle, PySvgDiagnostic, PySvgRenderResult, PyTocRebuildReport,
};
use formatting::{PyColorFormat, PyFont, PyParagraphFormat, PyTabStop, PyTabStops};
use paragraph::{PyParagraph, PyParagraphCollection};
use run::{PyRun, PyRunCollection};
use story::{PyHeaderFooter, PyHeaderFooterCell, PyHeaderFooterRow, PyHeaderFooterTable};
use table::{
    PyCell, PyCellCollection, PyCellParagraphCollection, PyColumn, PyRow, PyRowCollection, PyTable,
    PyTableCollection,
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

pub(crate) fn length_object(py: Python<'_>, value: rdocx::Length) -> PyResult<Py<PyAny>> {
    py.import("rdocx")?
        .getattr("Length")?
        .call1((value.to_emu(),))
        .map(Bound::unbind)
}

pub(crate) fn enum_object(py: Python<'_>, name: &str, value: i32) -> PyResult<Py<PyAny>> {
    py.import("rdocx")?
        .getattr(name)?
        .call1((value,))
        .map(Bound::unbind)
}

/// Reads a colour argument as the value Word stores, six uppercase
/// hexadecimal digits or `auto`.
///
/// Every parameter and property that sets a colour reads it here, so all of
/// them accept the same forms: an `RGBColor` or any triple of 0 to 255
/// integers, a six-digit hex string with or without `#`, or `auto`. `name`
/// is the parameter the errors name.
pub(crate) fn color_hex(value: &Bound<'_, PyAny>, name: &str) -> PyResult<String> {
    if value.is_instance_of::<PyString>() {
        let text = value.extract::<String>()?;
        let digits = text.strip_prefix('#').unwrap_or(&text);
        if digits.len() == 6 && digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Ok(digits.to_ascii_uppercase());
        }
        if text.eq_ignore_ascii_case("auto") {
            return Ok("auto".to_owned());
        }
        return Err(PyValueError::new_err(format!(
            "{name} must be six hexadecimal digits such as \"FF0000\" or \"auto\", got {text:?}"
        )));
    }
    if let Ok(channels) = value.extract::<(i64, i64, i64)>() {
        let channel = |value: i64| {
            u8::try_from(value).map_err(|_| {
                PyValueError::new_err(format!("{name} channels must be from 0 to 255"))
            })
        };
        let (red, green, blue) = (
            channel(channels.0)?,
            channel(channels.1)?,
            channel(channels.2)?,
        );
        return Ok(format!("{red:02X}{green:02X}{blue:02X}"));
    }
    Err(PyTypeError::new_err(format!(
        "{name} must be an RGBColor or a hex string such as \"FF0000\", got {}",
        value.get_type().name()?
    )))
}

/// Returns the `RGBColor` of a stored colour value, or `None` for `auto`,
/// which names no colour of its own, and for a value that is not six
/// hexadecimal digits.
pub(crate) fn color_object(py: Python<'_>, value: &str) -> PyResult<Option<Py<PyAny>>> {
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(None);
    }
    py.import("rdocx")?
        .getattr("RGBColor")?
        .call_method1("from_string", (value,))
        .map(|color| Some(color.unbind()))
}

fn public_exception_type<'py>(py: Python<'py>, class_name: &str) -> PyResult<Bound<'py, PyType>> {
    py.import("rdocx")
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
/// caller expected. A single replacement uses the wording of
/// `rdocx replace --expect`, and a batch names the failing pair.
pub(crate) fn replacement_count_to_pyerr(
    py: Python<'_>,
    mismatch: &rdocx::ReplacementCountMismatch,
    batch: bool,
) -> PyErr {
    let (message, index) = if batch {
        (mismatch.to_string(), Some(mismatch.index))
    } else {
        (
            format!(
                "expected {} replacement(s) of \"{}\", found {}",
                mismatch.expected, mismatch.placeholder, mismatch.found
            ),
            None,
        )
    };
    match public_exception_type(py, "ReplacementCountError") {
        Ok(class) => PyErr::from_type(class, (message, mismatch.expected, mismatch.found, index)),
        Err(_) => PyRuntimeError::new_err(message),
    }
}

/// python-docx names that rdocx spells differently or does not have, by
/// class and attribute, and the rdocx way to do the same.
const DIVERGENCES: &[(&str, &str, &str)] = &[
    (
        "Paragraph",
        "clear",
        "set paragraph.text = '' to remove its runs",
    ),
    (
        "Paragraph",
        "insert_paragraph_after",
        "use document.insert_paragraph(document.find_content_index(paragraph) + 1, text)",
    ),
    (
        "Run",
        "style",
        "use run.style_id = 'Emphasis', a character style ID",
    ),
    ("Run", "add_text", "use run.text = run.text + text"),
    ("Run", "clear", "use run.text = ''"),
    (
        "Cell",
        "merge",
        "use table.set_cell_grid_span(row, col, span) to merge across and table.set_cell_vertical_merge(row, col, 'restart' or 'continue') to merge down",
    ),
    (
        "Section",
        "left_margin",
        "read section.margin_left, and change it with document.update_section(index, margin_left=Inches(1))",
    ),
    (
        "Section",
        "right_margin",
        "read section.margin_right, and change it with document.update_section(index, margin_right=Inches(1))",
    ),
    (
        "Section",
        "top_margin",
        "read section.margin_top, and change it with document.update_section(index, margin_top=Inches(1))",
    ),
    (
        "Section",
        "bottom_margin",
        "read section.margin_bottom, and change it with document.update_section(index, margin_bottom=Inches(1))",
    ),
    (
        "Section",
        "start_type",
        "read section.break_type, and change it with document.update_section(index, break_type='nextPage')",
    ),
    (
        "Style",
        "font",
        "a Style is a snapshot: change it with document.set_style(name, font_name='Arial', font_size=Pt(11), bold=True)",
    ),
    (
        "Style",
        "paragraph_format",
        "a Style is a snapshot: change it with document.set_style(name, space_before=Pt(6), left_indent=Inches(0.5))",
    ),
    (
        "Document",
        "inline_shapes",
        "use document.story_items to find drawings and document.set_picture_size(relationship_id, width, height) to resize one",
    ),
];

/// The lxml handles python-docx scripts reach for, which have no rdocx
/// counterpart: raw XML is read and replaced instead.
const LXML_NAMES: &[&str] = &[
    "_element", "element", "_p", "_r", "_tbl", "_tc", "_tr", "_body", "_sectPr",
];

/// An `AttributeError` for `class.name` that names the rdocx way when
/// python-docx spells it differently.
pub(crate) fn missing_attribute(class: &str, name: &str) -> PyErr {
    let hint = DIVERGENCES
        .iter()
        .find(|(owner, attribute, _)| *owner == class && *attribute == name)
        .map(|(.., hint)| (*hint).to_owned())
        .or_else(|| {
            LXML_NAMES.contains(&name).then(|| {
                let handle = class.to_lowercase();
                let removal = if matches!(class, "Paragraph" | "Table") {
                    format!(
                        ", and delete it with document.remove_content(document.find_content_index({handle}))"
                    )
                } else {
                    String::new()
                };
                format!(
                    "rdocx has no lxml element: read {handle}.xml and write {handle}.replace_xml(xml){removal}"
                )
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

/// EMU in one twip, the unit Word stores most lengths in.
pub(crate) const TWIP_EMU: i64 = 635;

/// EMU in one half point, the unit Word stores font sizes in.
pub(crate) const HALF_POINT_EMU: i64 = 6_350;

/// The length a setter was given, refused when it is not zero but rounds to
/// zero in the file, as a bare int meant as points or twips does.
pub(crate) fn stored_length(name: &str, value: i64, unit_emu: i64) -> PyResult<rdocx::Length> {
    if value != 0 && value.abs() < unit_emu {
        let short = |value: f64| {
            let text = format!("{value:.3}");
            text.trim_end_matches('0').trim_end_matches('.').to_owned()
        };
        let points = short(value as f64 / 12_700.0);
        let step = short(unit_emu as f64 / 12_700.0);
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "{name} is {value} EMU ({points} pt), under the {step} pt step Word stores, so the \
             file would hold 0: a bare int is read as EMU, give a Length of at least Pt({step}), \
             such as Pt(12) or Inches(0.5)"
        )));
    }
    Ok(rdocx::Length::emu(value))
}

/// The extensions of the formats `Document.save` does not write, and the
/// call that writes each one.
const OTHER_FORMATS: &[(&str, &str)] = &[
    ("pdf", "write the bytes of document.to_pdf()"),
    (
        "png",
        "write the bytes of document.render_page_to_png(page_index)",
    ),
    (
        "doc",
        "save as .docx, rdocx writes WordprocessingML packages only",
    ),
    (
        "rtf",
        "save as .docx, rdocx writes WordprocessingML packages only",
    ),
    (
        "odt",
        "save as .docx, rdocx writes WordprocessingML packages only",
    ),
    (
        "html",
        "save as .docx, rdocx writes WordprocessingML packages only",
    ),
    (
        "md",
        "save as .docx, rdocx writes WordprocessingML packages only",
    ),
    ("txt", "save as .docx, or write the text of every paragraph"),
];

/// Refuse a save path whose extension names another format: the file would
/// hold a .docx package whatever its name says.
pub(crate) fn check_save_extension(path: &std::path::Path) -> PyResult<()> {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return Ok(());
    };
    let extension = extension.to_ascii_lowercase();
    match OTHER_FORMATS.iter().find(|(known, _)| *known == extension) {
        Some((_, instead)) => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "save writes a .docx package, not .{extension}: {instead}"
        ))),
        None => Ok(()),
    }
}

pub(crate) fn stale_to_pyerr(py: Python<'_>, error: StaleElementError) -> PyErr {
    public_error(py, "StaleElementError", error.to_string())
}

pub(crate) fn rdocx_to_pyerr(py: Python<'_>, error: rdocx::Error) -> PyErr {
    let class_name = match &error {
        rdocx::Error::Opc(_)
        | rdocx::Error::Io(_)
        | rdocx::Error::NoDocumentPart
        | rdocx::Error::UnavailableImageDimensions { .. } => "PackageError",
        rdocx::Error::Oxml(_) => "XmlError",
        rdocx::Error::Layout(_) | rdocx::Error::Pdf(_) | rdocx::Error::Raster(_) => "LayoutError",
        rdocx::Error::Rtf { .. }
        | rdocx::Error::Html { .. }
        | rdocx::Error::Mhtml { .. }
        | rdocx::Error::Odt { .. }
        | rdocx::Error::InvalidEmbeddedMutation { .. }
        | rdocx::Error::Story(_)
        | rdocx::Error::Other(_) => "RdocxError",
    };
    public_error(py, class_name, error.to_string())
}

#[pymodule]
fn _rdocx(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyDocument>()?;
    module.add_class::<PyRunPosition>()?;
    module.add_class::<PyRunRange>()?;
    module.add_class::<PyBookmark>()?;
    module.add_class::<PyStoryRunPosition>()?;
    module.add_class::<PyStoryRunRange>()?;
    module.add_class::<PyComment>()?;
    module.add_class::<PyComparisonDiagnostic>()?;
    module.add_class::<PySvgDiagnostic>()?;
    module.add_class::<PySvgRenderResult>()?;
    module.add_class::<PyBoundingBox>()?;
    module.add_class::<PyLayoutFragment>()?;
    module.add_class::<PyLayoutPage>()?;
    module.add_class::<PyLayoutBackedFieldUpdateReport>()?;
    module.add_class::<PyTocRebuildReport>()?;
    module.add_class::<PyRevision>()?;
    module.add_class::<PyStory>()?;
    module.add_class::<PyStoryItem>()?;
    module.add_class::<PyContentFragment>()?;
    module.add_class::<PyCoreProperties>()?;
    module.add_class::<PyHyperlink>()?;
    module.add_class::<PyPicture>()?;
    module.add_class::<PyHeaderFooterVariant>()?;
    module.add_class::<PySection>()?;
    module.add_class::<PyStyle>()?;
    module.add_class::<PyListLevel>()?;
    module.add_class::<PyParagraph>()?;
    module.add_class::<PyParagraphCollection>()?;
    module.add_class::<PyRun>()?;
    module.add_class::<PyRunCollection>()?;
    module.add_class::<PyFont>()?;
    module.add_class::<PyColorFormat>()?;
    module.add_class::<PyParagraphFormat>()?;
    module.add_class::<PyTabStop>()?;
    module.add_class::<PyTabStops>()?;
    module.add_class::<PyTable>()?;
    module.add_class::<PyColumn>()?;
    module.add_class::<PyTableCollection>()?;
    module.add_class::<PyRow>()?;
    module.add_class::<PyRowCollection>()?;
    module.add_class::<PyCell>()?;
    module.add_class::<PyCellCollection>()?;
    module.add_class::<PyCellParagraphCollection>()?;
    // Document-level views, templates, properties and validation.
    module.add_class::<PyValidationReport>()?;
    module.add_class::<PyDocumentFragment>()?;
    module.add_class::<PyContentControl>()?;
    module.add_class::<PySettings>()?;
    module.add_class::<PyCustomProperties>()?;
    module.add_class::<PyAppProperties>()?;
    module.add_class::<paragraph::PyEquation>()?;
    module.add_class::<PyHeaderFooter>()?;
    module.add_class::<PyHeaderFooterTable>()?;
    module.add_class::<PyHeaderFooterRow>()?;
    module.add_class::<PyHeaderFooterCell>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::ffi::c_str;

    #[test]
    fn layout_error_maps_to_the_exact_public_layout_error_class() {
        Python::initialize();
        Python::attach(|py| -> PyResult<()> {
            let package = PyModule::from_code(
                py,
                c_str!(
                    "class RdocxError(Exception):\n    pass\n\nclass LayoutError(RdocxError):\n    pass\n"
                ),
                c_str!("rdocx_test.py"),
                c_str!("rdocx"),
            )?;
            py.import("sys")?
                .getattr("modules")?
                .set_item("rdocx", &package)?;
            let expected = package.getattr("LayoutError")?.cast_into::<PyType>()?;

            let error = rdocx::Error::Layout(oxml_layout::LayoutError::Layout(
                "classifier regression".to_string(),
            ));
            let raised = rdocx_to_pyerr(py, error);

            assert!(raised.get_type(py).is(&expected));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn import_errors_map_to_the_generic_public_error_class() {
        Python::initialize();
        Python::attach(|py| -> PyResult<()> {
            let package = PyModule::from_code(
                py,
                c_str!("class RdocxError(Exception):\n    pass\n"),
                c_str!("rdocx_test.py"),
                c_str!("rdocx"),
            )?;
            py.import("sys")?
                .getattr("modules")?
                .set_item("rdocx", &package)?;
            let expected = package.getattr("RdocxError")?.cast_into::<PyType>()?;
            for error in [
                rdocx::Error::Html {
                    location: "body[0]".to_owned(),
                    message: "invalid HTML".to_owned(),
                },
                rdocx::Error::Odt {
                    part: Some("content.xml".to_owned()),
                    offset: 0,
                    message: "invalid ODT".to_owned(),
                },
                rdocx::Error::Mhtml {
                    part: Some("HTML root".to_owned()),
                    offset: 0,
                    message: "invalid MHTML".to_owned(),
                },
                rdocx::Error::InvalidEmbeddedMutation {
                    operation: "replace",
                    message: "invalid embedded mutation".to_owned(),
                },
                rdocx::Error::Story(rdocx::StoryError::InvalidPath {
                    path: vec![usize::MAX],
                }),
            ] {
                assert!(rdocx_to_pyerr(py, error).get_type(py).is(&expected));
            }
            Ok(())
        })
        .unwrap();
    }
}
