//! Fill, line, and colour formats, mirroring python-pptx `pptx.dml`.
//!
//! Shape fills, line fills, slide backgrounds, table cell fills, and cell
//! border fills share one fill model, so the formats read and write through a
//! single target.

use oxml_py_support::ContentPath;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;

use crate::presentation::PyPresentation;
use crate::shape::{length, shape_mut_at, shape_ref_at, slide_index};
use crate::table::{cell_mut_at, cell_ref_at};
use crate::{rpptx_to_pyerr, validate_path};

const MAX_LINE_WIDTH_EMU: i64 = 20_116_800;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyFillFormat>()?;
    module.add_class::<PyLineFormat>()?;
    module.add_class::<PyLineEndFormat>()?;
    module.add_class::<PyColorFormat>()?;
    Ok(())
}

/// The DrawingML fill one format object reads and writes.
///
/// `Line` and `CellBorder` also name the line a `LineFormat` reads and writes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum FillTarget {
    Shape,
    Line,
    Background,
    TableCell,
    CellBorder(rpptx::CellBorder),
}

impl FillTarget {
    fn suffix(self) -> &'static str {
        match self {
            Self::Shape | Self::TableCell => ".fill",
            Self::Line => ".line.fill",
            Self::Background => ".background.fill",
            Self::CellBorder(rpptx::CellBorder::Left) => ".border_left.fill",
            Self::CellBorder(rpptx::CellBorder::Right) => ".border_right.fill",
            Self::CellBorder(rpptx::CellBorder::Top) => ".border_top.fill",
            Self::CellBorder(rpptx::CellBorder::Bottom) => ".border_bottom.fill",
        }
    }

    fn line_suffix(self) -> &'static str {
        self.suffix()
            .strip_suffix(".fill")
            .expect("every fill suffix ends in .fill")
    }
}

/// Reads the shape line or cell border a line target names.
fn current_line(
    presentation: &rpptx::Presentation,
    path: &ContentPath,
    target: FillTarget,
) -> PyResult<Option<rpptx::CT_LineProperties>> {
    Ok(match target {
        FillTarget::CellBorder(edge) => cell_ref_at(presentation, path)
            .ok_or_else(|| PyIndexError::new_err("cell index out of range"))?
            .border(edge)
            .cloned(),
        _ => shape_ref_at(presentation, path)
            .ok_or_else(|| PyIndexError::new_err("shape index out of range"))?
            .line()
            .cloned(),
    })
}

/// Replaces the shape line or cell border a line target names.
fn write_line(
    py: Python<'_>,
    presentation: &mut rpptx::Presentation,
    path: &ContentPath,
    target: FillTarget,
    line: rpptx::CT_LineProperties,
) -> PyResult<()> {
    match target {
        FillTarget::CellBorder(edge) => {
            cell_mut_at(presentation, path)
                .ok_or_else(|| PyIndexError::new_err("cell index out of range"))?
                .set_border(edge, Some(line));
            Ok(())
        }
        _ => shape_mut_at(presentation, path)
            .ok_or_else(|| PyIndexError::new_err("shape index out of range"))?
            .set_line(line)
            .map_err(|error| rpptx_to_pyerr(py, error)),
    }
}

fn current_fill(
    presentation: &rpptx::Presentation,
    path: &ContentPath,
    target: FillTarget,
) -> PyResult<Option<rpptx::Fill>> {
    let missing = || PyIndexError::new_err("shape index out of range");
    Ok(match target {
        FillTarget::Shape => shape_ref_at(presentation, path)
            .ok_or_else(missing)?
            .fill()
            .cloned(),
        FillTarget::Line | FillTarget::CellBorder(_) => {
            current_line(presentation, path, target)?.and_then(|line| line.fill)
        }
        FillTarget::Background => presentation
            .slide(slide_index(path)?)
            .ok_or_else(|| PyIndexError::new_err("slide index out of range"))?
            .background_fill()
            .cloned(),
        FillTarget::TableCell => cell_ref_at(presentation, path)
            .ok_or_else(|| PyIndexError::new_err("cell index out of range"))?
            .fill()
            .cloned(),
    })
}

