use oxml_py_support::{ContentPath, PathSeg};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyTuple};

use crate::presentation::PyPresentation;
use crate::shape::slide_index;
use crate::{rpptx_to_pyerr, validate_path};

/// The python-pptx `XL_CHART_TYPE` value of each chart family `add_chart`
/// authors, the value python-pptx reads back from the written chart.
const CHART_TYPES: [(i64, rpptx::ChartKind); 7] = [
    (1, rpptx::ChartKind::Area),
    (51, rpptx::ChartKind::Bar),
    (-4120, rpptx::ChartKind::Doughnut),
    (65, rpptx::ChartKind::Line),
    (5, rpptx::ChartKind::Pie),
    (81, rpptx::ChartKind::Radar),
    (74, rpptx::ChartKind::Scatter),
];

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyChart>()?;
    module.add_class::<PyChartSeries>()?;
    Ok(())
}

/// Reads an `XL_CHART_TYPE` member, or its integer value, that `add_chart`
/// supports.
pub(crate) fn chart_kind(chart_type: &Bound<'_, PyAny>) -> PyResult<rpptx::ChartKind> {
    let value = chart_type.extract::<i64>()?;
    CHART_TYPES
        .iter()
        .find_map(|(candidate, kind)| (*candidate == value).then_some(*kind))
        .ok_or_else(|| PyValueError::new_err(format!("unsupported XL_CHART_TYPE value {value}")))
}

/// Reads a `CategoryChartData` or `XyChartData` into the facade's chart data.
pub(crate) fn chart_data(chart_data: &Bound<'_, PyAny>) -> PyResult<rpptx::ChartData> {
    if !chart_data.hasattr("_data")? {
        return Err(PyTypeError::new_err(
            "chart_data must be a CategoryChartData or an XyChartData",
        ));
    }
    let (categories, series, number_format) =
        chart_data
            .call_method0("_data")?
            .extract::<(Vec<String>, Vec<(String, Vec<f64>)>, Option<String>)>()?;
    Ok(rpptx::ChartData {
        categories,
        series,
        number_format,
        ..rpptx::ChartData::default()
    })
}

/// The chart of a graphic frame, read from its chart part on each access.
#[pyclass(name = "Chart")]
pub struct PyChart {
    presentation: Py<PyPresentation>,
    path: ContentPath,
}

impl PyChart {
    pub(crate) fn new(presentation: Py<PyPresentation>, path: ContentPath) -> Self {
        Self { presentation, path }
    }

    fn shape_path(&self) -> Vec<usize> {
        self.path
            .segs
            .iter()
            .filter_map(|segment| match segment {
                PathSeg::Shape(index) => Some(*index),
                _ => None,
            })
            .collect()
    }

    fn read(&self, py: Python<'_>) -> PyResult<(Option<rpptx::ChartKind>, rpptx::ChartData)> {
        let presentation = self.presentation.borrow(py);
        validate_path(py, &presentation, &self.path, "chart", ".chart")?;
        presentation
            .inner
            .chart(slide_index(&self.path)?, &self.shape_path())
            .map_err(|error| rpptx_to_pyerr(py, error))
    }
}

#[pymethods]
impl PyChart {
    /// The `XL_CHART_TYPE` member, or `None` for a chart `add_chart` cannot
    /// author, such as a stacked or 3-D chart.
    #[getter]
    fn chart_type(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let Some(kind) = self.read(py)?.0 else {
            return Ok(None);
        };
        let value = CHART_TYPES
            .iter()
            .find_map(|(value, candidate)| (*candidate == kind).then_some(*value))
            .expect("every chart kind has a chart type");
        py.import("rpptx.enum.chart")?
            .getattr("XL_CHART_TYPE")?
            .call1((value,))
            .map(|member| Some(member.unbind()))
    }

    /// The category labels of the first series, or its x values as text in an
    /// XY chart.
    #[getter]
    fn categories<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.read(py)?.1.categories)
    }

    /// Replaces the categories and series of the chart and its embedded
    /// workbook with a `CategoryChartData` or `XyChartData`, as python-pptx
    /// `replace_data` does. Remaining series keep their formatting, and an
    /// added series takes the formatting of the last one with its own colour.
    fn replace_data(&self, py: Python<'_>, chart_data: &Bound<'_, PyAny>) -> PyResult<()> {
        let data = self::chart_data(chart_data)?;
        let mut presentation = self.presentation.borrow_mut(py);
        validate_path(py, &presentation, &self.path, "chart", ".chart")?;
        presentation
            .inner
            .replace_chart_data(slide_index(&self.path)?, &self.shape_path(), &data)
            .map_err(|error| rpptx_to_pyerr(py, error))
    }

    /// Each series with its name and cached values, in plot order.
    #[getter]
    fn series<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let series = self
            .read(py)?
            .1
            .series
            .into_iter()
            .map(|(name, values)| PyChartSeries { name, values })
            .collect::<Vec<_>>();
        PyTuple::new(py, series)
    }
}

/// One chart series as read from the chart part's caches.
#[pyclass(name = "ChartSeries", frozen)]
pub struct PyChartSeries {
    name: String,
    values: Vec<f64>,
}

#[pymethods]
impl PyChartSeries {
    #[getter]
    fn name(&self) -> &str {
        &self.name
    }

    /// The cached values, with `None` for a point the cache leaves out.
    #[getter]
    fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(
            py,
            self.values
                .iter()
                .map(|value| (!value.is_nan()).then_some(*value)),
        )
    }
}
