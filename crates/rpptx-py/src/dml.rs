//! Fill, line, shadow, and colour formats, mirroring python-pptx `pptx.dml`.
//!
//! Shape fills, line fills, and slide backgrounds share one fill model, so
//! the three formats read and write through a single target.

use oxml_py_support::ContentPath;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;

use crate::presentation::PyPresentation;
use crate::shape::{
    ANGLE_UNITS_PER_DEGREE, ANGLE_UNITS_PER_TURN, MAX_COORDINATE, length, shape_mut_at,
    shape_ref_at, slide_index,
};
use crate::{rpptx_to_pyerr, validate_path};

const MAX_LINE_WIDTH_EMU: i64 = 20_116_800;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyFillFormat>()?;
    module.add_class::<PyLineFormat>()?;
    module.add_class::<PyColorFormat>()?;
    module.add_class::<PyShadowFormat>()?;
    Ok(())
}

/// The DrawingML fill one format object reads and writes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum FillTarget {
    Shape,
    Line,
    Background,
}

impl FillTarget {
    fn suffix(self) -> &'static str {
        match self {
            Self::Shape => ".fill",
            Self::Line => ".line.fill",
            Self::Background => ".background.fill",
        }
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
        FillTarget::Line => shape_ref_at(presentation, path)
            .ok_or_else(missing)?
            .line()
            .and_then(|line| line.fill.clone()),
        FillTarget::Background => presentation
            .slide(slide_index(path)?)
            .ok_or_else(|| PyIndexError::new_err("slide index out of range"))?
            .background_fill()
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
        FillTarget::Line => {
            let mut line = shape_ref_at(presentation, path)
                .ok_or_else(missing)?
                .line()
                .cloned()
                .unwrap_or_default();
            line.fill = Some(fill);
            shape_mut_at(presentation, path)
                .ok_or_else(missing)?
                .set_line(line)
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
    };
    result.map_err(|error| rpptx_to_pyerr(py, error))
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
        py.import("rpptx.enum.dml")?
            .getattr("MSO_FILL_TYPE")?
            .call1((value,))
            .map(|member| Some(member.unbind()))
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
                source: ColorSource::Fill {
                    target: self.target,
                    solidify: false,
                },
            },
        )
    }
}

/// The colour one colour format reads and writes.
#[derive(Clone, Copy)]
enum ColorSource {
    /// The foreground of a fill. `solidify` makes a set colour a solid fill.
    Fill { target: FillTarget, solidify: bool },
    /// The colour of the outer shadow.
    Shadow,
}

impl ColorSource {
    fn suffix(self) -> &'static str {
        match self {
            Self::Fill { target, .. } => target.suffix(),
            Self::Shadow => ".shadow.color",
        }
    }
}

/// A live view of the foreground colour of one fill, or of a shadow colour.
#[pyclass(name = "ColorFormat")]
pub struct PyColorFormat {
    presentation: Py<PyPresentation>,
    path: ContentPath,
    source: ColorSource,
}