fn write_fill(
    py: Python<'_>,
    presentation: &mut rpptx::Presentation,
    path: &ContentPath,
    target: FillTarget,
    fill: rpptx::Fill,
) -> PyResult<()> {
    let missing = || PyIndexError::new_err("shape index out of range");
    let result = match target {
        FillTarget::Shape => shape_mut_at(presentation, path)
            .ok_or_else(missing)?
            .set_fill(fill),
        FillTarget::Line | FillTarget::CellBorder(_) => {
            let mut line = current_line(presentation, path, target)?.unwrap_or_default();
            line.fill = Some(fill);
            return write_line(py, presentation, path, target, line);
        }
        FillTarget::Background => {
            let index = slide_index(path)?;
            let direct = presentation
                .slide(index)
                .ok_or_else(|| PyIndexError::new_err("slide index out of range"))?
                .background_fill()
                .is_some();
            let mut slide = presentation
                .slide_mut(index)
                .ok_or_else(|| PyIndexError::new_err("slide index out of range"))?;
            if !direct {
                slide.remove_background();
            }
            slide.set_background(fill)
        }
        FillTarget::TableCell => {
            cell_mut_at(presentation, path)
                .ok_or_else(|| PyIndexError::new_err("cell index out of range"))?
                .set_fill(Some(fill));
            Ok(())
        }
    };
    result.map_err(|error| rpptx_to_pyerr(py, error))
}

/// Changes the direct line of a shape or the border of a cell. Without one,
/// the line is created only when `create` is set, so a removal never adds an
/// empty line.
fn update_line(
    py: Python<'_>,
    presentation: &mut rpptx::Presentation,
    path: &ContentPath,
    target: FillTarget,
    create: bool,
    change: impl FnOnce(&mut rpptx::CT_LineProperties),
) -> PyResult<()> {
    let mut line = match current_line(presentation, path, target)? {
        Some(line) => line,
        None if create => rpptx::CT_LineProperties::default(),
        None => return Ok(()),
    };
    change(&mut line);
    write_line(py, presentation, path, target, line)
}

fn dml_enum(py: Python<'_>, name: &str, value: i32) -> PyResult<Py<PyAny>> {
    py.import("rpptx.enum.dml")?
        .getattr(name)?
        .call1((value,))
        .map(Bound::unbind)
}

/// `MSO_LINE_DASH_STYLE` values, with the python-pptx mapping for the nine
/// members it shares.
fn dash_value(dash: rpptx::ST_PresetLineDashVal) -> i32 {
    use rpptx::ST_PresetLineDashVal as Dash;
    match dash {
        Dash::Solid => 1,
        Dash::SystemDash => 2,
        Dash::SystemDot => 3,
        Dash::Dash => 4,
        Dash::DashDot => 5,
        Dash::LargeDashDotDot => 6,
        Dash::LargeDash => 7,
        Dash::LargeDashDot => 8,
        Dash::SystemDashDot => 12,
        Dash::Dot => 13,
        Dash::SystemDashDotDot => 14,
    }
}

fn dash_from_value(value: i32) -> PyResult<rpptx::ST_PresetLineDashVal> {
    rpptx::ST_PresetLineDashVal::ALL
        .into_iter()
        .find(|dash| dash_value(*dash) == value)
        .ok_or_else(|| {
            PyValueError::new_err(
                "dash style must be an MSO_LINE_DASH_STYLE member other than DASH_STYLE_MIXED",
            )
        })
}

/// `MSO_ARROWHEAD_STYLE` values, as `MsoArrowheadStyle` numbers them.
fn end_type_value(kind: rpptx::LineEndType) -> i32 {
    match kind {
        rpptx::LineEndType::None => 1,
        rpptx::LineEndType::Triangle => 2,
        rpptx::LineEndType::Arrow => 3,
        rpptx::LineEndType::Stealth => 4,
        rpptx::LineEndType::Diamond => 5,
        rpptx::LineEndType::Oval => 6,
    }
}

