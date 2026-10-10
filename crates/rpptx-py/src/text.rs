use oxml_drawing::text::{
    TextAutoNumber, TextAutoNumberScheme, TextBullet, TextBulletColor, TextBulletSize,
    TextBulletSizeValue, TextPointValue,
};
use oxml_py_support::{ContentPath, PathSeg};
use pyo3::PyClass;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyList, PySlice, PyString};
use rpptx::{
    AutofitMode, CT_TextCharacterProperties, CT_TextParagraphProperties, ColorChoice, Emu, Fill,
    SolidFill, TextAlignment, TextAnchor, TextBulletCharacter, TextBulletChoice, TextFont,
    TextNoBullet, TextSpacing, TextStrike, TextUnderline,
};

use crate::Scope;
use crate::dml::{ColorSource, PyColorFormat, rgb_color};
use crate::layout::PyAutofitResult;
use crate::normalize_index;
use crate::presentation::PyPresentation;
use crate::replacement_count_to_pyerr;
use crate::rpptx_to_pyerr;
use crate::shape::{shape_mut_at, shape_ref_at, slide_index};
use crate::slide::PySlide;
use crate::table::{cell_mut_at, cell_ref_at};
use crate::validate_path;

const EMU_PER_CENTIPOINT: i64 = 127;
const MAX_SPACING_EMU: i64 = 158_400 * EMU_PER_CENTIPOINT;
const MAX_TEXT_MARGIN: i64 = 51_206_400;

/// `ST_TextFontSize` in hundredths of a point, 1 to 4000 points.
const MIN_FONT_SIZE: i64 = 100;
const MAX_FONT_SIZE: i64 = 400_000;

/// `a:lnSpc`, `a:spcBef`, and `a:spcAft` accept at most 132 lines.
const MAX_SPACING_LINES: f64 = 132.0;

fn length_object(py: Python<'_>, emu: i64) -> PyResult<Py<PyAny>> {
    py.import("rpptx")?
        .getattr("Length")?
        .call1((emu,))
        .map(Bound::unbind)
}

fn text_enum(py: Python<'_>, name: &str, value: i32) -> PyResult<Py<PyAny>> {
    py.import("rpptx.enum.text")?
        .getattr(name)?
        .call1((value,))
        .map(Bound::unbind)
}

fn spacing_object(py: Python<'_>, spacing: Option<TextSpacing>) -> PyResult<Option<Py<PyAny>>> {
    match spacing {
        None => Ok(None),
        Some(TextSpacing::Points(centipoints)) => {
            length_object(py, i64::from(centipoints) * EMU_PER_CENTIPOINT).map(Some)
        }
        Some(TextSpacing::Percent(value)) => {
            let lines = match value.strip_suffix('%') {
                Some(percent) => percent.parse::<f64>().map(|percent| percent / 100.0),
                None => value.parse::<f64>().map(|value| value / 100_000.0),
            };
            let lines = lines.map_err(|_| PyValueError::new_err("spacing is not a number"))?;
            Ok(Some(PyFloat::new(py, lines).into_any().unbind()))
        }
    }
}

fn points_spacing(emu: i64) -> PyResult<TextSpacing> {
    if !(0..=MAX_SPACING_EMU).contains(&emu) {
        return Err(PyValueError::new_err(
            "spacing must be from 0 to 1584 points",
        ));
    }
    Ok(TextSpacing::Points((emu / EMU_PER_CENTIPOINT) as i32))
}

fn lines_spacing(lines: f64) -> PyResult<TextSpacing> {
    if !(0.0..=MAX_SPACING_LINES).contains(&lines) {
        return Err(PyValueError::new_err("spacing must be from 0 to 132 lines"));
    }
    Ok(TextSpacing::Percent(
        ((lines * 100_000.0).round() as i64).to_string(),
    ))
}

fn margin_value(value: Option<i64>, min: i64, name: &str) -> PyResult<Option<i32>> {
    value
        .map(|value| {
            if (min..=MAX_TEXT_MARGIN).contains(&value) {
                Ok(value as i32)
            } else {
                Err(PyValueError::new_err(format!(
                    "{name} must be from {min} to {MAX_TEXT_MARGIN} EMU"
                )))
            }
        })
        .transpose()
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyTextFrame>()?;
    module.add_class::<PyParagraph>()?;
    module.add_class::<PyParagraphCollection>()?;
    module.add_class::<PyRun>()?;
    module.add_class::<PyRunCollection>()?;
    module.add_class::<PyHyperlink>()?;
    module.add_class::<PyFont>()?;
    module.add_class::<PyMasterTextStyles>()?;
    module.add_class::<PyTextStyleList>()?;
    module.add_class::<PyTextStyleLevel>()?;
    Ok(())
}

fn paragraph_index(path: &ContentPath) -> Option<usize> {
    path.segs.iter().find_map(|segment| match segment {
        PathSeg::Para(index) => Some(*index),
        _ => None,
    })
}

fn run_index(path: &ContentPath) -> Option<usize> {
    path.segs.iter().find_map(|segment| match segment {
        PathSeg::Run(index) => Some(*index),
        _ => None,
    })
}

/// Returns the `MSO_VERTICAL_ANCHOR` member of a text anchor.
pub(crate) fn anchor_object(py: Python<'_>, anchor: TextAnchor) -> PyResult<Py<PyAny>> {
    let value = match anchor {
        TextAnchor::Top => 1,
        TextAnchor::Center => 3,
        TextAnchor::Bottom => 4,
        TextAnchor::Justified => 6,
        TextAnchor::Distributed => 7,
    };
    text_enum(py, "MSO_VERTICAL_ANCHOR", value)
}

/// Reads an `MSO_VERTICAL_ANCHOR` member as a text anchor.
pub(crate) fn anchor_from_value(value: i32) -> PyResult<TextAnchor> {
    match value {
        1 => Ok(TextAnchor::Top),
        3 => Ok(TextAnchor::Center),
        4 => Ok(TextAnchor::Bottom),
        6 => Ok(TextAnchor::Justified),
        7 => Ok(TextAnchor::Distributed),
        _ => Err(PyValueError::new_err(
            "vertical anchor must be an MSO_ANCHOR member",
        )),
    }
}

/// Whether a path names a table cell, whose text frame is the cell's own.
fn names_cell(path: &ContentPath) -> bool {
    path.segs
        .iter()
        .any(|segment| matches!(segment, PathSeg::Row(_)))
}

/// Returns the text frame a path names: a table cell's when the path names a
/// cell, otherwise its shape's.
fn frame_ref_at<'a>(
    presentation: &'a rpptx::Presentation,
    path: &ContentPath,
) -> Option<rpptx::TextFrameRef<'a>> {
    if names_cell(path) {
        return cell_ref_at(presentation, path)?.text_frame();
    }
    shape_ref_at(presentation, path)?.text_frame()
}

/// Returns the text frame a path names for mutation. A table cell without a
/// text body gets an empty one, as python-pptx adds one.
fn frame_mut_at<'a>(
    presentation: &'a mut rpptx::Presentation,
    path: &ContentPath,
) -> Option<rpptx::TextFrame<'a>> {
    if names_cell(path) {
        return cell_mut_at(presentation, path).map(rpptx::TableCellMut::into_text_frame);
    }
    shape_mut_at(presentation, path)?.into_text_frame()
}

pub(crate) fn font_properties(
    presentation: &rpptx::Presentation,
    path: &ContentPath,
) -> Option<rpptx::CT_TextCharacterProperties> {
    let paragraph = paragraph_index(path)?;
    let paragraph =
        frame_ref_at(presentation, path).and_then(|frame| frame.paragraph(paragraph))?;
    match run_index(path) {
        Some(run) => paragraph.run(run)?.properties().cloned(),
        None => paragraph.default_run_properties().cloned(),
    }
}

#[pyclass(name = "TextFrame")]
pub struct PyTextFrame {
    pub(crate) presentation: Py<PyPresentation>,
    pub(crate) path: ContentPath,
}

impl PyTextFrame {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<()> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "text frame",
            ".text_frame",
        )
    }

    fn read<T>(
        &self,
        py: Python<'_>,
        read: impl FnOnce(rpptx::TextFrameRef<'_>) -> T,
    ) -> PyResult<T> {
        self.validate(py)?;
        frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .map(read)
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))
    }

    fn update<T>(
        &self,
        py: Python<'_>,
        update: impl FnOnce(&mut rpptx::TextFrame<'_>) -> T,
    ) -> PyResult<T> {
        self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        frame_mut_at(&mut presentation.inner, &self.path)
            .map(|mut frame| update(&mut frame))
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))
    }

    fn margin(
        &self,
        py: Python<'_>,
        side: fn(&mut Insets) -> &mut Option<Emu>,
    ) -> PyResult<Option<Py<PyAny>>> {
        let mut insets = self.read(py, |frame| frame.insets())?;
        side(&mut insets)
            .map(|inset| length_object(py, inset.0))
            .transpose()
    }

    fn set_margin(
        &self,
        py: Python<'_>,
        side: fn(&mut Insets) -> &mut Option<Emu>,
        value: Option<i64>,
    ) -> PyResult<()> {
        if value.is_some_and(|value| i32::try_from(value).is_err()) {
            return Err(PyValueError::new_err(
                "text frame margin must fit a 32-bit EMU coordinate",
            ));
        }
        let mut insets = self.read(py, |frame| frame.insets())?;
        *side(&mut insets) = value.map(Emu);
        let (left, right, top, bottom) = insets;
        self.update(py, |frame| frame.set_insets(left, right, top, bottom))?
            .map_err(|error| rpptx_to_pyerr(py, error))
    }
}