#[pymethods]
impl PyColorFormat {
    #[getter]
    fn rgb(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let presentation = self.presentation.borrow(py);
        validate_path(py, &presentation, &self.path, "color", self.source.suffix())?;
        let color = match self.source {
            ColorSource::Fill { target, .. } => {
                match current_fill(&presentation.inner, &self.path, target)? {
                    Some(rpptx::Fill::Solid(fill)) => fill.color,
                    Some(rpptx::Fill::Pattern(fill)) => fill.foreground,
                    _ => None,
                }
            }
            ColorSource::Shadow => {
                current_shadow(&presentation.inner, &self.path)?.and_then(|shadow| shadow.color)
            }
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
        validate_path(py, &presentation, &self.path, "color", self.source.suffix())?;
        let (target, solidify) = match self.source {
            ColorSource::Fill { target, solidify } => (target, solidify),
            ColorSource::Shadow => {
                return edit_shadow(py, &mut presentation.inner, &self.path, |shadow| {
                    let color = shadow_rgb(shadow.color.take(), rgb);
                    shadow.replace_color(color);
                });
            }
        };
        let fill = match current_fill(&presentation.inner, &self.path, target)? {
            Some(rpptx::Fill::Solid(mut fill)) => {
                fill.color = Some(with_rgb(fill.color.take(), rgb));
                rpptx::Fill::Solid(fill)
            }
            Some(rpptx::Fill::Pattern(mut fill)) => {
                fill.foreground = Some(with_rgb(fill.foreground.take(), rgb));
                rpptx::Fill::Pattern(fill)
            }
            _ if solidify => {
                let mut fill = rpptx::SolidFill::default();
                fill.color = Some(rpptx::ColorChoice::srgb(rgb));
                rpptx::Fill::Solid(fill)
            }
            other => return Err(no_foreground(other.as_ref())),
        };
        write_fill(py, &mut presentation.inner, &self.path, target, fill)
    }
}

/// A live view of the outline of one shape, picture, or connector.
#[pyclass(name = "LineFormat")]
pub struct PyLineFormat {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyLineFormat {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn validate(&self, py: Python<'_>) -> PyResult<()> {
        validate_path(
            py,
            &self.presentation.borrow(py),
            &self.path,
            "line",
            ".line",
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
                source: ColorSource::Fill {
                    target: FillTarget::Line,
                    solidify: true,
                },
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
                FillTarget::Line,
            ),
        )
    }