fn end_type_from_value(value: i32) -> PyResult<rpptx::LineEndType> {
    Ok(match value {
        1 => rpptx::LineEndType::None,
        2 => rpptx::LineEndType::Triangle,
        3 => rpptx::LineEndType::Arrow,
        4 => rpptx::LineEndType::Stealth,
        5 => rpptx::LineEndType::Diamond,
        6 => rpptx::LineEndType::Oval,
        _ => {
            return Err(PyValueError::new_err(
                "line end type must be an MSO_ARROWHEAD_STYLE member",
            ));
        }
    })
}

/// `MSO_ARROWHEAD_WIDTH` and `MSO_ARROWHEAD_LENGTH` share these values.
fn end_size_value(size: rpptx::LineEndSize) -> i32 {
    match size {
        rpptx::LineEndSize::Small => 1,
        rpptx::LineEndSize::Medium => 2,
        rpptx::LineEndSize::Large => 3,
    }
}

fn end_size_from_value(value: i32, name: &str) -> PyResult<rpptx::LineEndSize> {
    Ok(match value {
        1 => rpptx::LineEndSize::Small,
        2 => rpptx::LineEndSize::Medium,
        3 => rpptx::LineEndSize::Large,
        _ => {
            return Err(PyValueError::new_err(format!(
                "line end size must be an {name} member"
            )));
        }
    })
}

fn fill_type_name(fill: Option<&rpptx::Fill>) -> &'static str {
    match fill {
        None => "_NoneFill",
        Some(rpptx::Fill::NoFill(_)) => "_NoFill",
        Some(rpptx::Fill::Solid(_)) => "_SolidFill",
        Some(rpptx::Fill::Gradient(_)) => "_GradFill",
        Some(rpptx::Fill::Pattern(_)) => "_PattFill",
        Some(rpptx::Fill::Blip(_)) => "_BlipFill",
    }
}

fn no_foreground(fill: Option<&rpptx::Fill>) -> PyErr {
    PyTypeError::new_err(format!(
        "fill type {} has no foreground color, call .solid() or .patterned() first",
        fill_type_name(fill)
    ))
}

fn with_rgb(color: Option<rpptx::ColorChoice>, rgb: rpptx::RgbColor) -> rpptx::ColorChoice {
    match color {
        Some(rpptx::ColorChoice::Srgb {
            transforms,
            raw_children,
            ..
        }) => rpptx::ColorChoice::Srgb {
            value: rgb,
            transforms,
            raw_children,
        },
        _ => rpptx::ColorChoice::srgb(rgb),
    }
}

/// A live view of one shape fill, line fill, or slide background fill.
#[pyclass(name = "FillFormat")]
pub struct PyFillFormat {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    target: FillTarget,
}

impl PyFillFormat {
    pub(crate) fn new(
        presentation: Py<PyPresentation>,
        path: ContentPath,
        target: FillTarget,
    ) -> Self {
        Self {
            presentation,
            path,
            target,
        }
    }

    fn fill(&self, py: Python<'_>) -> PyResult<Option<rpptx::Fill>> {
        let presentation = self.presentation.borrow(py);
        validate_path(py, &presentation, &self.path, "fill", self.target.suffix())?;
        current_fill(&presentation.inner, &self.path, self.target)
    }

    fn set(&self, py: Python<'_>, fill: rpptx::Fill) -> PyResult<()> {
        let mut presentation = self.presentation.borrow_mut(py);
        write_fill(py, &mut presentation.inner, &self.path, self.target, fill)
    }
}