/// Left, right, top, and bottom text insets, as the facade reports them.
type Insets = (Option<Emu>, Option<Emu>, Option<Emu>, Option<Emu>);

/// The slide and shape-tree path of a shape's own text frame, `None` for a
/// table cell, which PowerPoint never autofits.
fn autofit_shape_path(path: &ContentPath) -> Option<(usize, Vec<usize>)> {
    let mut slide = None;
    let mut shapes = Vec::new();
    for segment in &path.segs {
        match segment {
            PathSeg::Slide(index) => slide = Some(*index),
            PathSeg::Shape(index) => shapes.push(*index),
            _ => return None,
        }
    }
    Some((slide?, shapes))
}

impl PyTextFrame {
    /// Marks this frame for an autofit refresh before every save and render,
    /// or forgets it. A frame without a slide and shape id is refreshed now.
    fn mark_autofit(&self, py: Python<'_>, marked: bool) -> PyResult<()> {
        let Some((slide, shapes)) = autofit_shape_path(&self.path) else {
            return Ok(());
        };
        let key = {
            let presentation = self.presentation.borrow(py);
            let slide_id = presentation.inner.slide(slide).map(|slide| slide.id());
            let shape_id = shape_ref_at(&presentation.inner, &self.path)
                .and_then(|shape| shape.non_visual_id());
            slide_id.zip(shape_id)
        };
        let Some(key) = key else {
            if marked {
                self.presentation
                    .borrow_mut(py)
                    .inner
                    .refresh_shape_autofit(slide, &shapes)
                    .map_err(|error| rpptx_to_pyerr(py, error))?;
            }
            return Ok(());
        };
        let marks = &mut self.presentation.borrow_mut(py).autofit_marks;
        marks.retain(|mark| *mark != key);
        if marked {
            marks.push(key);
        }
        Ok(())
    }

    /// Stores this frame's autofit result, `None` for a table cell or a
    /// frame without normal or shape autofit in effect.
    fn refresh_shape_autofit(&self, py: Python<'_>) -> PyResult<Option<rpptx::AutofitResult>> {
        self.validate(py)?;
        let Some((slide, shapes)) = autofit_shape_path(&self.path) else {
            return Ok(None);
        };
        self.presentation
            .borrow_mut(py)
            .inner
            .refresh_shape_autofit(slide, &shapes)
            .map_err(|error| rpptx_to_pyerr(py, error))
    }
}

#[pymethods]
impl PyTextFrame {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        self.validate(py)?;
        frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .map(|frame| frame.text())
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))
    }

    #[setter]
    fn set_text(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        frame_mut_at(&mut presentation.inner, &self.path)
            .map(|mut frame| frame.set_text(value))
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))?;
        presentation
            .revisions
            .invalidate(Scope::Paragraphs, "TextFrame.text");
        Ok(())
    }

    #[getter]
    fn paragraphs(&self, py: Python<'_>) -> PyResult<Py<PyParagraphCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyParagraphCollection::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }

    /// Replaces literal text in this text frame only and returns the count.
    ///
    /// The contract is `Presentation.try_replace_text` restricted to this
    /// frame: with `expect`, the replacement runs on a copy of the text body,
    /// a count that differs raises and leaves the presentation and its
    /// revisions as they were, and run handles are invalidated only when
    /// something was replaced.
    #[pyo3(signature = (placeholder, replacement, *, expect = None))]
    fn try_replace_text(
        &self,
        py: Python<'_>,
        placeholder: &str,
        replacement: &str,
        expect: Option<usize>,
    ) -> PyResult<usize> {
        self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let count = frame_mut_at(&mut presentation.inner, &self.path)
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))?
            .try_replace_text(placeholder, replacement, expect)
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        if let Some(expected) = expect.filter(|&expected| expected != count) {
            return Err(replacement_count_to_pyerr(py, placeholder, expected, count));
        }
        if count > 0 {
            presentation
                .revisions
                .invalidate(Scope::Runs, "TextFrame.try_replace_text()");
        }
        Ok(count)
    }

    fn add_paragraph(&self, py: Python<'_>) -> PyResult<Py<PyParagraph>> {
        self.validate(py)?;
        let original_path = self.path.clone();
        let index = {
            let mut presentation = self.presentation.borrow_mut(py);
            let mut frame = frame_mut_at(&mut presentation.inner, &original_path)
                .ok_or_else(|| PyValueError::new_err("shape has no text frame"))?;
            let index = frame.paragraph_count();
            frame.add_paragraph();
            index
        };
        let path = {
            let presentation = self.presentation.borrow(py);
            let mut segments = original_path.segs.clone();
            segments.push(PathSeg::Para(index));
            presentation.revisions.capture(segments)
        };
        Py::new(py, PyParagraph::new(self.presentation.clone_ref(py), path))
    }

    #[getter]
    fn autofit(&self, py: Python<'_>) -> PyResult<Option<&'static str>> {
        self.validate(py)?;
        Ok(
            frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
                .and_then(|frame| frame.autofit_mode())
                .map(|mode| match mode {
                    rpptx::AutofitMode::None => "none",
                    rpptx::AutofitMode::Normal => "normal",
                    rpptx::AutofitMode::Shape => "shape",
                }),
        )
    }

    #[getter]
    fn auto_size(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let mode = self.read(py, |frame| frame.autofit_mode())?;
        mode.map(|mode| {
            let value = match mode {
                AutofitMode::None => 0,
                AutofitMode::Shape => 1,
                AutofitMode::Normal => 2,
            };
            text_enum(py, "MSO_AUTO_SIZE", value)
        })
        .transpose()
    }

    #[setter]
    fn set_auto_size(&self, py: Python<'_>, value: Option<i32>) -> PyResult<()> {
        let mode = value
            .map(|value| match value {
                0 => Ok(AutofitMode::None),
                1 => Ok(AutofitMode::Shape),
                2 => Ok(AutofitMode::Normal),
                _ => Err(PyValueError::new_err(
                    "auto size must be an MSO_AUTO_SIZE member",
                )),
            })
            .transpose()?;
        self.update(py, |frame| frame.set_autofit_mode(mode))?;
        self.mark_autofit(
            py,
            matches!(mode, Some(AutofitMode::Normal | AutofitMode::Shape)),
        )
    }

    #[getter]
    fn margin_left(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |insets| &mut insets.0)
    }

    #[setter]
    fn set_margin_left(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        self.set_margin(py, |insets| &mut insets.0, value)
    }

    #[getter]
    fn margin_right(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |insets| &mut insets.1)
    }

    #[setter]
    fn set_margin_right(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        self.set_margin(py, |insets| &mut insets.1, value)
    }

    #[getter]
    fn margin_top(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |insets| &mut insets.2)
    }

    #[setter]
    fn set_margin_top(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        self.set_margin(py, |insets| &mut insets.2, value)
    }

    #[getter]
    fn margin_bottom(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |insets| &mut insets.3)
    }

    #[setter]
    fn set_margin_bottom(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        self.set_margin(py, |insets| &mut insets.3, value)
    }

    #[getter]
    fn vertical_anchor(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let anchor = self.read(py, |frame| frame.vertical_anchor())?;
        anchor.map(|anchor| anchor_object(py, anchor)).transpose()
    }

    #[setter]
    fn set_vertical_anchor(&self, py: Python<'_>, value: Option<i32>) -> PyResult<()> {
        let anchor = value.map(anchor_from_value).transpose()?;
        self.update(py, |frame| frame.set_vertical_anchor(anchor))
    }

    #[getter]
    fn word_wrap(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read(py, |frame| frame.word_wrap())
    }

    #[setter]
    fn set_word_wrap(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.update(py, |frame| frame.set_word_wrap(value))
    }

    /// Stores the values this frame's autofit asks for now and returns them.
    ///
    /// `TEXT_TO_FIT_SHAPE` gets PowerPoint's `fontScale` and
    /// `lnSpcReduction`, `SHAPE_TO_FIT_TEXT` a shape size that fits the
    /// text, keeping the anchored edge. A frame whose `auto_size` was set is
    /// refreshed on every save and render anyway. `None` means the frame has
    /// no such autofit in effect.
    fn refresh_autofit(&self, py: Python<'_>) -> PyResult<Option<PyAutofitResult>> {
        if autofit_shape_path(&self.path).is_none() {
            return Err(PyValueError::new_err(
                "PowerPoint does not autofit table cell text: set the row height or the font size instead",
            ));
        }
        Ok(self.refresh_shape_autofit(py)?.map(PyAutofitResult::from))
    }

    /// Writes the largest size up to `max_size` points at which the text
    /// fits the shape on every run, as python-pptx does.
    ///
    /// The frame then has no autofit and wraps words. `font_family` `None`
    /// keeps each run's typeface, where python-pptx would write Calibri. The
    /// text is measured with rpptx's deterministic fonts, or with
    /// `font_file` when given, and the result lists any typeface it had to
    /// replace. An empty frame is left unchanged and returns `None`.
    #[pyo3(signature = (font_family = None, max_size = 18.0, bold = false, italic = false, font_file = None))]
    fn fit_text(
        &self,
        py: Python<'_>,
        font_family: Option<&str>,
        max_size: f64,
        bold: bool,
        italic: bool,
        font_file: Option<std::path::PathBuf>,
    ) -> PyResult<Option<PyAutofitResult>> {
        self.validate(py)?;
        // python-pptx takes `int(max_size)`, so 18.0 and 18.5 mean 18 points.
        let max_size = max_size.trunc();
        if !(1.0..=4_000.0).contains(&max_size) {
            return Err(PyValueError::new_err(
                "max_size must be from 1 to 4000 points",
            ));
        }
        let max_size = max_size as u32;
        let Some((slide, shapes)) = autofit_shape_path(&self.path) else {
            return Err(PyValueError::new_err(
                "fit_text measures a shape's own text frame: set table cell font sizes directly",
            ));
        };
        let font_file = font_file
            .map(|path| {
                std::fs::read(&path).map_err(|error| {
                    PyValueError::new_err(format!(
                        "cannot read font_file {}: {error}",
                        path.display()
                    ))
                })
            })
            .transpose()?;
        let options = rpptx::FitTextOptions {
            font_family,
            max_size,
            bold,
            italic,
            font_file: font_file.as_deref(),
        };
        let result = self
            .presentation
            .borrow_mut(py)
            .inner
            .fit_text(slide, &shapes, options)
            .map_err(|error| rpptx_to_pyerr(py, error))?;
        if result.is_some() {
            // The frame now has no autofit, so a save must not refit it.
            self.mark_autofit(py, false)?;
        }
        Ok(result.map(PyAutofitResult::from))
    }
}