    #[getter]
    fn width(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.validate(py)?;
        let width = shape_ref_at(&self.presentation.borrow(py).inner, &self.path)
            .ok_or_else(|| PyIndexError::new_err("shape index out of range"))?
            .line()
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
        let missing = || PyIndexError::new_err("shape index out of range");
        let mut line = shape_ref_at(&presentation.inner, &self.path)
            .ok_or_else(missing)?
            .line()
            .cloned()
            .unwrap_or_default();
        line.width = Some(width as u32);
        shape_mut_at(&mut presentation.inner, &self.path)
            .ok_or_else(missing)?
            .set_line(line)
            .map_err(|error| rpptx_to_pyerr(py, error))
    }
}

const DEFAULT_SHADOW_BLUR_RADIUS: i64 = 50_800;
const DEFAULT_SHADOW_DISTANCE: i64 = 38_100;
const DEFAULT_SHADOW_DIRECTION: i32 = 2_700_000;
const DEFAULT_SHADOW_ALPHA: i32 = 40_000;
const OPAQUE_ALPHA: f64 = 100_000.0;
const RECT_ALIGNMENTS: &str = "tl, t, tr, l, ctr, r, bl, b, br";

fn current_effects(
    presentation: &rpptx::Presentation,
    path: &ContentPath,
) -> PyResult<Option<rpptx::CT_EffectList>> {
    Ok(shape_ref_at(presentation, path)
        .ok_or_else(|| PyIndexError::new_err("shape index out of range"))?
        .effects()
        .cloned())
}

fn current_shadow(
    presentation: &rpptx::Presentation,
    path: &ContentPath,
) -> PyResult<Option<rpptx::CT_OuterShadowEffect>> {
    Ok(current_effects(presentation, path)?.and_then(|effects| effects.outer_shadow))
}

fn write_effects(
    py: Python<'_>,
    presentation: &mut rpptx::Presentation,
    path: &ContentPath,
    effects: Option<rpptx::CT_EffectList>,
) -> PyResult<()> {
    shape_mut_at(presentation, path)
        .ok_or_else(|| PyIndexError::new_err("shape index out of range"))?
        .set_effects(effects)
        .map_err(|error| rpptx_to_pyerr(py, error))
}

/// The outer shadow PowerPoint for Mac writes for its Offset Diagonal
/// Bottom Right preset, `msoShadow21`: preset black at 40% opacity, with a
/// 4 pt blur, 3 pt away at 45 degrees.
fn default_outer_shadow() -> rpptx::CT_OuterShadowEffect {
    let mut shadow = rpptx::CT_OuterShadowEffect::default();
    shadow.blur_radius = Some(DEFAULT_SHADOW_BLUR_RADIUS);
    shadow.distance = Some(DEFAULT_SHADOW_DISTANCE);
    shadow.direction = Some(rpptx::Angle(DEFAULT_SHADOW_DIRECTION));
    shadow.alignment = Some(rpptx::RectAlignment::TopLeft);
    shadow.rotate_with_shape = Some(false);
    shadow.color = Some(rpptx::ColorChoice::Preset {
        value: "black".to_owned(),
        transforms: vec![rpptx::ColorTransform::Alpha(rpptx::Percent1000(
            DEFAULT_SHADOW_ALPHA,
        ))],
        raw_children: Default::default(),
    });
    shadow
}

/// Changes the outer shadow, first adding PowerPoint's default one when the
/// shape has none. Sibling effects in the list are kept.
fn edit_shadow(
    py: Python<'_>,
    presentation: &mut rpptx::Presentation,
    path: &ContentPath,
    change: impl FnOnce(&mut rpptx::CT_OuterShadowEffect),
) -> PyResult<()> {
    let mut effects = current_effects(presentation, path)?.unwrap_or_default();
    change(
        effects
            .outer_shadow
            .get_or_insert_with(default_outer_shadow),
    );
    write_effects(py, presentation, path, Some(effects))
}

/// Gives a shadow colour a new sRGB value. The opacity survives a change of
/// colour kind, which a theme or preset colour's other transforms do not.
fn shadow_rgb(color: Option<rpptx::ColorChoice>, rgb: rpptx::RgbColor) -> rpptx::ColorChoice {
    if let Some(color @ rpptx::ColorChoice::Srgb { .. }) = color {
        return with_rgb(Some(color), rgb);
    }
    rpptx::ColorChoice::Srgb {
        value: rgb,
        transforms: color
            .iter()
            .flat_map(rpptx::ColorChoice::transforms)
            .filter(|transform| matches!(transform, rpptx::ColorTransform::Alpha(_)))
            .copied()
            .collect(),
        raw_children: Default::default(),
    }
}

fn color_transforms_mut(color: &mut rpptx::ColorChoice) -> &mut Vec<rpptx::ColorTransform> {
    match color {
        rpptx::ColorChoice::Srgb { transforms, .. }
        | rpptx::ColorChoice::Scheme { transforms, .. }
        | rpptx::ColorChoice::System { transforms, .. }
        | rpptx::ColorChoice::Preset { transforms, .. } => transforms,
    }
}

fn check_shadow_length(name: &str, value: i64) -> PyResult<i64> {
    if (0..=MAX_COORDINATE).contains(&value) {
        Ok(value)
    } else {
        Err(PyValueError::new_err(format!(
            "shadow {name} must be between 0 and {MAX_COORDINATE} EMU, got {value}"
        )))
    }
}

/// A live view of the shadow of one shape, picture, connector, or group.
///
/// `inherit` is python-pptx's: whether the shape has no `a:effectLst` of
/// its own and so takes its theme effect. The other properties read and
/// write the shape's own `a:outerShdw`. They read `None` when the shape has
/// none, and a property the element omits reads as its schema default.
/// Setting one on a shape without an outer shadow first adds PowerPoint's
/// Offset Diagonal Bottom Right shadow, preset black at 40% opacity with a
/// 4 pt blur, 3 pt away at 45 degrees.
#[pyclass(name = "ShadowFormat")]
pub struct PyShadowFormat {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyShadowFormat {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn read<T>(
        &self,
        py: Python<'_>,
        read: impl FnOnce(Option<rpptx::CT_EffectList>) -> T,
    ) -> PyResult<T> {
        let presentation = self.presentation.borrow(py);
        validate_path(py, &presentation, &self.path, "shadow", ".shadow")?;
        Ok(read(current_effects(&presentation.inner, &self.path)?))
    }