#[pymethods]
impl PyFillFormat {
    #[getter(r#type)]
    fn fill_type(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let value = match self.fill(py)? {
            None => return Ok(None),
            Some(rpptx::Fill::Solid(_)) => 1,
            Some(rpptx::Fill::Pattern(_)) => 2,
            Some(rpptx::Fill::Gradient(_)) => 3,
            Some(rpptx::Fill::NoFill(_)) => 5,
            Some(rpptx::Fill::Blip(_)) => 6,
        };
        dml_enum(py, "MSO_FILL_TYPE", value).map(Some)
    }

    /// Sets a solid fill, keeping an existing solid fill and its colour.
    fn solid(&self, py: Python<'_>) -> PyResult<()> {
        if matches!(self.fill(py)?, Some(rpptx::Fill::Solid(_))) {
            return Ok(());
        }
        self.set(py, rpptx::Fill::Solid(rpptx::SolidFill::default()))
    }

    /// Sets an explicit no-fill, so what lies behind shows through.
    fn background(&self, py: Python<'_>) -> PyResult<()> {
        self.fill(py)?;
        self.set(py, rpptx::Fill::NoFill(rpptx::NoFill::default()))
    }

    #[getter]
    fn fore_color(&self, py: Python<'_>) -> PyResult<Py<PyColorFormat>> {
        let fill = self.fill(py)?;
        if !matches!(fill, Some(rpptx::Fill::Solid(_) | rpptx::Fill::Pattern(_))) {
            return Err(no_foreground(fill.as_ref()));
        }
        Py::new(
            py,
            PyColorFormat {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                target: self.target,
                solidify: false,
            },
        )
    }
}

/// A live view of the foreground colour of one fill.
#[pyclass(name = "ColorFormat")]
pub struct PyColorFormat {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    target: FillTarget,
    solidify: bool,
}

#[pymethods]
impl PyColorFormat {
    #[getter]
    fn rgb(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let presentation = self.presentation.borrow(py);
        validate_path(py, &presentation, &self.path, "color", self.target.suffix())?;
        let color = match current_fill(&presentation.inner, &self.path, self.target)? {
            Some(rpptx::Fill::Solid(fill)) => fill.color,
            Some(rpptx::Fill::Pattern(fill)) => fill.foreground,
            _ => None,
        };
        drop(presentation);
        let Some(rpptx::ColorChoice::Srgb { value, .. }) = color else {
            return Ok(None);
        };
        let [red, green, blue] = value.components();
        py.import("rpptx.dml.color")?
            .getattr("RGBColor")?
            .call1((red, green, blue))
            .map(|color| Some(color.unbind()))
    }

    #[setter]
    fn set_rgb(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let rgb_class = py.import("rpptx.dml.color")?.getattr("RGBColor")?;
        if !value.is_instance(&rgb_class)? {
            return Err(PyValueError::new_err(
                "assigned value must be type RGBColor",
            ));
        }
        let (red, green, blue) = value.extract::<(u8, u8, u8)>()?;
        let rgb = rpptx::RgbColor::new(red, green, blue);
        let mut presentation = self.presentation.borrow_mut(py);
        validate_path(py, &presentation, &self.path, "color", self.target.suffix())?;
        let fill = match current_fill(&presentation.inner, &self.path, self.target)? {
            Some(rpptx::Fill::Solid(mut fill)) => {
                fill.color = Some(with_rgb(fill.color.take(), rgb));
                rpptx::Fill::Solid(fill)
            }
            Some(rpptx::Fill::Pattern(mut fill)) => {
                fill.foreground = Some(with_rgb(fill.foreground.take(), rgb));
                rpptx::Fill::Pattern(fill)
            }
            _ if self.solidify => {
                let mut fill = rpptx::SolidFill::default();
                fill.color = Some(rpptx::ColorChoice::srgb(rgb));
                rpptx::Fill::Solid(fill)
            }
            other => return Err(no_foreground(other.as_ref())),
        };
        write_fill(py, &mut presentation.inner, &self.path, self.target, fill)
    }
}