#[pyclass(name = "Paragraph")]
pub struct PyParagraph {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyParagraph {
    fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "paragraph",
            "",
        )?;
        paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))
    }

    fn read<T>(
        &self,
        py: Python<'_>,
        read: impl FnOnce(Option<&CT_TextParagraphProperties>) -> T,
    ) -> PyResult<T> {
        let index = self.validate(py)?;
        frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|frame| frame.paragraph(index))
            .map(|paragraph| read(paragraph.properties()))
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))
    }

    /// Applies `update` to the direct paragraph properties, writing them only
    /// when they change, so clearing an absent value inserts no `a:pPr`.
    fn update(
        &self,
        py: Python<'_>,
        update: impl FnOnce(&mut CT_TextParagraphProperties),
    ) -> PyResult<()> {
        let index = self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut paragraph = frame_mut_at(&mut presentation.inner, &self.path)
            .and_then(|frame| frame.into_paragraph_mut(index))
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        let current = paragraph.properties().cloned().unwrap_or_default();
        let mut properties = current.clone();
        update(&mut properties);
        if properties != current {
            paragraph.set_properties(properties);
        }
        Ok(())
    }

    fn spacing(
        &self,
        py: Python<'_>,
        spacing: fn(&CT_TextParagraphProperties) -> Option<&TextSpacing>,
    ) -> PyResult<Option<Py<PyAny>>> {
        let spacing = self.read(py, |properties| properties.and_then(spacing).cloned())?;
        spacing_object(py, spacing)
    }

    fn margin(
        &self,
        py: Python<'_>,
        margin: fn(&CT_TextParagraphProperties) -> Option<i32>,
    ) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |properties| properties.and_then(margin))?
            .map(|emu| length_object(py, i64::from(emu)))
            .transpose()
    }
}