    fn read_shadow<T>(
        &self,
        py: Python<'_>,
        read: impl FnOnce(&rpptx::CT_OuterShadowEffect) -> T,
    ) -> PyResult<Option<T>> {
        self.read(py, |effects| {
            effects.and_then(|effects| effects.outer_shadow.as_ref().map(read))
        })
    }

    fn write(&self, py: Python<'_>, effects: Option<rpptx::CT_EffectList>) -> PyResult<()> {
        let mut presentation = self.presentation.borrow_mut(py);
        validate_path(py, &presentation, &self.path, "shadow", ".shadow")?;
        write_effects(py, &mut presentation.inner, &self.path, effects)
    }

    fn edit(
        &self,
        py: Python<'_>,
        change: impl FnOnce(&mut rpptx::CT_OuterShadowEffect),
    ) -> PyResult<()> {
        let mut presentation = self.presentation.borrow_mut(py);
        validate_path(py, &presentation, &self.path, "shadow", ".shadow")?;
        edit_shadow(py, &mut presentation.inner, &self.path, change)
    }
}

#[pymethods]
impl PyShadowFormat {
    /// Whether the shape takes its effects from the theme.
    ///
    /// Setting `True` removes the shape's `a:effectLst`, and with it every
    /// effect of its own. Setting `False` adds an empty one when absent, so
    /// no effect shows, as python-pptx does.
    #[getter]
    fn inherit(&self, py: Python<'_>) -> PyResult<bool> {
        self.read(py, |effects| effects.is_none())
    }

    #[setter]
    fn set_inherit(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let effects = self.read(py, |effects| effects)?;
        match (value, effects) {
            (true, None) | (false, Some(_)) => Ok(()),
            (true, Some(_)) => self.write(py, None),
            (false, None) => self.write(py, Some(rpptx::CT_EffectList::default())),
        }
    }

    /// Whether the shape has an outer shadow of its own.
    ///
    /// A theme shadow reached through `inherit` is not reported. Setting
    /// `False` removes the outer shadow and keeps the effect list, so the
    /// theme shadow does not return either. On a shape without an outer
    /// shadow of its own it changes nothing.
    #[getter]
    fn visible(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.read_shadow(py, |_| ())?.is_some())
    }

    #[setter]
    fn set_visible(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        if value {
            return self.edit(py, |_| {});
        }
        let Some(mut effects) = self.read(py, |effects| effects)? else {
            return Ok(());
        };
        if effects.outer_shadow.take().is_none() {
            return Ok(());
        }
        self.write(py, Some(effects))
    }

    #[getter]
    fn color(&self, py: Python<'_>) -> PyResult<Py<PyColorFormat>> {
        self.read(py, |_| ())?;
        Py::new(
            py,
            PyColorFormat {
                presentation: self.presentation.clone_ref(py),
                path: self.path.clone(),
                source: ColorSource::Shadow,
            },
        )
    }

    /// The shadow opacity from 0.0, transparent, to 1.0, opaque.
    ///
    /// A colour rpptx does not model, `a:scrgbClr` or `a:hslClr`, reads
    /// `None` and refuses a new opacity until `color.rgb` replaces it.
    #[getter]
    fn alpha(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        let alpha = self.read_shadow(py, |shadow| {
            if shadow.has_unmodelled_color() {
                return None;
            }
            Some(
                shadow
                    .color
                    .iter()
                    .flat_map(rpptx::ColorChoice::transforms)
                    .rev()
                    .find_map(|transform| match transform {
                        rpptx::ColorTransform::Alpha(value) => {
                            Some(f64::from(value.0) / OPAQUE_ALPHA)
                        }
                        _ => None,
                    })
                    .unwrap_or(1.0),
            )
        })?;
        Ok(alpha.flatten())
    }

