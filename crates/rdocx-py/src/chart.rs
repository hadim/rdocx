use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyTuple};

/// The python-pptx `XL_CHART_TYPE` value of each chart family `add_chart`
/// authors, the value python-pptx reads back from the same chart in a deck.
const CHART_TYPES: [(i64, rdocx::ChartKind); 7] = [
    (1, rdocx::ChartKind::Area),
    (51, rdocx::ChartKind::Bar),
    (-4120, rdocx::ChartKind::Doughnut),
    (65, rdocx::ChartKind::Line),
    (5, rdocx::ChartKind::Pie),
    (81, rdocx::ChartKind::Radar),
    (74, rdocx::ChartKind::Scatter),
];

/// Reads an `XL_CHART_TYPE` member, or its integer value, that `add_chart`
/// supports.
pub(crate) fn chart_kind(chart_type: &Bound<'_, PyAny>) -> PyResult<rdocx::ChartKind> {
    let value = chart_type.extract::<i64>()?;
    CHART_TYPES
        .iter()
        .find_map(|(candidate, kind)| (*candidate == value).then_some(*kind))
        .ok_or_else(|| PyValueError::new_err(format!("unsupported XL_CHART_TYPE value {value}")))
}

/// Reads a `CategoryChartData` or `XyChartData` into the facade's chart data.
pub(crate) fn chart_data(chart_data: &Bound<'_, PyAny>) -> PyResult<rdocx::ChartData> {
    if !chart_data.hasattr("_data")? {
        return Err(PyTypeError::new_err(
            "chart_data must be a CategoryChartData or an XyChartData",
        ));
    }
    let (categories, series, number_format) =
        chart_data
            .call_method0("_data")?
            .extract::<(Vec<String>, Vec<(String, Vec<f64>)>, Option<String>)>()?;
    Ok(rdocx::ChartData {
        categories,
        series,
        number_format,
        ..rdocx::ChartData::default()
    })
}

/// A snapshot of one chart's type and cached data.
#[pyclass(name = "Chart", frozen)]
pub struct PyChart {
    kind: Option<rdocx::ChartKind>,
    data: rdocx::ChartData,
}

impl PyChart {
    pub(crate) fn new((kind, data): (Option<rdocx::ChartKind>, rdocx::ChartData)) -> Self {
        Self { kind, data }
    }
}

#[pymethods]
impl PyChart {
    /// The `XL_CHART_TYPE` member, or `None` for a chart `add_chart` cannot
    /// author, such as a stacked or 3-D chart.
    #[getter]
    fn chart_type(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let Some(kind) = self.kind else {
            return Ok(None);
        };
        let value = CHART_TYPES
            .iter()
            .find_map(|(value, candidate)| (*candidate == kind).then_some(*value))
            .expect("every chart kind has a chart type");
        py.import("rdocx.enum.chart")?
            .getattr("XL_CHART_TYPE")?
            .call1((value,))
            .map(|member| Some(member.unbind()))
    }

    /// The category labels of the first series, or its x values as text in an
    /// XY chart.
    #[getter]
    fn categories<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.data.categories)
    }

    /// Each series with its name and cached values, in plot order.
    #[getter]
    fn series<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let series = self
            .data
            .series
            .iter()
            .map(|(name, values)| PyChartSeries {
                name: name.clone(),
                values: values.clone(),
            })
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

    #[getter]
    fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.values)
    }
}