/// Reads a space-before or space-after value: a `Length` is exact points,
/// and a float is a number of lines.
fn paragraph_spacing(value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<TextSpacing>> {
    match value {
        None => Ok(None),
        Some(value) if value.is_none() => Ok(None),
        Some(value) if value.is_instance_of::<PyFloat>() => {
            lines_spacing(value.extract()?).map(Some)
        }
        Some(value) => points_spacing(value.extract()?).map(Some),
    }
}

fn alignment_value(alignment: TextAlignment) -> i32 {
    match alignment {
        TextAlignment::Left => 1,
        TextAlignment::Center => 2,
        TextAlignment::Right => 3,
        TextAlignment::Justified => 4,
        TextAlignment::Distributed => 5,
        TextAlignment::ThaiDistributed => 6,
        TextAlignment::JustifiedLow => 7,
    }
}

fn alignment_from_value(value: i32) -> PyResult<TextAlignment> {
    match value {
        1 => Ok(TextAlignment::Left),
        2 => Ok(TextAlignment::Center),
        3 => Ok(TextAlignment::Right),
        4 => Ok(TextAlignment::Justified),
        5 => Ok(TextAlignment::Distributed),
        6 => Ok(TextAlignment::ThaiDistributed),
        7 => Ok(TextAlignment::JustifiedLow),
        _ => Err(PyValueError::new_err(
            "paragraph alignment must be a PP_ALIGN member",
        )),
    }
}

#[pymethods]
impl PyParagraph {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        let index = self.validate(py)?;
        frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|frame| frame.paragraph(index))
            .map(|paragraph| paragraph.text())
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))
    }

    #[setter]
    fn set_text(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        let index = self.validate(py)?;
        let mut presentation = self.presentation.borrow_mut(py);
        frame_mut_at(&mut presentation.inner, &self.path)
            .and_then(|frame| frame.into_paragraph_mut(index))
            .map(|mut paragraph| paragraph.set_text(value))
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        presentation
            .revisions
            .invalidate(Scope::Runs, "Paragraph.text");
        Ok(())
    }

    #[getter]
    fn level(&self, py: Python<'_>) -> PyResult<u8> {
        let index = self.validate(py)?;
        frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|frame| frame.paragraph(index))
            .map(|paragraph| paragraph.level())
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))
    }

    #[setter]
    fn set_level(&self, py: Python<'_>, value: u8) -> PyResult<()> {
        let index = self.validate(py)?;
        let changed = frame_mut_at(&mut self.presentation.borrow_mut(py).inner, &self.path)
            .and_then(|frame| frame.into_paragraph_mut(index))
            .is_some_and(|mut paragraph| paragraph.set_level(value));
        if changed {
            Ok(())
        } else if value > 8 {
            Err(PyValueError::new_err("paragraph level must be from 0 to 8"))
        } else {
            Err(PyIndexError::new_err("paragraph index out of range"))
        }
    }

    #[getter]
    fn font(&self, py: Python<'_>) -> PyResult<Py<PyFont>> {
        self.validate(py)?;
        Py::new(
            py,
            PyFont {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                style: None,
            },
        )
    }

    #[getter]
    fn runs(&self, py: Python<'_>) -> PyResult<Py<PyRunCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyRunCollection::new(self.presentation.clone_ref(py), self.path.clone()),
        )
    }

    #[pyo3(signature = (text = ""))]
    fn add_run(&self, py: Python<'_>, text: &str) -> PyResult<Py<PyRun>> {
        let index = self.validate(py)?;
        let run = frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|frame| frame.paragraph(index))
            .map(|paragraph| paragraph.run_count())
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        let path = {
            let mut presentation = self.presentation.borrow_mut(py);
            frame_mut_at(&mut presentation.inner, &self.path)
                .and_then(|frame| frame.into_paragraph_mut(index))
                .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?
                .add_run(text);
            let mut segments = self.path.segs.clone();
            segments.push(PathSeg::Run(run));
            presentation.revisions.capture(segments)
        };
        Py::new(py, PyRun::new(self.presentation.clone_ref(py), path))
    }

    /// Appends a field, `slidenum` for the slide number or a date field
    /// `datetime` or `datetime1` to `datetime13`, which PowerPoint refreshes
    /// when it opens the file (rpptx extension).
    ///
    /// `text` is the cached value other readers show. By default a slide
    /// number caches the placeholder PowerPoint writes, which rpptx renders
    /// as the number, and a date field caches today's date in its format.
    /// `paragraph.runs` lists regular runs only, so it does not change.
    #[pyo3(signature = (field_type, text = None))]
    fn add_field(&self, py: Python<'_>, field_type: &str, text: Option<String>) -> PyResult<()> {
        let index = self.validate(py)?;
        let text = match text {
            Some(text) => Some(text),
            None if field_type.starts_with("datetime") => Some(crate::slide::today_field_text(
                py,
                field_type,
                " or pass text= with the cached value",
            )?),
            None => None,
        };
        let mut presentation = self.presentation.borrow_mut(py);
        shape_mut_at(&mut presentation.inner, &self.path)
            .and_then(rpptx::ShapeMut::into_text_frame)
            .and_then(|frame| frame.into_paragraph_mut(index))
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?
            .add_field(field_type, text.as_deref())
            .map_err(|error| crate::rpptx_to_pyerr(py, error))
    }

    #[getter]
    fn alignment(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |properties| {
            properties.and_then(|properties| properties.alignment)
        })?
        .map(|alignment| text_enum(py, "PP_PARAGRAPH_ALIGNMENT", alignment_value(alignment)))
        .transpose()
    }

    #[setter]
    fn set_alignment(&self, py: Python<'_>, value: Option<i32>) -> PyResult<()> {
        let alignment = value.map(alignment_from_value).transpose()?;
        self.update(py, |properties| properties.alignment = alignment)
    }

    #[getter]
    fn line_spacing(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.spacing(py, |properties| properties.line_spacing.as_ref())
    }

    #[setter]
    fn set_line_spacing(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let spacing = match value {
            None => None,
            Some(value) if value.is_none() => None,
            Some(value) if value.is_instance(&py.import("rpptx")?.getattr("Length")?)? => {
                Some(points_spacing(value.extract()?)?)
            }
            Some(value) => Some(lines_spacing(value.extract()?)?),
        };
        self.update(py, |properties| properties.line_spacing = spacing)
    }

    #[getter]
    fn space_before(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.spacing(py, |properties| properties.space_before.as_ref())
    }

    #[setter]
    fn set_space_before(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let spacing = paragraph_spacing(value)?;
        self.update(py, |properties| properties.space_before = spacing)
    }

    #[getter]
    fn space_after(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.spacing(py, |properties| properties.space_after.as_ref())
    }

    #[setter]
    fn set_space_after(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let spacing = paragraph_spacing(value)?;
        self.update(py, |properties| properties.space_after = spacing)
    }

    #[getter]
    fn left_indent(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |properties| properties.left_margin)
    }

    #[setter]
    fn set_left_indent(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        let margin = margin_value(value, 0, "left indent")?;
        self.update(py, |properties| properties.left_margin = margin)
    }

    #[getter]
    fn right_indent(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |properties| properties.right_margin)
    }

    #[setter]
    fn set_right_indent(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        let margin = margin_value(value, 0, "right indent")?;
        self.update(py, |properties| properties.right_margin = margin)
    }

    #[getter]
    fn first_line_indent(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |properties| properties.indent)
    }

    #[setter]
    fn set_first_line_indent(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        let indent = margin_value(value, -MAX_TEXT_MARGIN, "first line indent")?;
        self.update(py, |properties| properties.indent = indent)
    }

    #[getter]
    fn bullet(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let bullet = self.read(py, |properties| properties.and_then(read_bullet))?;
        Ok(bullet_object(py, bullet))
    }

    #[setter]
    fn set_bullet(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let choice = bullet_choice(value)?;
        self.update(py, |properties| apply_bullet(properties, choice))
    }

    /// The auto-numbering scheme of a numbered paragraph, such as
    /// `arabicPeriod` (1. 2. 3.), `alphaLcParenR` (a) b) c)), or
    /// `romanUcPeriod` (I. II. III.), or `None` when the paragraph is not
    /// numbered. Assigning a scheme numbers the paragraph, keeping a start
    /// value already set, and `None` removes the numbering.
    #[getter]
    fn auto_number(&self, py: Python<'_>) -> PyResult<Option<&'static str>> {
        self.read_bullet(py, |bullet| match &bullet.choice {
            Some(TextBulletChoice::AutoNumber(number)) => Some(number.scheme.as_str()),
            _ => None,
        })
    }

    #[setter]
    fn set_auto_number(&self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
        let scheme = value
            .map(|value| {
                TextAutoNumberScheme::parse(value).ok_or_else(|| {
                    PyValueError::new_err(format!(
                        "auto number scheme must be an ST_TextAutonumberScheme token such as \
                         \"arabicPeriod\", got {value:?}"
                    ))
                })
            })
            .transpose()?;
        self.edit_bullet(py, |bullet| match (scheme, &mut bullet.choice) {
            (Some(scheme), Some(TextBulletChoice::AutoNumber(number))) => number.scheme = scheme,
            (Some(scheme), choice) => {
                *choice = Some(TextBulletChoice::AutoNumber(TextAutoNumber::new(scheme)));
            }
            (None, choice @ Some(TextBulletChoice::AutoNumber(_))) => *choice = None,
            (None, _) => {}
        })
    }

    /// The number of the first paragraph of an auto-numbered list, from 1
    /// to 32767, or `None` to start at 1.
    #[getter]
    fn auto_number_start(&self, py: Python<'_>) -> PyResult<Option<u16>> {
        self.read_bullet(py, |bullet| match &bullet.choice {
            Some(TextBulletChoice::AutoNumber(number)) => number.start_at,
            _ => None,
        })
    }

    #[setter]
    fn set_auto_number_start(&self, py: Python<'_>, value: Option<u16>) -> PyResult<()> {
        if value.is_some_and(|value| !(1..=32_767).contains(&value)) {
            return Err(PyValueError::new_err(
                "auto number start must be from 1 to 32767",
            ));
        }
        if value.is_some() && self.auto_number(py)?.is_none() {
            return Err(PyValueError::new_err(
                "the paragraph is not numbered, set auto_number first",
            ));
        }
        self.edit_bullet(py, |bullet| {
            if let Some(TextBulletChoice::AutoNumber(number)) = &mut bullet.choice {
                number.start_at = value;
            }
        })
    }

    /// The sRGB colour of the bullet or number as an `RGBColor`, or `None`
    /// when it follows the text or uses another kind of colour.
    #[getter]
    fn bullet_color(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let color = self.read_bullet(py, |bullet| match bullet.color.as_ref()?.color {
            ColorChoice::Srgb { value, .. } => Some(value),
            _ => None,
        })?;
        color
            .map(|color| {
                let [red, green, blue] = color.components();
                py.import("rpptx.dml.color")?
                    .getattr("RGBColor")?
                    .call1((red, green, blue))
                    .map(Bound::unbind)
            })
            .transpose()
    }

    #[setter]
    fn set_bullet_color(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let color = value
            .filter(|value| !value.is_none())
            .map(|value| rgb_color(value, "bullet_color"))
            .transpose()?;
        self.edit_bullet(py, |bullet| {
            bullet.color = color.map(|color| TextBulletColor::new(ColorChoice::srgb(color)));
        })
    }

    /// The bullet or number size as a fraction of the text size, from 0.25
    /// to 4.0, or `None` when it follows the text.
    #[getter]
    fn bullet_size(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        let size = self.read_bullet(py, |bullet| match &bullet.size.as_ref()?.value {
            TextBulletSizeValue::Percent(value) => Some(value.clone()),
            TextBulletSizeValue::Points(_) => None,
        })?;
        size.map(|value| {
            match value.strip_suffix('%') {
                Some(percent) => percent.parse::<f64>().map(|percent| percent / 100.0),
                None => value.parse::<f64>().map(|value| value / 100_000.0),
            }
            .map_err(|_| PyValueError::new_err(format!("bullet size {value:?} is not a number")))
        })
        .transpose()
    }

    #[setter]
    fn set_bullet_size(&self, py: Python<'_>, value: Option<f64>) -> PyResult<()> {
        let size = value
            .map(|value| {
                TextBulletSize::percent(((value * 100_000.0).round_ties_even() as i64).to_string())
                    .map_err(|_| {
                        PyValueError::new_err(format!(
                            "bullet size must be from 0.25 to 4.0, got {value}"
                        ))
                    })
            })
            .transpose()?;
        self.edit_bullet(py, |bullet| bullet.size = size)
    }

    /// The typeface of a bullet character, such as `Wingdings`, or `None`
    /// when it follows the text.
    #[getter]
    fn bullet_font(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.read_bullet(py, |bullet| {
            bullet.font.as_ref().map(|font| font.typeface.clone())
        })
    }

    #[setter]
    fn set_bullet_font(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        let font = typeface(value)?;
        self.edit_bullet(py, |bullet| set_typeface(&mut bullet.font, font))
    }
}

impl PyParagraph {
    /// Reads one part of the direct bullet.
    fn read_bullet<T>(
        &self,
        py: Python<'_>,
        read: impl FnOnce(&TextBullet) -> Option<T>,
    ) -> PyResult<Option<T>> {
        self.read(py, |properties| {
            properties
                .and_then(|properties| properties.bullet.as_ref())
                .and_then(read)
        })
    }

    /// Changes the direct bullet, removing it when nothing is left.
    fn edit_bullet(&self, py: Python<'_>, edit: impl FnOnce(&mut TextBullet)) -> PyResult<()> {
        self.update(py, |properties| {
            let mut bullet = properties.bullet.clone().unwrap_or_default();
            edit(&mut bullet);
            properties.set_bullet((bullet != TextBullet::default()).then_some(bullet));
        })
    }
}