/// A live view of the outline of one shape, picture, or connector, or of one
/// table cell border.
#[pyclass(name = "LineFormat")]
pub struct PyLineFormat {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    target: FillTarget,
}

impl PyLineFormat {
    /// Creates a view of a shape line, with `FillTarget::Line`, or of a cell
    /// border, with `FillTarget::CellBorder`.
    pub(crate) fn new(
        presentation: Py<PyPresentation>,
        path: ContentPath,
        target: FillTarget,
    ) -> Self {
        Self {
            presentation,
            path,
            target,
        }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<()> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "line",
            self.target.line_suffix(),
        )
    }
}

#[pymethods]
impl PyLineFormat {
    /// Returns the line colour. Setting its `rgb` makes the line fill solid.
    #[getter]
    fn color(&self, py: Python<'_>) -> PyResult<Py<PyColorFormat>> {
        self.validate(py)?;
        Py::new(
            py,
            PyColorFormat {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                target: self.target,
                solidify: true,
            },
        )
    }

    #[getter]
    fn fill(&self, py: Python<'_>) -> PyResult<Py<PyFillFormat>> {
        self.validate(py)?;
        Py::new(
            py,
            PyFillFormat::new(
                self.presentation.clone_ref(py),
                self.path.clone(),
                self.target,
            ),
        )
    }

    #[getter]
    fn width(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.validate(py)?;
        let width = current_line(&self.presentation.borrow(py).inner, &self.path, self.target)?
            .and_then(|line| line.width)
            .unwrap_or(0);
        length(py, Some(rpptx::Emu(i64::from(width))))
    }

    #[setter]
    fn set_width(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        self.validate(py)?;
        let width = value.unwrap_or(0);
        if !(0..=MAX_LINE_WIDTH_EMU).contains(&width) {
            return Err(PyValueError::new_err(format!(
                "line width must be between 0 and {MAX_LINE_WIDTH_EMU} EMU, got {width}"
            )));
        }
        let mut presentation = self.presentation.borrow_mut(py);
        update_line(
            py,
            &mut presentation.inner,
            &self.path,
            self.target,
            true,
            |line| {
                line.width = Some(width as u32);
            },
        )
    }

    /// Returns the preset dash, or `None` without one or with a custom dash.
    #[getter]
    fn dash_style(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.validate(py)?;
        let line = current_line(&self.presentation.borrow(py).inner, &self.path, self.target)?;
        let dash = line.as_ref().and_then(|line| match &line.dash {
            Some(rpptx::LineDash::Preset(dash)) => Some(dash.value),
            _ => None,
        });
        dash.map(|dash| dml_enum(py, "MSO_LINE_DASH_STYLE", dash_value(dash)))
            .transpose()
    }

    /// Writes `a:prstDash`. `None` removes a preset or custom dash.
    #[setter]
    fn set_dash_style(&self, py: Python<'_>, value: Option<i32>) -> PyResult<()> {
        self.validate(py)?;
        let dash = value.map(dash_from_value).transpose()?;
        let mut presentation = self.presentation.borrow_mut(py);
        update_line(
            py,
            &mut presentation.inner,
            &self.path,
            self.target,
            dash.is_some(),
            |line| {
                line.dash = match (dash, line.dash.take()) {
                    (None, _) => None,
                    (Some(value), Some(rpptx::LineDash::Preset(mut preset))) => {
                        preset.value = value;
                        Some(rpptx::LineDash::Preset(preset))
                    }
                    (Some(value), _) => {
                        Some(rpptx::LineDash::Preset(rpptx::PresetDash::new(value)))
                    }
                };
            },
        )
    }

    /// Returns the `a:headEnd`, the end at the first point of the line.
    #[getter]
    fn head_end(&self, py: Python<'_>) -> PyResult<Py<PyLineEndFormat>> {
        self.line_end(py, false)
    }

    /// Returns the `a:tailEnd`, the end at the last point of the line.
    #[getter]
    fn tail_end(&self, py: Python<'_>) -> PyResult<Py<PyLineEndFormat>> {
        self.line_end(py, true)
    }
}