    #[setter]
    fn set_alpha(&self, py: Python<'_>, value: f64) -> PyResult<()> {
        if !(0.0..=1.0).contains(&value) {
            return Err(PyValueError::new_err(format!(
                "shadow alpha must be between 0.0 and 1.0, got {value}"
            )));
        }
        if self.read_shadow(py, rpptx::CT_OuterShadowEffect::has_unmodelled_color)? == Some(true) {
            return Err(PyValueError::new_err(
                "shadow colour is an a:scrgbClr or a:hslClr, which rpptx does not model, \
                 set color.rgb first",
            ));
        }
        let alpha = (value * OPAQUE_ALPHA).round_ties_even() as i32;
        self.edit(py, |shadow| {
            let color = shadow
                .color
                .get_or_insert_with(|| rpptx::ColorChoice::srgb(rpptx::RgbColor::new(0, 0, 0)));
            let transforms = color_transforms_mut(color);
            transforms.retain(|transform| !matches!(transform, rpptx::ColorTransform::Alpha(_)));
            if alpha < OPAQUE_ALPHA as i32 {
                transforms.push(rpptx::ColorTransform::Alpha(rpptx::Percent1000(alpha)));
            }
        })
    }

    #[getter]
    fn blur_radius(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let blur = self.read_shadow(py, |shadow| shadow.blur_radius.unwrap_or(0))?;
        length(py, blur.map(rpptx::Emu))
    }

    #[setter]
    fn set_blur_radius(&self, py: Python<'_>, value: i64) -> PyResult<()> {
        let value = check_shadow_length("blur_radius", value)?;
        self.edit(py, |shadow| shadow.blur_radius = Some(value))
    }

    #[getter]
    fn distance(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let distance = self.read_shadow(py, |shadow| shadow.distance.unwrap_or(0))?;
        length(py, distance.map(rpptx::Emu))
    }

    #[setter]
    fn set_distance(&self, py: Python<'_>, value: i64) -> PyResult<()> {
        let value = check_shadow_length("distance", value)?;
        self.edit(py, |shadow| shadow.distance = Some(value))
    }

    /// The direction the shadow is cast in, in degrees clockwise from the
    /// positive x axis, normalized to `0.0 <= direction < 360.0`.
    #[getter]
    fn direction(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        self.read_shadow(py, |shadow| {
            f64::from(shadow.direction.unwrap_or_default().0) / ANGLE_UNITS_PER_DEGREE
        })
    }

    #[setter]
    fn set_direction(&self, py: Python<'_>, value: f64) -> PyResult<()> {
        if !value.is_finite() {
            return Err(PyValueError::new_err(
                "shadow direction must be a finite number of degrees",
            ));
        }
        let units = ((value * ANGLE_UNITS_PER_DEGREE).round_ties_even() as i64)
            .rem_euclid(ANGLE_UNITS_PER_TURN);
        let angle = rpptx::Angle(i32::try_from(units).expect("a normalized angle fits in i32"));
        self.edit(py, |shadow| shadow.direction = Some(angle))
    }

    /// The `ST_RectAlignment` token the shadow is anchored at: `tl`, `t`,
    /// `tr`, `l`, `ctr`, `r`, `bl`, `b`, or `br`.
    #[getter]
    fn align(&self, py: Python<'_>) -> PyResult<Option<&'static str>> {
        self.read_shadow(py, |shadow| {
            shadow
                .alignment
                .unwrap_or(rpptx::RectAlignment::Bottom)
                .as_str()
        })
    }

    #[setter]
    fn set_align(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        let alignment = rpptx::RectAlignment::parse(value).ok_or_else(|| {
            PyValueError::new_err(format!(
                "shadow align must be one of {RECT_ALIGNMENTS}, got {value:?}"
            ))
        })?;
        self.edit(py, |shadow| shadow.alignment = Some(alignment))
    }

    #[getter]
    fn rotate_with_shape(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read_shadow(py, |shadow| shadow.rotate_with_shape.unwrap_or(true))
    }

    #[setter]
    fn set_rotate_with_shape(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        self.edit(py, |shadow| shadow.rotate_with_shape = Some(value))
    }
}