/// A paragraph's bullet: its character, `Err(false)` for none, or
/// `Err(true)` for a numbered or picture bullet.
fn read_bullet(properties: &CT_TextParagraphProperties) -> Option<Result<String, bool>> {
    let choice = properties
        .bullet
        .as_ref()
        .and_then(|bullet| bullet.choice.as_ref());
    match choice {
        Some(TextBulletChoice::Character(character)) => Some(Ok(character.character.clone())),
        Some(TextBulletChoice::None(_)) => Some(Err(false)),
        Some(TextBulletChoice::AutoNumber(_)) => Some(Err(true)),
        None => properties.has_picture_bullet().then_some(Err(true)),
    }
}

fn bullet_object(py: Python<'_>, bullet: Option<Result<String, bool>>) -> Option<Py<PyAny>> {
    bullet.map(|bullet| match bullet {
        Ok(character) => PyString::new(py, &character).into_any().unbind(),
        Err(shown) => PyBool::new(py, shown).to_owned().into_any().unbind(),
    })
}

/// Reads a bullet assignment: a character, `False` for no bullet, or `None`
/// to inherit.
fn bullet_choice(value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<TextBulletChoice>> {
    Ok(match value {
        None => None,
        Some(value) if value.is_none() => None,
        Some(value) if value.is_instance_of::<PyBool>() => {
            if value.extract::<bool>()? {
                return Err(PyValueError::new_err(
                    "assign a bullet character, False for no bullet, or None",
                ));
            }
            Some(TextBulletChoice::None(TextNoBullet::default()))
        }
        Some(value) => {
            let character = value.extract::<String>()?;
            let character = TextBulletCharacter::new(character)
                .map_err(|_| PyValueError::new_err("bullet character must not be empty"))?;
            Some(TextBulletChoice::Character(character))
        }
    })
}

fn apply_bullet(properties: &mut CT_TextParagraphProperties, choice: Option<TextBulletChoice>) {
    let Some(choice) = choice else {
        properties.set_bullet(None);
        return;
    };
    let mut bullet = properties.bullet.clone().unwrap_or_default();
    match (&mut bullet.choice, choice) {
        (Some(TextBulletChoice::Character(current)), TextBulletChoice::Character(character)) => {
            current.character = character.character
        }
        (current, choice) => *current = Some(choice),
    }
    properties.set_bullet(Some(bullet));
}

#[pyclass(name = "ParagraphCollection")]
pub struct PyParagraphCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyParagraphCollection {
    fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let presentation = self.presentation.borrow(py);
        validate_path(
            py,
            &presentation,
            &self.path,
            "paragraph collection",
            ".text_frame.paragraphs",
        )?;
        drop(presentation);
        frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .map(|frame| frame.paragraph_count())
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyParagraph>> {
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Para(index));
        let path = self.presentation.borrow(py).revisions.capture(segments);
        Py::new(py, PyParagraph::new(self.presentation.clone_ref(py), path))
    }
}

#[pymethods]
impl PyParagraphCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        sequence_item(py, key, self.len(py)?, "paragraph", |index| {
            self.item(py, index)
        })
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyParagraphIterator>> {
        self.len(py)?;
        Py::new(
            py,
            PyParagraphIterator {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyParagraphIterator {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    index: usize,
}

#[pymethods]
impl PyParagraphIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyParagraph>>> {
        let collection =
            PyParagraphCollection::new(self.presentation.clone_ref(py), self.path.clone());
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Run")]
pub struct PyRun {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyRun {
    fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }
}

#[pymethods]
impl PyRun {
    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "run", "")?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let run =
            run_index(&self.path).ok_or_else(|| PyIndexError::new_err("run index is missing"))?;
        frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|frame| frame.paragraph(paragraph))
            .and_then(|paragraph| paragraph.run(run))
            .map(|run| run.text().to_owned())
            .ok_or_else(|| PyIndexError::new_err("run index out of range"))
    }

    /// Replaces this run's text in place. No run, break, or paragraph is
    /// added or removed for any value, so the revision does not advance.
    #[setter]
    fn set_text(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "run", "")?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let run =
            run_index(&self.path).ok_or_else(|| PyIndexError::new_err("run index is missing"))?;
        let mut presentation = self.presentation.borrow_mut(py);
        let frame = frame_mut_at(&mut presentation.inner, &self.path)
            .ok_or_else(|| PyValueError::new_err("shape has no text frame"))?;
        let mut paragraph = frame
            .into_paragraph_mut(paragraph)
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        let mut run = paragraph
            .run_mut(run)
            .ok_or_else(|| PyIndexError::new_err("run index out of range"))?;
        run.set_text(value);
        Ok(())
    }

    #[getter]
    fn font(&self, py: Python<'_>) -> PyResult<Py<PyFont>> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "run", "")?;
        Py::new(
            py,
            PyFont {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                style: None,
            },
        )
    }

    #[getter]
    fn hyperlink(&self, py: Python<'_>) -> PyResult<Py<PyHyperlink>> {
        validate_path(py, &self.presentation.borrow(py), &self.path, "run", "")?;
        Py::new(
            py,
            PyHyperlink {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
            },
        )
    }
}

/// The click hyperlink of one run, like python-pptx `_Hyperlink`.
#[pyclass(name = "Hyperlink")]
pub struct PyHyperlink {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyHyperlink {
    /// Returns the run's direct character properties, raising when the run
    /// no longer exists.
    fn run_properties(
        &self,
        py: Python<'_>,
        presentation: &PyPresentation,
    ) -> PyResult<Option<CT_TextCharacterProperties>> {
        validate_path(py, presentation, &self.path, "hyperlink", ".hyperlink")?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let run =
            run_index(&self.path).ok_or_else(|| PyIndexError::new_err("run index is missing"))?;
        frame_ref_at(&presentation.inner, &self.path)
            .and_then(|frame| frame.paragraph(paragraph))
            .and_then(|paragraph| paragraph.run(run))
            .map(|run| run.properties().cloned())
            .ok_or_else(|| PyIndexError::new_err("run index out of range"))
    }
}

#[pymethods]
impl PyHyperlink {
    /// The address the run's click hyperlink opens, or `None` without one.
    #[getter]
    fn address(&self, py: Python<'_>) -> PyResult<Option<String>> {
        let presentation = self.presentation.borrow(py);
        let properties = self.run_properties(py, &presentation)?;
        let Some(relationship_id) = properties
            .as_ref()
            .and_then(|properties| properties.hyperlink_click.as_ref())
            .and_then(|hyperlink| hyperlink.relationship_id.as_deref())
            .filter(|id| !id.is_empty())
        else {
            return Ok(None);
        };
        Ok(presentation
            .inner
            .hyperlink_address(slide_index(&self.path)?, relationship_id)
            .map(str::to_owned))
    }

    /// Points the run's click hyperlink at `value`, or removes it for `None`
    /// or an empty string, as python-pptx does. The relationship the old
    /// hyperlink used goes when nothing else on the slide uses it.
    #[setter]
    fn set_address(&self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
        let mut presentation = self.presentation.borrow_mut(py);
        self.run_properties(py, &presentation)?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let run =
            run_index(&self.path).ok_or_else(|| PyIndexError::new_err("run index is missing"))?;
        if names_cell(&self.path) {
            return Err(PyValueError::new_err(
                "a hyperlink cannot be set on a run in a table cell yet",
            ));
        }
        let shape_id = shape_ref_at(&presentation.inner, &self.path)
            .and_then(|shape| shape.non_visual_id())
            .ok_or_else(|| PyValueError::new_err("shape has no id"))?;
        presentation
            .inner
            .set_run_hyperlink(
                slide_index(&self.path)?,
                shape_id,
                paragraph,
                run,
                value.filter(|value| !value.is_empty()),
            )
            .map_err(|error| rpptx_to_pyerr(py, error))
    }

    // Shape authoring: a run hyperlink that jumps to a slide.

    /// The slide the run's click hyperlink jumps to, or `None` for any other
    /// hyperlink or none, as `shape.click_action.target_slide` reads a
    /// shape's. Assigning a slide makes the run jump there, and `None`
    /// removes the hyperlink.
    #[getter]
    fn target_slide(&self, py: Python<'_>) -> PyResult<Option<Py<PySlide>>> {
        let presentation = self.presentation.borrow(py);
        self.run_properties(py, &presentation)?;
        let (slide, shape_id, paragraph, run) = self.run_location(&presentation)?;
        let Some(target) = presentation
            .inner
            .run_target_slide(slide, shape_id, paragraph, run)
            .map_err(|error| rpptx_to_pyerr(py, error))?
        else {
            return Ok(None);
        };
        let path = presentation
            .revisions
            .capture(smallvec::smallvec![PathSeg::Slide(target)]);
        drop(presentation);
        Py::new(py, PySlide::new(self.presentation.clone_ref(py), path)).map(Some)
    }