impl PyLineFormat {
    fn line_end(&self, py: Python<'_>, tail: bool) -> PyResult<Py<PyLineEndFormat>> {
        self.validate(py)?;
        Py::new(
            py,
            PyLineEndFormat {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                target: self.target,
                tail,
            },
        )
    }
}

/// A live view of one end of a line, its `a:headEnd` or `a:tailEnd`.
///
/// Each property is `None` when its attribute is absent. Assigning `None`
/// removes the attribute, and an end left with no attribute is removed. An
/// `a:ln` left empty is kept, as python-pptx keeps it.
#[pyclass(name = "LineEndFormat")]
pub struct PyLineEndFormat {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    target: FillTarget,
    tail: bool,
}

impl PyLineEndFormat {
    fn end(&self, py: Python<'_>) -> PyResult<Option<rpptx::LineEnd>> {
        let presentation = self.presentation.borrow(py);
        let end = if self.tail { ".tail_end" } else { ".head_end" };
        let suffix = format!("{}{end}", self.target.line_suffix());
        validate_path(py, &presentation, &self.path, "line end", &suffix)?;
        let line = current_line(&presentation.inner, &self.path, self.target)?;
        Ok(line.and_then(|line| {
            if self.tail {
                line.tail_end.clone()
            } else {
                line.head_end.clone()
            }
        }))
    }

    fn update(
        &self,
        py: Python<'_>,
        create: bool,
        change: impl FnOnce(&mut rpptx::LineEnd),
    ) -> PyResult<()> {
        self.end(py)?;
        let tail = self.tail;
        let mut presentation = self.presentation.borrow_mut(py);
        update_line(
            py,
            &mut presentation.inner,
            &self.path,
            self.target,
            create,
            |line| {
                let slot = if tail {
                    &mut line.tail_end
                } else {
                    &mut line.head_end
                };
                if slot.is_none() && !create {
                    return;
                }
                let mut end = slot.take().unwrap_or_default();
                change(&mut end);
                *slot = (end != rpptx::LineEnd::default()).then_some(end);
            },
        )
    }
}

#[pymethods]
impl PyLineEndFormat {
    #[getter(r#type)]
    fn end_type(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.end(py)?
            .and_then(|end| end.kind)
            .map(|kind| dml_enum(py, "MSO_ARROWHEAD_STYLE", end_type_value(kind)))
            .transpose()
    }

    /// Writes `type`. `NONE` writes `type="none"`, as PowerPoint scripting does.
    #[setter(r#type)]
    fn set_end_type(&self, py: Python<'_>, value: Option<i32>) -> PyResult<()> {
        let kind = value.map(end_type_from_value).transpose()?;
        self.update(py, kind.is_some(), |end| end.kind = kind)
    }

    #[getter]
    fn width(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.end(py)?
            .and_then(|end| end.width)
            .map(|size| dml_enum(py, "MSO_ARROWHEAD_WIDTH", end_size_value(size)))
            .transpose()
    }

    #[setter]
    fn set_width(&self, py: Python<'_>, value: Option<i32>) -> PyResult<()> {
        let size = value
            .map(|value| end_size_from_value(value, "MSO_ARROWHEAD_WIDTH"))
            .transpose()?;
        self.update(py, size.is_some(), |end| end.width = size)
    }

    #[getter]
    fn length(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.end(py)?
            .and_then(|end| end.length)
            .map(|size| dml_enum(py, "MSO_ARROWHEAD_LENGTH", end_size_value(size)))
            .transpose()
    }

    #[setter]
    fn set_length(&self, py: Python<'_>, value: Option<i32>) -> PyResult<()> {
        let size = value
            .map(|value| end_size_from_value(value, "MSO_ARROWHEAD_LENGTH"))
            .transpose()?;
        self.update(py, size.is_some(), |end| end.length = size)
    }
}