    #[setter]
    fn set_target_slide(&self, py: Python<'_>, slide: Option<PyRef<'_, PySlide>>) -> PyResult<()> {
        let target = match slide {
            Some(slide) if !slide.presentation.is(&self.presentation) => {
                return Err(PyValueError::new_err("slide is not in this presentation"));
            }
            Some(slide) => Some(slide.validate(py)?),
            None => None,
        };
        let mut presentation = self.presentation.borrow_mut(py);
        self.run_properties(py, &presentation)?;
        let (slide, shape_id, paragraph, run) = self.run_location(&presentation)?;
        presentation
            .inner
            .set_run_target_slide(slide, shape_id, paragraph, run, target)
            .map_err(|error| rpptx_to_pyerr(py, error))
    }
}

impl PyHyperlink {
    /// The slide index, shape id, paragraph index, and run index of the run.
    fn run_location(&self, presentation: &PyPresentation) -> PyResult<(usize, u32, usize, usize)> {
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let run =
            run_index(&self.path).ok_or_else(|| PyIndexError::new_err("run index is missing"))?;
        let shape_id = shape_ref_at(&presentation.inner, &self.path)
            .and_then(|shape| shape.non_visual_id())
            .ok_or_else(|| PyValueError::new_err("shape has no id"))?;
        Ok((slide_index(&self.path)?, shape_id, paragraph, run))
    }
}

#[pyclass(name = "RunCollection")]
pub struct PyRunCollection {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyRunCollection {
    fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "run collection",
            ".runs",
        )?;
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        frame_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .and_then(|frame| frame.paragraph(paragraph))
            .map(|paragraph| paragraph.run_count())
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyRun>> {
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Run(index));
        let path = self.presentation.borrow(py).revisions.capture(segments);
        Py::new(py, PyRun::new(self.presentation.clone_ref(py), path))
    }
}

#[pymethods]
impl PyRunCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        sequence_item(py, key, self.len(py)?, "run", |index| self.item(py, index))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyRunIterator>> {
        self.len(py)?;
        Py::new(
            py,
            PyRunIterator {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyRunIterator {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    index: usize,
}

#[pymethods]
impl PyRunIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyRun>>> {
        let collection = PyRunCollection::new(self.presentation.clone_ref(py), self.path.clone());
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Font")]
pub struct PyFont {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    /// A master text style level, whose default run properties this font
    /// reads and writes instead of a paragraph's or run's (#312).
    style: Option<(rpptx::MasterTextStyle, usize)>,
}

impl PyFont {
    /// A font that reads and writes a master text style level when `style`
    /// names one, and the run or paragraph `path` names otherwise.
    pub(crate) fn styled(
        presentation: Py<PyPresentation>,
        path: ContentPath,
        style: Option<(rpptx::MasterTextStyle, usize)>,
    ) -> Self {
        Self {
            presentation,
            path,
            style,
        }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<()> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "font",
            ".font",
        )
    }

    pub(crate) fn read<T>(
        &self,
        py: Python<'_>,
        read: impl FnOnce(Option<&CT_TextCharacterProperties>) -> T,
    ) -> PyResult<T> {
        self.validate(py)?;
        if let Some((style, level)) = self.style {
            return text_style_read(py, &self.presentation, &self.path, style, level, |level| {
                read(level.and_then(|level| level.default_run_properties.as_ref()))
            });
        }
        let properties = font_properties(&self.presentation.borrow(py).inner, &self.path);
        Ok(read(properties.as_ref()))
    }

    /// Applies `update` to the direct run or paragraph default properties,
    /// writing them only when they change, so clearing an absent value
    /// inserts no `a:rPr` or `a:defRPr`.
    pub(crate) fn update(
        &self,
        py: Python<'_>,
        update: impl FnOnce(&mut CT_TextCharacterProperties),
    ) -> PyResult<()> {
        self.validate(py)?;
        if let Some((style, level)) = self.style {
            return text_style_update(py, &self.presentation, &self.path, style, level, |level| {
                let current = level.default_run_properties.clone().unwrap_or_default();
                let mut properties = current.clone();
                update(&mut properties);
                if properties != current {
                    level.default_run_properties = Some(properties);
                }
            });
        }
        let paragraph = paragraph_index(&self.path)
            .ok_or_else(|| PyIndexError::new_err("paragraph index is missing"))?;
        let mut presentation = self.presentation.borrow_mut(py);
        let mut paragraph = frame_mut_at(&mut presentation.inner, &self.path)
            .and_then(|frame| frame.into_paragraph_mut(paragraph))
            .ok_or_else(|| PyIndexError::new_err("paragraph index out of range"))?;
        if let Some(run) = run_index(&self.path) {
            let mut run = paragraph
                .run_mut(run)
                .ok_or_else(|| PyIndexError::new_err("run index out of range"))?;
            let current = run.properties().cloned().unwrap_or_default();
            let mut properties = current.clone();
            update(&mut properties);
            if properties != current {
                run.set_properties(properties);
            }
        } else {
            let current = paragraph
                .properties()
                .and_then(|properties| properties.default_run_properties.clone())
                .unwrap_or_default();
            let mut properties = current.clone();
            update(&mut properties);
            if properties != current {
                *paragraph.default_run_properties_mut() = properties;
            }
        }
        Ok(())
    }
}

fn underline_from_value(value: i32) -> PyResult<TextUnderline> {
    Ok(match value {
        0 => TextUnderline::None,
        1 => TextUnderline::Words,
        2 => TextUnderline::Single,
        3 => TextUnderline::Double,
        4 => TextUnderline::Heavy,
        5 => TextUnderline::Dotted,
        6 => TextUnderline::DottedHeavy,
        7 => TextUnderline::Dash,
        8 => TextUnderline::DashHeavy,
        9 => TextUnderline::DashLong,
        10 => TextUnderline::DashLongHeavy,
        11 => TextUnderline::DotDash,
        12 => TextUnderline::DotDashHeavy,
        13 => TextUnderline::DotDotDash,
        14 => TextUnderline::DotDotDashHeavy,
        15 => TextUnderline::Wavy,
        16 => TextUnderline::WavyHeavy,
        17 => TextUnderline::WavyDouble,
        _ => {
            return Err(PyValueError::new_err(
                "underline must be True, False, None, or an MSO_UNDERLINE member",
            ));
        }
    })
}

fn underline_value(underline: TextUnderline) -> i32 {
    match underline {
        TextUnderline::None => 0,
        TextUnderline::Words => 1,
        TextUnderline::Single => 2,
        TextUnderline::Double => 3,
        TextUnderline::Heavy => 4,
        TextUnderline::Dotted => 5,
        TextUnderline::DottedHeavy => 6,
        TextUnderline::Dash => 7,
        TextUnderline::DashHeavy => 8,
        TextUnderline::DashLong => 9,
        TextUnderline::DashLongHeavy => 10,
        TextUnderline::DotDash => 11,
        TextUnderline::DotDashHeavy => 12,
        TextUnderline::DotDotDash => 13,
        TextUnderline::DotDotDashHeavy => 14,
        TextUnderline::Wavy => 15,
        TextUnderline::WavyHeavy => 16,
        TextUnderline::WavyDouble => 17,
    }
}

fn typeface(value: Option<String>) -> PyResult<Option<TextFont>> {
    value
        .map(TextFont::new)
        .transpose()
        .map_err(|_| PyValueError::new_err("font name must not be empty"))
}

/// Replaces or removes one typeface. An existing font element keeps its
/// panose and charset attributes.
fn set_typeface(slot: &mut Option<TextFont>, font: Option<TextFont>) {
    match (slot.as_mut(), font) {
        (_, None) => *slot = None,
        (Some(current), Some(font)) => current.typeface = font.typeface,
        (None, font) => *slot = font,
    }
}

#[pymethods]
impl PyFont {
    #[getter]
    fn bold(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read(py, |properties| {
            properties.and_then(|properties| properties.bold)
        })
    }

    #[setter]
    fn set_bold(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.update(py, |properties| properties.bold = value)
    }

    #[getter]
    fn italic(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read(py, |properties| {
            properties.and_then(|properties| properties.italic)
        })
    }

    #[setter]
    fn set_italic(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.update(py, |properties| properties.italic = value)
    }

    #[getter]
    fn underline(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let underline = self.read(py, |properties| {
            properties.and_then(|properties| properties.underline)
        })?;
        underline
            .map(|underline| match underline {
                TextUnderline::None => Ok(PyBool::new(py, false).to_owned().into_any().unbind()),
                TextUnderline::Single => Ok(PyBool::new(py, true).to_owned().into_any().unbind()),
                underline => text_enum(py, "MSO_TEXT_UNDERLINE_TYPE", underline_value(underline)),
            })
            .transpose()
    }

    #[setter]
    fn set_underline(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let underline = match value {
            None => None,
            Some(value) if value.is_none() => None,
            Some(value) if value.is_instance_of::<PyBool>() => Some(if value.extract::<bool>()? {
                TextUnderline::Single
            } else {
                TextUnderline::None
            }),
            Some(value) => Some(underline_from_value(value.extract()?)?),
        };
        self.update(py, |properties| properties.underline = underline)
    }

    #[getter]
    fn strike(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read(py, |properties| {
            properties
                .and_then(|properties| properties.strike)
                .map(|strike| strike != TextStrike::None)
        })
    }

    #[setter]
    fn set_strike(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.update(py, |properties| {
            properties.strike = match value {
                None => None,
                Some(false) => Some(TextStrike::None),
                // A double strike already struck through stays double.
                Some(true) if properties.strike == Some(TextStrike::Double) => {
                    Some(TextStrike::Double)
                }
                Some(true) => Some(TextStrike::Single),
            };
        })
    }

    #[getter]
    fn all_caps(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read(py, |properties| {
            properties.and_then(|properties| properties.all_caps)
        })
    }

    #[setter]
    fn set_all_caps(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.update(py, |properties| {
            properties.all_caps = value;
            if value == Some(true) {
                properties.small_caps = None;
            }
        })
    }

    #[getter]
    fn name(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.read(py, |properties| {
            properties
                .and_then(|properties| properties.latin.as_ref())
                .map(|font| font.typeface.clone())
        })
    }

    #[setter]
    fn set_name(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        let font = typeface(value)?;
        self.update(py, |properties| set_typeface(&mut properties.latin, font))
    }

    /// The font colour as a `ColorFormat`, as python-pptx returns it.
    #[getter]
    fn color(&self, py: Python<'_>) -> PyResult<Py<PyColorFormat>> {
        self.validate(py)?;
        Py::new(
            py,
            PyColorFormat::new(
                self.presentation.clone_ref(py),
                self.path.clone(),
                ColorSource::Font { style: self.style },
            ),
        )
    }

    /// Assigning an `RGBColor` or a hex string sets `color.rgb` directly, and
    /// `None` removes the font fill.
    #[setter]
    fn set_color(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let color = match value {
            None => None,
            Some(value) if value.is_none() => None,
            Some(value) => Some(rgb_color(value, "color")?),
        };
        self.update(py, |properties| {
            let Some(color) = color else {
                properties.fill = None;
                return;
            };
            match properties.fill.as_mut() {
                // An existing sRGB colour keeps its transforms, such as `a:alpha`.
                Some(Fill::Solid(SolidFill {
                    color: Some(ColorChoice::Srgb { value, .. }),
                    ..
                })) => *value = color,
                Some(Fill::Solid(fill)) => fill.color = Some(ColorChoice::srgb(color)),
                _ => {
                    let mut fill = SolidFill::default();
                    fill.color = Some(ColorChoice::srgb(color));
                    properties.fill = Some(Fill::Solid(fill));
                }
            }
        })
    }

    /// The vertical offset as a fraction of the font size: positive raises
    /// the text (0.3 is PowerPoint's superscript), negative lowers it (-0.25
    /// is its subscript), and `None` inherits.
    #[getter]
    fn baseline(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        let baseline = self.read(py, |properties| {
            properties.and_then(|properties| properties.baseline.clone())
        })?;
        baseline
            .map(|value| {
                match value.strip_suffix('%') {
                    Some(percent) => percent.parse::<f64>().map(|percent| percent / 100.0),
                    None => value.parse::<f64>().map(|value| value / 100_000.0),
                }
                .map_err(|_| PyValueError::new_err(format!("baseline {value:?} is not a number")))
            })
            .transpose()
    }

    #[setter]
    fn set_baseline(&self, py: Python<'_>, value: Option<f64>) -> PyResult<()> {
        let baseline = value
            .map(|value| {
                if (-1.0..=1.0).contains(&value) {
                    Ok(((value * 100_000.0).round_ties_even() as i32).to_string())
                } else {
                    Err(PyValueError::new_err(format!(
                        "baseline must be from -1.0 to 1.0, got {value}"
                    )))
                }
            })
            .transpose()?;
        self.update(py, |properties| properties.baseline = baseline)
    }

    /// The character spacing as a `Length`, positive to expand and negative
    /// to condense, or `None` to inherit.
    #[getter]
    fn spacing(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let spacing = self.read(py, |properties| {
            properties.and_then(|properties| properties.spacing.clone())
        })?;
        match spacing {
            Some(TextPointValue::Centipoints(centipoints)) => {
                length_object(py, i64::from(centipoints) * EMU_PER_CENTIPOINT).map(Some)
            }
            _ => Ok(None),
        }
    }

    #[setter]
    fn set_spacing(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        let spacing = value
            .map(|emu| {
                let centipoints = emu / EMU_PER_CENTIPOINT;
                if (-MAX_FONT_SIZE..=MAX_FONT_SIZE).contains(&centipoints) {
                    Ok(TextPointValue::Centipoints(centipoints as i32))
                } else {
                    Err(PyValueError::new_err(
                        "character spacing must be from -4000 to 4000 points",
                    ))
                }
            })
            .transpose()?;
        self.update(py, |properties| properties.spacing = spacing)
    }

    /// The language tag of the text, such as `en-US` or `fr-FR`, which
    /// proofing and hyphenation use, or `None` to inherit.
    #[getter]
    fn language(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.read(py, |properties| {
            properties
                .and_then(CT_TextCharacterProperties::language)
                .map(str::to_owned)
        })
    }

    #[setter]
    fn set_language(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        if let Some(value) = value.as_deref()
            && (value.is_empty()
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'))
        {
            return Err(PyValueError::new_err(format!(
                "language must be a tag such as \"en-US\", got {value:?}"
            )));
        }
        self.update(py, |properties| properties.set_language(value.as_deref()))
    }

    /// The language as an `MSO_LANGUAGE_ID` member, as python-pptx reads it:
    /// `NONE` without a language. A tag no member stands for raises and
    /// names `font.language`, which reads any tag.
    #[getter]
    fn language_id(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let language = self.language(py)?;
        let module = py.import("rpptx.enum.lang")?;
        let Some(tag) = language else {
            return module
                .getattr("MSO_LANGUAGE_ID")?
                .getattr("NONE")
                .map(Bound::unbind);
        };
        match module
            .getattr("_MEMBERS")?
            .get_item(tag.to_ascii_lowercase())
        {
            Ok(member) => Ok(member.unbind()),
            Err(_) => Err(PyValueError::new_err(format!(
                "language {tag:?} has no MSO_LANGUAGE_ID member, read it with font.language"
            ))),
        }
    }

    /// Writes the tag an `MSO_LANGUAGE_ID` member stands for. `None` or
    /// `NONE` removes the language, as python-pptx does.
    #[setter]
    fn set_language_id(&self, py: Python<'_>, value: Option<i32>) -> PyResult<()> {
        let tag = match value {
            None | Some(0) => None,
            Some(value) => {
                let tags = py.import("rpptx.enum.lang")?.getattr("_TAGS")?;
                let tag = tags.get_item(value).map_err(|_| {
                    PyValueError::new_err(format!(
                        "language id {value} names no language tag, pass an MSO_LANGUAGE_ID \
                         member such as ENGLISH_US or set font.language = \"en-US\""
                    ))
                })?;
                Some(tag.extract::<String>()?)
            }
        };
        self.update(py, |properties| properties.set_language(tag.as_deref()))
    }

    /// The East Asian typeface (`a:ea`), or `None` to inherit.
    #[getter]
    fn east_asian_name(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.read(py, |properties| {
            properties
                .and_then(|properties| properties.east_asian.as_ref())
                .map(|font| font.typeface.clone())
        })
    }

    #[setter]
    fn set_east_asian_name(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        let font = typeface(value)?;
        self.update(py, |properties| {
            set_typeface(&mut properties.east_asian, font);
        })
    }

    /// The complex-script typeface (`a:cs`), used for Arabic, Hebrew, Thai
    /// and other complex scripts, or `None` to inherit.
    #[getter]
    fn complex_script_name(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.read(py, |properties| {
            properties
                .and_then(|properties| properties.complex_script.as_ref())
                .map(|font| font.typeface.clone())
        })
    }

    #[setter]
    fn set_complex_script_name(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        let font = typeface(value)?;
        self.update(py, |properties| {
            set_typeface(&mut properties.complex_script, font);
        })
    }

    #[getter]
    fn size(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |properties| {
            properties.and_then(|properties| properties.font_size)
        })?
        .map(|centipoints| length_object(py, i64::from(centipoints) * EMU_PER_CENTIPOINT))
        .transpose()
    }

    #[setter]
    fn set_size(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        let centipoints = value
            .map(|emu| {
                let centipoints = emu.div_euclid(EMU_PER_CENTIPOINT);
                if (MIN_FONT_SIZE..=MAX_FONT_SIZE).contains(&centipoints) {
                    Ok(centipoints as i32)
                } else {
                    Err(PyValueError::new_err(
                        "font size must be from 1 to 4000 points",
                    ))
                }
            })
            .transpose()?;
        self.update(py, |properties| properties.font_size = centipoints)
    }

    // Shape authoring: highlight and small capitals.

    /// The colour drawn behind the text, `a:highlight`, as an `RGBColor`, or
    /// `None` without one or for a theme or preset highlight colour.
    /// Assigning takes an `RGBColor`, a hex string with or without `#`, or a
    /// `(red, green, blue)` tuple, and `None` removes the highlight.
    #[getter]
    fn highlight_color(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let color = self.read(py, |properties| match properties?.highlight.as_ref()? {
            ColorChoice::Srgb { value, .. } => Some(value.components()),
            _ => None,
        })?;
        color
            .map(|[red, green, blue]| {
                py.import("rpptx.dml.color")?
                    .getattr("RGBColor")?
                    .call1((red, green, blue))
                    .map(Bound::unbind)
            })
            .transpose()
    }

    #[setter]
    fn set_highlight_color(
        &self,
        py: Python<'_>,
        value: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let color = match value {
            Some(value) if !value.is_none() => Some(rgb_color(value, "highlight_color")?),
            _ => None,
        };
        self.update(py, |properties| {
            properties.highlight = color.map(ColorChoice::srgb);
        })
    }

    /// Whether the text shows in small capitals, `cap="small"`: `True`,
    /// `False` for an explicit other capitalisation, or `None` to inherit.
    /// `all_caps` and `small_caps` share one attribute, so setting one to
    /// `True` clears the other.
    #[getter]
    fn small_caps(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read(py, |properties| {
            let properties = properties?;
            match (properties.small_caps, properties.all_caps) {
                (Some(true), _) => Some(true),
                (Some(false), _) | (None, Some(_)) => Some(false),
                (None, None) => None,
            }
        })
    }

    #[setter]
    fn set_small_caps(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.update(py, |properties| {
            properties.small_caps = value;
            if value == Some(true) {
                properties.all_caps = None;
            }
        })
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

// Master text styles (#312).

fn master_index(path: &ContentPath) -> PyResult<usize> {
    match crate::shape::part_of(path)? {
        rpptx::PartRef::Master(index) => Ok(index),
        _ => Err(PyIndexError::new_err("slide master index is missing")),
    }
}

fn validate_text_style(
    py: Python<'_>,
    presentation: &PyPresentation,
    path: &ContentPath,
) -> PyResult<()> {
    validate_path(py, presentation, path, "text style", ".text_styles")
}

/// Reads one level of a master text style, `None` when the master sets
/// nothing for it.
fn text_style_read<T>(
    py: Python<'_>,
    presentation: &Py<PyPresentation>,
    path: &ContentPath,
    style: rpptx::MasterTextStyle,
    level: usize,
    read: impl FnOnce(Option<&CT_TextParagraphProperties>) -> T,
) -> PyResult<T> {
    let presentation = presentation.borrow(py);
    validate_text_style(py, &presentation, path)?;
    let level = presentation
        .inner
        .master_text_style(master_index(path)?, style, level)
        .map_err(|error| rpptx_to_pyerr(py, error))?;
    Ok(read(level))
}

/// Changes one level of a master text style, creating it when absent.
fn text_style_update(
    py: Python<'_>,
    presentation: &Py<PyPresentation>,
    path: &ContentPath,
    style: rpptx::MasterTextStyle,
    level: usize,
    update: impl FnOnce(&mut CT_TextParagraphProperties),
) -> PyResult<()> {
    let mut presentation = presentation.borrow_mut(py);
    validate_text_style(py, &presentation, path)?;
    let level = presentation
        .inner
        .master_text_style_mut(master_index(path)?, style, level)
        .map_err(|error| rpptx_to_pyerr(py, error))?;
    update(level);
    Ok(())
}

/// A slide master's text styles (rpptx extension): `title`, `body` and
/// `other`, each indexed by paragraph level 0 to 8 as `paragraph.level`
/// numbers them.
#[pyclass(name = "MasterTextStyles")]
pub struct PyMasterTextStyles {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyMasterTextStyles {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn list(&self, py: Python<'_>, style: rpptx::MasterTextStyle) -> PyResult<Py<PyTextStyleList>> {
        validate_text_style(py, &self.presentation.borrow(py), &self.path)?;
        Py::new(
            py,
            PyTextStyleList {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                style,
            },
        )
    }
}

#[pymethods]
impl PyMasterTextStyles {
    /// The title placeholders' style.
    #[getter]
    fn title(&self, py: Python<'_>) -> PyResult<Py<PyTextStyleList>> {
        self.list(py, rpptx::MasterTextStyle::Title)
    }

    /// The body, subtitle and content placeholders' style.
    #[getter]
    fn body(&self, py: Python<'_>) -> PyResult<Py<PyTextStyleList>> {
        self.list(py, rpptx::MasterTextStyle::Body)
    }

    /// The style of text boxes and shapes that are not placeholders.
    #[getter]
    fn other(&self, py: Python<'_>) -> PyResult<Py<PyTextStyleList>> {
        self.list(py, rpptx::MasterTextStyle::Other)
    }
}

/// The nine levels of one master text style.
#[pyclass(name = "TextStyleList")]
pub struct PyTextStyleList {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    style: rpptx::MasterTextStyle,
}

#[pymethods]
impl PyTextStyleList {
    fn __len__(&self) -> usize {
        9
    }

    /// The level that paragraphs with `paragraph.level == level` follow.
    fn __getitem__(&self, py: Python<'_>, level: isize) -> PyResult<Py<PyTextStyleLevel>> {
        let level = normalize_index(level, 9, "text style level")?;
        Py::new(
            py,
            PyTextStyleLevel {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                style: self.style,
                level: level + 1,
            },
        )
    }
}

/// One level of a master text style: the font, bullet and indents of every
/// paragraph at that level that inherits it.
#[pyclass(name = "TextStyleLevel")]
pub struct PyTextStyleLevel {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    style: rpptx::MasterTextStyle,
    level: usize,
}

impl PyTextStyleLevel {
    fn read<T>(
        &self,
        py: Python<'_>,
        read: impl FnOnce(Option<&CT_TextParagraphProperties>) -> T,
    ) -> PyResult<T> {
        text_style_read(
            py,
            &self.presentation,
            &self.path,
            self.style,
            self.level,
            read,
        )
    }

    fn update(
        &self,
        py: Python<'_>,
        update: impl FnOnce(&mut CT_TextParagraphProperties),
    ) -> PyResult<()> {
        text_style_update(
            py,
            &self.presentation,
            &self.path,
            self.style,
            self.level,
            update,
        )
    }

    fn margin(
        &self,
        py: Python<'_>,
        value: impl FnOnce(&CT_TextParagraphProperties) -> Option<i32>,
    ) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |properties| properties.and_then(value))?
            .map(|emu| length_object(py, i64::from(emu)))
            .transpose()
    }
}

#[pymethods]
impl PyTextStyleLevel {
    /// The level's font: name, size, bold, italic and colour.
    #[getter]
    fn font(&self, py: Python<'_>) -> PyResult<Py<PyFont>> {
        self.read(py, |_| ())?;
        Py::new(
            py,
            PyFont {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                style: Some((self.style, self.level)),
            },
        )
    }

    /// The bullet character, `False` for no bullet, or `None` to inherit,
    /// as `paragraph.bullet` reads it. PowerPoint draws the character in the
    /// level's bullet font, Arial in the default template, and drops some
    /// characters rpptx still draws, such as `▪` on a Mac, so prefer `•` or
    /// `–`, which both draw alike.
    #[getter]
    fn bullet(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let bullet = self.read(py, |properties| properties.and_then(read_bullet))?;
        Ok(bullet_object(py, bullet))
    }

    #[setter]
    fn set_bullet(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let choice = bullet_choice(value)?;
        self.update(py, |properties| apply_bullet(properties, choice))
    }

    /// The bullet colour as an `RGBColor`, `None` when the bullet follows
    /// the text colour or uses a theme colour.
    #[getter]
    fn bullet_color(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let color = self.read(py, |properties| {
            match &properties?.bullet.as_ref()?.color.as_ref()?.color {
                ColorChoice::Srgb { value, .. } => Some(value.components()),
                _ => None,
            }
        })?;
        color
            .map(|[red, green, blue]| {
                py.import("rpptx.dml.color")?
                    .getattr("RGBColor")?
                    .call1((red, green, blue))
                    .map(Bound::unbind)
            })
            .transpose()
    }

    /// Sets the bullet colour from an `RGBColor`, a hex string or an
    /// `(r, g, b)` triple, or `None` to follow the text colour.
    #[setter]
    fn set_bullet_color(&self, py: Python<'_>, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let color = match value {
            Some(value) if !value.is_none() => Some(rgb_color(value, "bullet_color")?),
            _ => None,
        };
        self.update(py, |properties| {
            let mut bullet = properties.bullet.clone().unwrap_or_default();
            bullet.color = color.map(|color| TextBulletColor::new(ColorChoice::srgb(color)));
            properties.set_bullet(Some(bullet));
        })
    }

    /// The distance from the text frame's left inset to the text.
    #[getter]
    fn left_indent(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |properties| properties.left_margin)
    }

    #[setter]
    fn set_left_indent(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        let margin = margin_value(value, 0, "left indent")?;
        self.update(py, |properties| properties.left_margin = margin)
    }

    /// The first line's offset from the left indent, negative for a
    /// hanging bullet.
    #[getter]
    fn first_line_indent(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.margin(py, |properties| properties.indent)
    }

    #[setter]
    fn set_first_line_indent(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        let indent = margin_value(value, -MAX_TEXT_MARGIN, "first line indent")?;
        self.update(py, |properties| properties.indent = indent)
    }
}
