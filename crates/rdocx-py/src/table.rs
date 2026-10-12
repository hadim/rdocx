use oxml_py_support::{ContentPath, PathSeg, revision_to_pyerr};
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PySlice, PyTuple};
use smallvec::smallvec;

use crate::document::PyDocument;
use crate::paragraph::PyParagraph;
use crate::{
    color_hex, color_object, enum_object, length_object, normalize_index, rdocx_to_pyerr,
    stale_to_pyerr,
};

fn path_index(
    path: &ContentPath,
    expected: fn(PathSeg) -> Option<usize>,
    kind: &str,
) -> PyResult<usize> {
    path.segs
        .iter()
        .copied()
        .find_map(expected)
        .ok_or_else(|| PyIndexError::new_err(format!("{kind} index is missing")))
}

#[derive(Clone)]
enum TableLocation {
    Body(usize),
    Related(rdocx::ContentLocation),
}
impl TableLocation {
    fn new(index: usize, related: &Option<rdocx::ContentLocation>) -> Self {
        related.as_ref().map_or(Self::Body(index), |location| {
            Self::Related(location.clone())
        })
    }
    fn refreshed(
        document: &rdocx::Document,
        location: &rdocx::ContentLocation,
    ) -> rdocx::Result<rdocx::ContentLocation> {
        let owner = location.story();
        let story = document
            .stories()?
            .into_iter()
            .find(|story| {
                story.kind() == owner.kind()
                    && story.part_name() == owner.part_name()
                    && story.owner_index() == owner.owner_index()
            })
            .ok_or_else(|| rdocx::Error::Other("rich table owner disappeared".to_owned()))?;
        Ok(rdocx::ContentLocation::new(
            story,
            location.item_kind(),
            location.index_path().to_vec(),
        ))
    }
    fn read<T>(
        &self,
        document: &rdocx::Document,
        read: impl FnOnce(rdocx::TableRef<'_>) -> T,
    ) -> rdocx::Result<T> {
        match self {
            Self::Body(index) => document
                .table(*index)
                .map(read)
                .ok_or_else(|| rdocx::Error::Other("table index out of range".to_owned())),
            Self::Related(location) => {
                document.read_story_table(&Self::refreshed(document, location)?, read)
            }
        }
    }
    fn edit<T>(
        &self,
        document: &mut rdocx::Document,
        edit: impl FnOnce(&mut rdocx::Table<'_>) -> rdocx::Result<T>,
    ) -> rdocx::Result<T> {
        match self {
            Self::Body(index) => edit(
                &mut document
                    .table_mut(*index)
                    .ok_or_else(|| rdocx::Error::Other("table index out of range".to_owned()))?,
            ),
            Self::Related(location) => {
                document.edit_story_table(&Self::refreshed(document, location)?, edit)
            }
        }
    }
}

fn table_index(path: &ContentPath) -> PyResult<usize> {
    path_index(
        path,
        |segment| match segment {
            PathSeg::Body(index) => Some(index),
            _ => None,
        },
        "table",
    )
}

fn row_index(path: &ContentPath) -> PyResult<usize> {
    path_index(
        path,
        |segment| match segment {
            PathSeg::Row(index) => Some(index),
            _ => None,
        },
        "row",
    )
}

fn cell_index(path: &ContentPath) -> PyResult<usize> {
    path_index(
        path,
        |segment| match segment {
            PathSeg::Cell(index) => Some(index),
            _ => None,
        },
        "cell",
    )
}

fn alignment_from_int(value: i32) -> PyResult<rdocx::Alignment> {
    match value {
        0 => Ok(rdocx::Alignment::Left),
        1 => Ok(rdocx::Alignment::Center),
        2 => Ok(rdocx::Alignment::Right),
        _ => Err(PyValueError::new_err("unsupported table alignment")),
    }
}

fn alignment_to_int(value: rdocx::Alignment) -> Option<i32> {
    match value {
        rdocx::Alignment::Left => Some(0),
        rdocx::Alignment::Center => Some(1),
        rdocx::Alignment::Right => Some(2),
        rdocx::Alignment::Justify => None,
    }
}

fn vertical_from_int(value: i32) -> PyResult<rdocx::VerticalAlignment> {
    match value {
        0 => Ok(rdocx::VerticalAlignment::Top),
        1 => Ok(rdocx::VerticalAlignment::Center),
        3 => Ok(rdocx::VerticalAlignment::Bottom),
        _ => Err(PyValueError::new_err("unsupported cell vertical alignment")),
    }
}

fn vertical_to_int(value: rdocx::VerticalAlignment) -> i32 {
    match value {
        rdocx::VerticalAlignment::Top => 0,
        rdocx::VerticalAlignment::Center => 1,
        rdocx::VerticalAlignment::Bottom => 3,
    }
}

pub(crate) fn border_style_from_name(value: &str) -> PyResult<rdocx::BorderStyle> {
    match value {
        "none" => Ok(rdocx::BorderStyle::None),
        "single" => Ok(rdocx::BorderStyle::Single),
        "thick" => Ok(rdocx::BorderStyle::Thick),
        "double" => Ok(rdocx::BorderStyle::Double),
        "dotted" => Ok(rdocx::BorderStyle::Dotted),
        "dashed" => Ok(rdocx::BorderStyle::Dashed),
        "dotDash" => Ok(rdocx::BorderStyle::DotDash),
        "wave" => Ok(rdocx::BorderStyle::Wave),
        _ => Err(PyValueError::new_err(
            "border style must be none, single, thick, double, dotted, dashed, dotDash or wave",
        )),
    }
}

fn table_border_edge(value: &str) -> PyResult<rdocx::TableBorderEdge> {
    match value {
        "top" => Ok(rdocx::TableBorderEdge::Top),
        "bottom" => Ok(rdocx::TableBorderEdge::Bottom),
        "left" => Ok(rdocx::TableBorderEdge::Left),
        "right" => Ok(rdocx::TableBorderEdge::Right),
        "insideH" => Ok(rdocx::TableBorderEdge::InsideHorizontal),
        "insideV" => Ok(rdocx::TableBorderEdge::InsideVertical),
        _ => Err(PyValueError::new_err(
            "border edge must be top, bottom, left, right, insideH or insideV",
        )),
    }
}

fn cell_border_edge(value: &str) -> PyResult<rdocx::CellBorderEdge> {
    match value {
        "top" => Ok(rdocx::CellBorderEdge::Top),
        "bottom" => Ok(rdocx::CellBorderEdge::Bottom),
        "left" => Ok(rdocx::CellBorderEdge::Left),
        "right" => Ok(rdocx::CellBorderEdge::Right),
        "insideH" => Ok(rdocx::CellBorderEdge::InsideHorizontal),
        "insideV" => Ok(rdocx::CellBorderEdge::InsideVertical),
        _ => Err(PyValueError::new_err(
            "border edge must be top, bottom, left, right, insideH or insideV",
        )),
    }
}

/// A border edge as `(style, size in eighths of a point, color)`.
type BorderSnapshot = (String, Option<u32>, Option<Py<PyAny>>);

fn table_color_hex(value: &Bound<'_, PyAny>, parameter: &str) -> PyResult<String> {
    if value
        .extract::<String>()
        .is_ok_and(|value| value.eq_ignore_ascii_case("auto"))
    {
        Ok("auto".to_owned())
    } else {
        color_hex(value, parameter)
    }
}

fn border_snapshot(py: Python<'_>, border: rdocx::TableBorderRef<'_>) -> PyResult<BorderSnapshot> {
    Ok((
        border.style().to_owned(),
        border.size_eighths_pt(),
        border
            .color()
            .map(|value| color_object(py, value))
            .transpose()?,
    ))
}

/// Cell margins as `(top, right, bottom, left)`, each a `Length` or `None`.
type MarginSnapshot = (
    Option<Py<PyAny>>,
    Option<Py<PyAny>>,
    Option<Py<PyAny>>,
    Option<Py<PyAny>>,
);

fn margin_snapshot(py: Python<'_>, margins: rdocx::TableCellMargins) -> PyResult<MarginSnapshot> {
    let length =
        |value: Option<rdocx::Length>| value.map(|value| length_object(py, value)).transpose();
    Ok((
        length(margins.top)?,
        length(margins.right)?,
        length(margins.bottom)?,
        length(margins.left)?,
    ))
}

/// Read a `WD_ROW_HEIGHT_RULE` value, where `AT_LEAST` is 1 and `EXACTLY` is 2.
fn row_height_rule_is_exact(value: i32) -> PyResult<bool> {
    match value {
        1 => Ok(false),
        2 => Ok(true),
        _ => Err(PyValueError::new_err("unsupported row height rule")),
    }
}

fn row_height(length: rdocx::Length, exact: bool) -> rdocx::RowHeight {
    if exact {
        rdocx::RowHeight::Exact(length)
    } else {
        rdocx::RowHeight::AtLeast(length)
    }
}

fn row_height_parts(value: rdocx::RowHeight) -> (rdocx::Length, bool) {
    match value {
        rdocx::RowHeight::AtLeast(length) => (length, false),
        rdocx::RowHeight::Exact(length) => (length, true),
    }
}

fn vertical_merge_from_name(value: Option<&str>) -> PyResult<Option<rdocx::VMerge>> {
    match value {
        None => Ok(None),
        Some("restart") => Ok(Some(rdocx::VMerge::Restart)),
        Some("continue") => Ok(Some(rdocx::VMerge::Continue)),
        Some(_) => Err(PyValueError::new_err(
            "vertical merge must be restart, continue or None",
        )),
    }
}

fn vertical_merge_name(value: rdocx::VMerge) -> &'static str {
    match value {
        rdocx::VMerge::Restart => "restart",
        rdocx::VMerge::Continue => "continue",
    }
}

#[pyclass(name = "TableCollection")]
pub struct PyTableCollection {
    document: Py<PyDocument>,
}

impl PyTableCollection {
    pub(crate) fn new(document: Py<PyDocument>) -> Self {
        Self { document }
    }

    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyTable>> {
        let path = self
            .document
            .borrow(py)
            .revisions
            .capture(smallvec![PathSeg::Body(index)]);
        Py::new(py, PyTable::new(self.document.clone_ref(py), path))
    }
}

#[pymethods]
impl PyTableCollection {
    fn __len__(&self, py: Python<'_>) -> usize {
        self.document.borrow(py).inner.table_count()
    }

    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let len = self.__len__(py);
        if let Ok(index) = key.extract::<isize>() {
            return Ok(self
                .item(py, normalize_index(index, len, "table")?)?
                .into_any());
        }
        if key.is_instance_of::<PySlice>() {
            let (start, stop, step): (isize, isize, isize) =
                key.call_method1("indices", (len,))?.extract()?;
            let items = PyList::empty(py);
            let mut index = start;
            while if step > 0 { index < stop } else { index > stop } {
                items.append(self.item(py, index as usize)?)?;
                index += step;
            }
            return Ok(items.into_any().unbind());
        }
        Err(PyTypeError::new_err(
            "table indices must be integers or slices",
        ))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyTableIterator>> {
        Py::new(
            py,
            PyTableIterator {
                document: self.document.clone_ref(py),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyTableIterator {
    document: Py<PyDocument>,
    index: usize,
}

#[pymethods]
impl PyTableIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyTable>>> {
        let collection = PyTableCollection::new(self.document.clone_ref(py));
        if self.index >= collection.__len__(py) {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Table")]
pub struct PyTable {
    document: Py<PyDocument>,
    path: ContentPath,
    related: Option<rdocx::ContentLocation>,
}

impl PyTable {
    pub(crate) fn new(document: Py<PyDocument>, path: ContentPath) -> Self {
        Self {
            document,
            path,
            related: None,
        }
    }

    pub(crate) fn for_story(
        document: Py<PyDocument>,
        path: ContentPath,
        location: rdocx::ContentLocation,
    ) -> Self {
        Self {
            document,
            path,
            related: Some(location),
        }
    }
    fn read<T>(&self, py: Python<'_>, read: impl FnOnce(rdocx::TableRef<'_>) -> T) -> PyResult<T> {
        let index = self.validate(py)?;
        TableLocation::new(index, &self.related)
            .read(&self.document.borrow(py).inner, read)
            .map_err(|error| rdocx_to_pyerr(py, error))
    }
    pub(crate) fn direct_body_index(&self, py: Python<'_>) -> PyResult<usize> {
        let index = self.validate(py)?;
        let document = self.document.borrow(py);
        if let Some(location) = &self.related {
            let location = TableLocation::refreshed(&document.inner, location)
                .map_err(|error| rdocx_to_pyerr(py, error))?;
            return document
                .inner
                .story_item_snapshots()
                .map_err(|error| rdocx_to_pyerr(py, error))?
                .into_iter()
                .find(|item| {
                    item.location() == &location
                        && item.is_direct_child()
                        && location.story().kind() == rdocx::StoryKind::Body
                })
                .and_then(|item| item.direct_body_index())
                .ok_or_else(|| PyValueError::new_err("content handle is not a direct body child"));
        }
        document
            .inner
            .content_index_of_table(index)
            .ok_or_else(|| PyValueError::new_err("content handle is not a direct body child"))
    }
    pub(crate) fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        let document = self.document.borrow(py);
        self.path
            .validate_scopes(
                &document.revisions,
                "table",
                "Re-fetch it with doc.tables[i].",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        table_index(&self.path)
    }

    pub(crate) fn belongs_to(&self, py: Python<'_>, document: &Py<PyDocument>) -> bool {
        self.document.bind(py).is(document.bind(py))
    }

    /// Apply one checked native table edit without advancing the revision.
    fn edit<T>(
        &self,
        py: Python<'_>,
        edit: impl FnOnce(&mut rdocx::Table<'_>) -> rdocx::Result<T>,
    ) -> PyResult<T> {
        let index = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        let result = TableLocation::new(index, &self.related)
            .edit(&mut candidate, edit)
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        document.inner = candidate;
        Ok(result)
    }

    /// Resolve possibly negative row and cell indexes against this table.
    fn cell_coordinates(&self, py: Python<'_>, row: isize, col: isize) -> PyResult<(usize, usize)> {
        self.read(py, |table| {
            let row = normalize_index(row, table.row_count(), "row")?;
            let col = normalize_index(
                col,
                table.row(row).map(|row| row.cell_count()).unwrap_or(0),
                "cell",
            )?;
            Ok((row, col))
        })?
    }
}

#[pymethods]
impl PyTable {
    #[getter]
    fn rows(&self, py: Python<'_>) -> PyResult<Py<PyRowCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyRowCollection::new(
                self.document.clone_ref(py),
                self.path.clone(),
                self.related.clone(),
            ),
        )
    }

    fn cell(&self, py: Python<'_>, row: isize, col: isize) -> PyResult<Py<PyCell>> {
        let (row, col) = self.cell_coordinates(py, row, col)?;
        let table_index = table_index(&self.path)?;
        let path = self.document.borrow(py).revisions.capture(smallvec![
            PathSeg::Body(table_index),
            PathSeg::Row(row),
            PathSeg::Cell(col)
        ]);
        Py::new(
            py,
            PyCell::new(self.document.clone_ref(py), path, self.related.clone()),
        )
    }

    #[getter]
    fn style(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.read(py, |table| table.style_id().map(str::to_owned))
    }

    #[setter]
    fn set_style(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        self.validate(py)?;
        let style_id = {
            let document = self.document.borrow(py);
            let styles = document.inner.styles();
            let exact_id = styles.iter().any(|style| {
                style.style_id() == value && style.style_type() == rdocx::StyleType::Table
            });
            if !exact_id {
                let matching = styles
                    .iter()
                    .filter(|style| {
                        style.style_type() == rdocx::StyleType::Table
                            && style
                                .name()
                                .is_some_and(|name| name.eq_ignore_ascii_case(value))
                    })
                    .count();
                if matching > 1 {
                    return Err(PyValueError::new_err("table style name is ambiguous"));
                }
            }
            crate::paragraph::style_id_of_type(&document.inner, value, rdocx::StyleType::Table)?
        };
        self.edit(py, |table| {
            table.set_style(&style_id);
            Ok(())
        })
    }

    #[getter]
    fn autofit(&self, py: Python<'_>) -> PyResult<bool> {
        self.read(py, |table| {
            table.layout() != Some(rdocx::TableLayout::Fixed)
        })
    }

    #[setter]
    fn set_autofit(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        self.edit(py, |table| {
            table.set_layout(if value {
                rdocx::TableLayout::AutoFit
            } else {
                rdocx::TableLayout::Fixed
            });
            Ok(())
        })
    }

    #[getter]
    fn first_row(&self, py: Python<'_>) -> PyResult<bool> {
        self.read(py, |table| {
            table
                .look()
                .unwrap_or(rdocx::TableLook {
                    horizontal_banding: true,
                    vertical_banding: true,
                    ..rdocx::TableLook::default()
                })
                .first_row
        })
    }

    #[setter]
    fn set_first_row(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let mut look = self.read(py, |table| {
            table.look().unwrap_or(rdocx::TableLook {
                horizontal_banding: true,
                vertical_banding: true,
                ..rdocx::TableLook::default()
            })
        })?;
        look.first_row = value;
        self.edit(py, |table| {
            table.set_look(look);
            Ok(())
        })
    }

    #[getter]
    fn last_row(&self, py: Python<'_>) -> PyResult<bool> {
        self.read(py, |table| {
            table
                .look()
                .unwrap_or(rdocx::TableLook {
                    horizontal_banding: true,
                    vertical_banding: true,
                    ..rdocx::TableLook::default()
                })
                .last_row
        })
    }

    #[setter]
    fn set_last_row(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let mut look = self.read(py, |table| {
            table.look().unwrap_or(rdocx::TableLook {
                horizontal_banding: true,
                vertical_banding: true,
                ..rdocx::TableLook::default()
            })
        })?;
        look.last_row = value;
        self.edit(py, |table| {
            table.set_look(look);
            Ok(())
        })
    }

    #[getter]
    fn first_column(&self, py: Python<'_>) -> PyResult<bool> {
        self.read(py, |table| {
            table
                .look()
                .unwrap_or(rdocx::TableLook {
                    horizontal_banding: true,
                    vertical_banding: true,
                    ..rdocx::TableLook::default()
                })
                .first_column
        })
    }

    #[setter]
    fn set_first_column(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let mut look = self.read(py, |table| {
            table.look().unwrap_or(rdocx::TableLook {
                horizontal_banding: true,
                vertical_banding: true,
                ..rdocx::TableLook::default()
            })
        })?;
        look.first_column = value;
        self.edit(py, |table| {
            table.set_look(look);
            Ok(())
        })
    }

    #[getter]
    fn last_column(&self, py: Python<'_>) -> PyResult<bool> {
        self.read(py, |table| {
            table
                .look()
                .unwrap_or(rdocx::TableLook {
                    horizontal_banding: true,
                    vertical_banding: true,
                    ..rdocx::TableLook::default()
                })
                .last_column
        })
    }

    #[setter]
    fn set_last_column(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let mut look = self.read(py, |table| {
            table.look().unwrap_or(rdocx::TableLook {
                horizontal_banding: true,
                vertical_banding: true,
                ..rdocx::TableLook::default()
            })
        })?;
        look.last_column = value;
        self.edit(py, |table| {
            table.set_look(look);
            Ok(())
        })
    }

    #[getter]
    fn banded_rows(&self, py: Python<'_>) -> PyResult<bool> {
        self.read(py, |table| {
            table
                .look()
                .unwrap_or(rdocx::TableLook {
                    horizontal_banding: true,
                    vertical_banding: true,
                    ..rdocx::TableLook::default()
                })
                .horizontal_banding
        })
    }

    #[setter]
    fn set_banded_rows(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let mut look = self.read(py, |table| {
            table.look().unwrap_or(rdocx::TableLook {
                horizontal_banding: true,
                vertical_banding: true,
                ..rdocx::TableLook::default()
            })
        })?;
        look.horizontal_banding = value;
        self.edit(py, |table| {
            table.set_look(look);
            Ok(())
        })
    }

    #[getter]
    fn banded_columns(&self, py: Python<'_>) -> PyResult<bool> {
        self.read(py, |table| {
            table
                .look()
                .unwrap_or(rdocx::TableLook {
                    horizontal_banding: true,
                    vertical_banding: true,
                    ..rdocx::TableLook::default()
                })
                .vertical_banding
        })
    }

    #[setter]
    fn set_banded_columns(&self, py: Python<'_>, value: bool) -> PyResult<()> {
        let mut look = self.read(py, |table| {
            table.look().unwrap_or(rdocx::TableLook {
                horizontal_banding: true,
                vertical_banding: true,
                ..rdocx::TableLook::default()
            })
        })?;
        look.vertical_banding = value;
        self.edit(py, |table| {
            table.set_look(look);
            Ok(())
        })
    }

    #[getter]
    fn indent(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |table| table.indent())?
            .map(|value| length_object(py, value))
            .transpose()
    }

    #[setter]
    fn set_indent(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()> {
        self.edit(py, |table| {
            table.set_indent_value(value.map(rdocx::Length::emu))
        })
    }

    #[getter]
    fn alignment(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |table| table.alignment())?
            .and_then(alignment_to_int)
            .map(|value| enum_object(py, "WD_TABLE_ALIGNMENT", value))
            .transpose()
    }

    #[setter]
    fn set_alignment(&self, py: Python<'_>, value: i32) -> PyResult<()> {
        let value = alignment_from_int(value)?;
        self.edit(py, |table| {
            table.set_alignment(value);
            Ok(())
        })
    }

    #[getter]
    fn width(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |table| table.width())?
            .map(|value| length_object(py, value))
            .transpose()
    }

    #[setter]
    fn set_width(&self, py: Python<'_>, value: i64) -> PyResult<()> {
        self.edit(py, |table| {
            table.set_width(rdocx::Length::emu(value));
            Ok(())
        })
    }

    #[pyo3(signature = (style, *, size, color))]
    fn set_borders(
        &self,
        py: Python<'_>,
        style: &str,
        size: u32,
        color: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let color = table_color_hex(color, "Table.set_borders(color)")?;
        let style = border_style_from_name(style)?;
        self.edit(py, |table| {
            table.set_all_borders_checked(style, size, &color)
        })
    }

    #[pyo3(signature = (edge, style, *, size, color))]
    fn set_border(
        &self,
        py: Python<'_>,
        edge: &str,
        style: &str,
        size: u32,
        color: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let color = table_color_hex(color, "Table.set_border(color)")?;
        let edge = table_border_edge(edge)?;
        let style = border_style_from_name(style)?;
        self.edit(py, |table| {
            table.set_border_checked(edge, style, size, &color)
        })
    }

    fn border(&self, py: Python<'_>, edge: &str) -> PyResult<Option<BorderSnapshot>> {
        let edge = table_border_edge(edge)?;
        self.read(py, |table| {
            table.border(edge).map(|border| border_snapshot(py, border))
        })?
        .transpose()
    }

    #[getter]
    fn cell_margins(&self, py: Python<'_>) -> PyResult<Option<MarginSnapshot>> {
        self.read(py, |table| table.cell_margins())?
            .map(|margins| margin_snapshot(py, margins))
            .transpose()
    }

    #[pyo3(signature = (*, top, right, bottom, left))]
    fn set_cell_margins(
        &self,
        py: Python<'_>,
        top: i64,
        right: i64,
        bottom: i64,
        left: i64,
    ) -> PyResult<()> {
        self.edit(py, |table| {
            table.set_cell_margins_checked(
                rdocx::Length::emu(top),
                rdocx::Length::emu(right),
                rdocx::Length::emu(bottom),
                rdocx::Length::emu(left),
            )
        })
    }

    #[getter]
    fn grid_widths<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let widths = self
            .read(py, |table| table.grid_widths())?
            .into_iter()
            .map(|width| length_object(py, width))
            .collect::<PyResult<Vec<_>>>()?;
        PyTuple::new(py, widths)
    }

    #[setter]
    fn set_grid_widths(&self, py: Python<'_>, value: Vec<i64>) -> PyResult<()> {
        let widths = value
            .into_iter()
            .map(rdocx::Length::emu)
            .collect::<Vec<_>>();
        self.edit(py, |table| table.set_grid_widths(&widths))
    }

    fn set_column_width(&self, py: Python<'_>, column: isize, width: i64) -> PyResult<()> {
        let columns = self.read(py, |table| table.grid_widths().len())?;
        let column = normalize_index(column, columns, "column")?;
        let applied = self.edit(py, |table| {
            Ok(table.set_column_width(column, rdocx::Length::emu(width)))
        })?;
        if !applied {
            return Err(PyValueError::new_err(
                "column width must be nonnegative and every row must fit the table grid",
            ));
        }
        Ok(())
    }

    #[pyo3(signature = (row, col, span))]
    fn set_cell_grid_span(
        &self,
        py: Python<'_>,
        row: isize,
        col: isize,
        span: Option<u32>,
    ) -> PyResult<()> {
        let (row, col) = self.cell_coordinates(py, row, col)?;
        let table_index = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        let changed = {
            TableLocation::new(table_index, &self.related)
                .edit(&mut candidate, |table| {
                    let before = table.row(row).map(|row| row.cell_count());
                    table.set_cell_grid_span_checked(row, col, span)?;
                    Ok(table.row(row).map(|row| row.cell_count()) != before)
                })
                .map_err(|error| rdocx_to_pyerr(py, error))?
        };
        if changed {
            let prepared = document
                .revisions
                .prepare_scopes("Table.set_cell_grid_span", &[], &[])
                .map_err(oxml_py_support::revision_to_pyerr)?;
            document
                .revisions
                .commit_scopes(prepared)
                .map_err(oxml_py_support::revision_to_pyerr)?;
        }
        document.inner = candidate;
        Ok(())
    }

    #[pyo3(signature = (row, col, merge))]
    fn set_cell_vertical_merge(
        &self,
        py: Python<'_>,
        row: isize,
        col: isize,
        merge: Option<&str>,
    ) -> PyResult<()> {
        let merge = vertical_merge_from_name(merge)?;
        let (row, col) = self.cell_coordinates(py, row, col)?;
        self.edit(py, |table| table.set_cell_vertical_merge(row, col, merge))
    }

    fn add_row(&self, py: Python<'_>) -> PyResult<Py<PyRow>> {
        let index = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        let inserted = match &self.related {
            None => candidate.add_empty_table_row(index),
            Some(location) => {
                let location = TableLocation::refreshed(&candidate, location)
                    .map_err(|error| rdocx_to_pyerr(py, error))?;
                candidate.edit_story_table(&location, |table| {
                    let index = table.row_count();
                    table.add_row()?;
                    Ok(index)
                })
            }
        }
        .map_err(|error| rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Table.add_row", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        let mut segments = self.path.segs.clone();
        segments.push(PathSeg::Row(inserted));
        let path = document
            .revisions
            .capture_prepared(&prepared, segments)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        let result = Py::new(
            py,
            PyRow::new(self.document.clone_ref(py), path, self.related.clone()),
        )?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(result)
    }

    #[pyo3(signature = (index, *, before = true))]
    fn insert_column(&self, py: Python<'_>, index: usize, before: bool) -> PyResult<()> {
        let table_index = self.validate(py)?;
        if index >= self.read(py, |table| table.column_count())? {
            return Err(PyIndexError::new_err(
                "column insertion index is out of range",
            ));
        }
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        match &self.related {
            None => candidate.insert_table_column(table_index, index, before),
            Some(location) => {
                let location = TableLocation::refreshed(&candidate, location)
                    .map_err(|error| rdocx_to_pyerr(py, error))?;
                candidate
                    .edit_story_table(&location, |table| table.insert_column(index, before, None))
            }
        }
        .map_err(|error| rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Table.insert_column", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(())
    }

    fn delete_column(&self, py: Python<'_>, index: usize) -> PyResult<()> {
        let table_index = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        match &self.related {
            None => candidate.delete_table_column(table_index, index),
            Some(location) => candidate.delete_story_table_column(
                &TableLocation::refreshed(&candidate, location)
                    .map_err(|error| rdocx_to_pyerr(py, error))?,
                index,
            ),
        }
        .map_err(|error| rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Table.delete_column", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(())
    }

    #[pyo3(signature = (index, at = None))]
    fn clone_row(&self, py: Python<'_>, index: isize, at: Option<usize>) -> PyResult<Py<PyRow>> {
        let table_index = self.validate(py)?;
        let result = {
            let mut document = self.document.borrow_mut(py);
            let mut candidate = document.inner.clone_for_staging();
            let row_count = TableLocation::new(table_index, &self.related)
                .read(&candidate, |table| table.row_count())
                .map_err(|error| rdocx_to_pyerr(py, error))?;
            let source = normalize_index(index, row_count, "row")?;
            let insert_at = at.unwrap_or(source + 1);
            if insert_at > row_count {
                return Err(PyIndexError::new_err("row insertion index out of range"));
            }
            let inserted = match &self.related {
                Some(location) => {
                    let location = TableLocation::refreshed(&candidate, location)
                        .map_err(|error| rdocx_to_pyerr(py, error))?;
                    candidate.clone_story_table_row(&location, source, insert_at)
                }
                None => candidate.clone_table_row(table_index, source, insert_at),
            }
            .map_err(|error| rdocx_to_pyerr(py, error))?;
            let prepared = document
                .revisions
                .prepare_scopes("Table.clone_row", &[], &[])
                .map_err(oxml_py_support::revision_to_pyerr)?;
            let mut segments = self.path.segs.clone();
            segments.push(PathSeg::Row(inserted));
            let path = document
                .revisions
                .capture_prepared(&prepared, segments)
                .map_err(oxml_py_support::revision_to_pyerr)?;
            let result = Py::new(
                py,
                PyRow::new(self.document.clone_ref(py), path, self.related.clone()),
            )?;
            document
                .revisions
                .commit_scopes(prepared)
                .map_err(oxml_py_support::revision_to_pyerr)?;
            document.inner = candidate;
            result
        };
        Ok(result)
    }

    fn remove_row(&self, py: Python<'_>, index: isize) -> PyResult<()> {
        let table_index = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        let row_count = TableLocation::new(table_index, &self.related)
            .read(&candidate, |table| table.row_count())
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        let row_index = normalize_index(index, row_count, "row")?;
        match &self.related {
            Some(location) => {
                let location = TableLocation::refreshed(&candidate, location)
                    .map_err(|error| rdocx_to_pyerr(py, error))?;
                candidate.remove_story_table_row(&location, row_index)
            }
            None => candidate.remove_table_row(table_index, row_index),
        }
        .map_err(|error| rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Table.remove_row", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(())
    }
}

#[pyclass(name = "RowCollection")]
pub struct PyRowCollection {
    document: Py<PyDocument>,
    table_path: ContentPath,
    related: Option<rdocx::ContentLocation>,
}

impl PyRowCollection {
    fn new(
        document: Py<PyDocument>,
        table_path: ContentPath,
        related: Option<rdocx::ContentLocation>,
    ) -> Self {
        Self {
            document,
            table_path,
            related,
        }
    }
    fn validate(&self, py: Python<'_>) -> PyResult<usize> {
        let document = self.document.borrow(py);
        self.table_path
            .validate_scopes(
                &document.revisions,
                "row collection",
                "Re-fetch it with table.rows.",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        table_index(&self.table_path)
    }
    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let index = self.validate(py)?;
        TableLocation::new(index, &self.related)
            .read(&self.document.borrow(py).inner, |table| table.row_count())
            .map_err(|error| rdocx_to_pyerr(py, error))
    }
    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyRow>> {
        self.validate(py)?;
        let mut segments = self.table_path.segs.clone();
        segments.push(PathSeg::Row(index));
        let path = self.document.borrow(py).revisions.capture(segments);
        Py::new(
            py,
            PyRow::new(self.document.clone_ref(py), path, self.related.clone()),
        )
    }
}

#[pymethods]
impl PyRowCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }
    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let len = self.len(py)?;
        if let Ok(index) = key.extract::<isize>() {
            return Ok(self
                .item(py, normalize_index(index, len, "row")?)?
                .into_any());
        }
        if key.is_instance_of::<PySlice>() {
            let (start, stop, step): (isize, isize, isize) =
                key.call_method1("indices", (len,))?.extract()?;
            let items = PyList::empty(py);
            let mut index = start;
            while if step > 0 { index < stop } else { index > stop } {
                items.append(self.item(py, index as usize)?)?;
                index += step;
            }
            return Ok(items.into_any().unbind());
        }
        Err(PyTypeError::new_err(
            "row indices must be integers or slices",
        ))
    }
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyRowIterator>> {
        self.validate(py)?;
        Py::new(
            py,
            PyRowIterator {
                document: self.document.clone_ref(py),
                table_path: self.table_path.clone(),
                related: self.related.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyRowIterator {
    document: Py<PyDocument>,
    table_path: ContentPath,
    index: usize,
    related: Option<rdocx::ContentLocation>,
}

#[pymethods]
impl PyRowIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyRow>>> {
        let collection = PyRowCollection::new(
            self.document.clone_ref(py),
            self.table_path.clone(),
            self.related.clone(),
        );
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Row")]
pub struct PyRow {
    document: Py<PyDocument>,
    path: ContentPath,
    related: Option<rdocx::ContentLocation>,
}

impl PyRow {
    fn new(
        document: Py<PyDocument>,
        path: ContentPath,
        related: Option<rdocx::ContentLocation>,
    ) -> Self {
        Self {
            document,
            path,
            related,
        }
    }
    fn validate(&self, py: Python<'_>) -> PyResult<(usize, usize)> {
        let document = self.document.borrow(py);
        self.path
            .validate_scopes(
                &document.revisions,
                "row",
                "Re-fetch it with table.rows[i].",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        Ok((table_index(&self.path)?, row_index(&self.path)?))
    }

    fn read<T>(&self, py: Python<'_>, read: impl FnOnce(rdocx::RowRef<'_>) -> T) -> PyResult<T> {
        let (table, row) = self.validate(py)?;
        TableLocation::new(table, &self.related)
            .read(&self.document.borrow(py).inner, |table| {
                let row = table
                    .row(row)
                    .ok_or_else(|| PyIndexError::new_err("row index out of range"))?;
                Ok(read(row))
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?
    }

    /// Apply one checked native row edit that moves no content, so live
    /// handles stay valid.
    fn edit(
        &self,
        py: Python<'_>,
        edit: impl FnOnce(&mut rdocx::Row<'_>) -> rdocx::Result<()>,
    ) -> PyResult<()> {
        let (table, row) = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        TableLocation::new(table, &self.related)
            .edit(&mut candidate, |table| {
                let mut row = table
                    .row(row)
                    .ok_or_else(|| rdocx::Error::Other("row index out of range".to_owned()))?;
                edit(&mut row)
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        document.inner = candidate;
        Ok(())
    }
}

#[pymethods]
impl PyRow {
    #[getter]
    fn cells(&self, py: Python<'_>) -> PyResult<Py<PyCellCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyCellCollection::new(
                self.document.clone_ref(py),
                self.path.clone(),
                self.related.clone(),
            ),
        )
    }

    #[getter]
    fn height(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |row| row.height())?
            .map(|height| length_object(py, row_height_parts(height).0))
            .transpose()
    }

    // An exact height keeps its rule. Any other row gets a minimum height,
    // including one whose w:trHeight has an auto rule or no value, which the
    // native reader reports as no height.
    #[setter]
    fn set_height(&self, py: Python<'_>, value: i64) -> PyResult<()> {
        let exact = self
            .read(py, |row| row.height())?
            .is_some_and(|height| row_height_parts(height).1);
        let height = row_height(rdocx::Length::emu(value), exact);
        self.edit(py, |row| row.set_height_checked(height))
    }

    #[getter]
    fn height_rule(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |row| row.height())?
            .map(|height| {
                let exact = row_height_parts(height).1;
                enum_object(py, "WD_ROW_HEIGHT_RULE", if exact { 2 } else { 1 })
            })
            .transpose()
    }

    #[setter]
    fn set_height_rule(&self, py: Python<'_>, value: i32) -> PyResult<()> {
        let exact = row_height_rule_is_exact(value)?;
        let length = self
            .read(py, |row| row.height())?
            .map(|height| row_height_parts(height).0)
            .ok_or_else(|| {
                PyValueError::new_err(
                    "row has no height for the rule to apply to, set height first",
                )
            })?;
        self.edit(py, |row| row.set_height_checked(row_height(length, exact)))
    }

    #[getter]
    fn cant_split(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read(py, |row| row.cant_split_value())
    }

    #[setter]
    fn set_cant_split(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.edit(py, |row| {
            row.set_cant_split_value(value);
            Ok(())
        })
    }

    #[getter]
    fn is_header(&self, py: Python<'_>) -> PyResult<Option<bool>> {
        self.read(py, |row| row.header_value())
    }

    #[setter]
    fn set_is_header(&self, py: Python<'_>, value: Option<bool>) -> PyResult<()> {
        self.edit(py, |row| {
            row.set_header_value(value);
            Ok(())
        })
    }
}

#[pyclass(name = "CellCollection")]
pub struct PyCellCollection {
    document: Py<PyDocument>,
    row_path: ContentPath,
    related: Option<rdocx::ContentLocation>,
}

impl PyCellCollection {
    fn new(
        document: Py<PyDocument>,
        row_path: ContentPath,
        related: Option<rdocx::ContentLocation>,
    ) -> Self {
        Self {
            document,
            row_path,
            related,
        }
    }
    fn validate(&self, py: Python<'_>) -> PyResult<(usize, usize)> {
        let document = self.document.borrow(py);
        self.row_path
            .validate_scopes(
                &document.revisions,
                "cell collection",
                "Re-fetch it with row.cells.",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        Ok((table_index(&self.row_path)?, row_index(&self.row_path)?))
    }
    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let (table, row) = self.validate(py)?;
        TableLocation::new(table, &self.related)
            .read(&self.document.borrow(py).inner, |table| {
                table.row(row).map(|row| row.cell_count())
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?
            .ok_or_else(|| PyIndexError::new_err("row index out of range"))
    }
    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyCell>> {
        self.validate(py)?;
        let mut segments = self.row_path.segs.clone();
        segments.push(PathSeg::Cell(index));
        let path = self.document.borrow(py).revisions.capture(segments);
        Py::new(
            py,
            PyCell::new(self.document.clone_ref(py), path, self.related.clone()),
        )
    }
}

#[pymethods]
impl PyCellCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }
    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let len = self.len(py)?;
        if let Ok(index) = key.extract::<isize>() {
            return Ok(self
                .item(py, normalize_index(index, len, "cell")?)?
                .into_any());
        }
        if key.is_instance_of::<PySlice>() {
            let (start, stop, step): (isize, isize, isize) =
                key.call_method1("indices", (len,))?.extract()?;
            let items = PyList::empty(py);
            let mut index = start;
            while if step > 0 { index < stop } else { index > stop } {
                items.append(self.item(py, index as usize)?)?;
                index += step;
            }
            return Ok(items.into_any().unbind());
        }
        Err(PyTypeError::new_err(
            "cell indices must be integers or slices",
        ))
    }
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyCellIterator>> {
        self.validate(py)?;
        Py::new(
            py,
            PyCellIterator {
                document: self.document.clone_ref(py),
                row_path: self.row_path.clone(),
                related: self.related.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyCellIterator {
    document: Py<PyDocument>,
    row_path: ContentPath,
    index: usize,
    related: Option<rdocx::ContentLocation>,
}

#[pymethods]
impl PyCellIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyCell>>> {
        let collection = PyCellCollection::new(
            self.document.clone_ref(py),
            self.row_path.clone(),
            self.related.clone(),
        );
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}

#[pyclass(name = "Cell")]
pub struct PyCell {
    document: Py<PyDocument>,
    path: ContentPath,
    related: Option<rdocx::ContentLocation>,
}

impl PyCell {
    fn physical_story(
        &self,
        py: Python<'_>,
        document: &rdocx::Document,
        table: usize,
        row: usize,
        cell: usize,
    ) -> PyResult<rdocx::StoryId> {
        match &self.related {
            Some(location) => {
                let location = TableLocation::refreshed(document, location)
                    .map_err(|error| rdocx_to_pyerr(py, error))?;
                document
                    .story_table_cell(&location, row, cell)
                    .map_err(|error| rdocx_to_pyerr(py, error))
            }
            None => document
                .cell_paragraph_story_location(table, row, cell, 0)
                .map(|location| location.story().clone())
                .map_err(|error| rdocx_to_pyerr(py, error)),
        }
    }

    fn new(
        document: Py<PyDocument>,
        path: ContentPath,
        related: Option<rdocx::ContentLocation>,
    ) -> Self {
        Self {
            document,
            path,
            related,
        }
    }
    fn validate(&self, py: Python<'_>) -> PyResult<(usize, usize, usize)> {
        let document = self.document.borrow(py);
        self.path
            .validate_scopes(
                &document.revisions,
                "cell",
                "Re-fetch it with row.cells[i].",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        Ok((
            table_index(&self.path)?,
            row_index(&self.path)?,
            cell_index(&self.path)?,
        ))
    }

    fn read<T>(&self, py: Python<'_>, read: impl FnOnce(rdocx::CellRef<'_>) -> T) -> PyResult<T> {
        let (table, row, cell) = self.validate(py)?;
        TableLocation::new(table, &self.related)
            .read(&self.document.borrow(py).inner, |table| {
                let cell = table
                    .cell(row, cell)
                    .ok_or_else(|| PyIndexError::new_err("cell index out of range"))?;
                Ok(read(cell))
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?
    }

    /// Apply one checked native cell edit that moves no content, so live
    /// handles stay valid.
    fn edit(
        &self,
        py: Python<'_>,
        edit: impl FnOnce(&mut rdocx::Cell<'_>) -> rdocx::Result<()>,
    ) -> PyResult<()> {
        let (table, row, cell) = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        TableLocation::new(table, &self.related)
            .edit(&mut candidate, |table| {
                let mut cell = table
                    .cell(row, cell)
                    .ok_or_else(|| rdocx::Error::Other("cell index out of range".to_owned()))?;
                edit(&mut cell)
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        document.inner = candidate;
        Ok(())
    }
}

#[pymethods]
impl PyCell {
    #[pyo3(signature = (data, filename=None, width=None, height=None, *, options=None))]
    fn add_picture(
        &self,
        py: Python<'_>,
        data: &Bound<'_, PyAny>,
        filename: Option<&str>,
        width: Option<&Bound<'_, PyAny>>,
        height: Option<&Bound<'_, PyAny>>,
        options: Option<PyRef<'_, crate::document::PyPictureOptions>>,
    ) -> PyResult<Py<crate::document::PyPicture>> {
        let (table, row, cell) = self.validate(py)?;
        let (data, filename) = crate::document::picture_input(data, filename)?;
        let width = crate::document::checked_section_length(width, "picture width")?
            .map(rdocx::Length::emu);
        let height = crate::document::checked_section_length(height, "picture height")?
            .map(rdocx::Length::emu);
        let mut document = self.document.borrow_mut(py);
        let story = self.physical_story(py, &document.inner, table, row, cell)?;
        let mut candidate = document.inner.clone_for_staging();
        let location = candidate
            .insert_picture_to_story_with_options(
                &story,
                None,
                &data,
                &filename,
                width,
                height,
                options.map(|o| o.inner.clone()),
            )
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        let picture = candidate
            .pictures()
            .map_err(|error| rdocx_to_pyerr(py, error))?
            .into_iter()
            .find(|p| p.location == location)
            .ok_or_else(|| PyIndexError::new_err("inserted cell picture was not inventoried"))?;
        let prepared = document
            .revisions
            .prepare_scopes("Cell.add_picture", &[], &[])
            .map_err(revision_to_pyerr)?;
        let result = Py::new(
            py,
            crate::document::PyPicture {
                inner: picture,
                binding: document
                    .revisions
                    .capture_prepared(&prepared, smallvec![])
                    .map_err(revision_to_pyerr)?,
            },
        )?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(revision_to_pyerr)?;
        document.inner = candidate;
        Ok(result)
    }

    /// Replace literal text in this cell and its supported nested descendants.
    #[pyo3(signature = (old, new, *, expect = None))]
    fn replace_text(
        &self,
        py: Python<'_>,
        old: &str,
        new: &str,
        expect: Option<usize>,
    ) -> PyResult<usize> {
        let (table, row, cell) = self.validate(py)?;
        let related = match &self.related {
            Some(location) => {
                let document = self.document.borrow(py);
                let location = TableLocation::refreshed(&document.inner, location)
                    .map_err(|error| rdocx_to_pyerr(py, error))?;
                Some(
                    document
                        .inner
                        .story_table_cell(&location, row, cell)
                        .map_err(|error| rdocx_to_pyerr(py, error))?,
                )
            }
            None => None,
        };
        self.document
            .borrow_mut(py)
            .scoped_replacement(py, "Cell.replace_text", |document| match related.as_ref() {
                Some(story) => document.try_replace_text_in_story_cell(story, old, new, expect),
                None => {
                    document.try_replace_text_in_cell((table, row, cell), None, old, new, expect)
                }
            })
    }

    fn split(&self, py: Python<'_>) -> PyResult<()> {
        let (table, row, cell) = self.validate(py)?;
        let noop = self.read(py, |cell| {
            cell.grid_span().unwrap_or(1) == 1
                && cell.v_merge().is_none()
                && !cell.has_horizontal_merge()
        })?;
        if noop {
            return Ok(());
        }
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        match &self.related {
            Some(location) => {
                let location = TableLocation::refreshed(&candidate, location)
                    .map_err(|error| rdocx_to_pyerr(py, error))?;
                candidate.edit_story_table(&location, |table| {
                    table.split_cell(row, cell)?;
                    Ok(())
                })
            }
            None => {
                let logical_column = candidate
                    .table(table)
                    .and_then(|table| {
                        let row = table.row(row)?;
                        (0..cell).try_fold(
                            row.grid_before().unwrap_or(0) as usize,
                            |column, index| {
                                column
                                    .checked_add(row.cell(index)?.grid_span().unwrap_or(1) as usize)
                            },
                        )
                    })
                    .ok_or_else(|| {
                        PyValueError::new_err("cell has no checked logical grid position")
                    })?;
                candidate
                    .split_table_cell(table, row, logical_column)
                    .map(|_| ())
            }
        }
        .map_err(|error| rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Cell.split", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(())
    }

    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        self.read(py, |cell| cell.text())
    }
    #[getter]
    fn tables<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let (table, row, cell) = self.validate(py)?;
        let document = self.document.borrow(py);
        let story = self.physical_story(py, &document.inner, table, row, cell)?;
        let mut tables = Vec::new();
        for item in document
            .inner
            .story_items(&story)
            .map_err(|error| rdocx_to_pyerr(py, error))?
        {
            if item.kind() == rdocx::StoryItemKind::Table {
                let location = item.location().clone();
                let path = document
                    .revisions
                    .capture(smallvec![PathSeg::Body(location.index_path()[0])]);
                tables.push(Py::new(
                    py,
                    PyTable::for_story(self.document.clone_ref(py), path, location),
                )?);
            }
        }
        PyTuple::new(py, tables)
    }
    fn add_table(&self, py: Python<'_>, rows: usize, cols: usize) -> PyResult<Py<PyTable>> {
        let (table, row, cell) = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        let story = self.physical_story(py, &candidate, table, row, cell)?;
        let before = candidate
            .story_items(&story)
            .map_err(|error| rdocx_to_pyerr(py, error))?
            .into_iter()
            .filter(|item| item.kind() == rdocx::StoryItemKind::Table)
            .count();
        TableLocation::new(table, &self.related)
            .edit(&mut candidate, |table| {
                table
                    .cell(row, cell)
                    .ok_or_else(|| rdocx::Error::Other("cell index out of range".to_owned()))?
                    .add_table_checked(rows, cols)?;
                Ok(())
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?;
        let story = self.physical_story(py, &candidate, table, row, cell)?;
        let tables = candidate
            .story_items(&story)
            .map_err(|error| rdocx_to_pyerr(py, error))?
            .into_iter()
            .filter(|item| item.kind() == rdocx::StoryItemKind::Table)
            .map(|item| item.location().clone())
            .collect::<Vec<_>>();
        if tables.len() != before + 1 {
            return Err(PyValueError::new_err(
                "nested table insertion did not retain physical owner correspondence",
            ));
        }
        let location = tables
            .last()
            .cloned()
            .ok_or_else(|| PyValueError::new_err("nested table was not published"))?;
        let prepared = document
            .revisions
            .prepare_scopes("Cell.add_table", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        let path = document
            .revisions
            .capture_prepared(
                &prepared,
                smallvec![PathSeg::Body(location.index_path()[0])],
            )
            .map_err(oxml_py_support::revision_to_pyerr)?;
        let result = Py::new(
            py,
            PyTable::for_story(self.document.clone_ref(py), path, location),
        )?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(result)
    }
    #[setter]
    fn set_text(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        let (table, row, cell) = self.validate(py)?;
        let mut document = self.document.borrow_mut(py);
        let mut candidate = document.inner.clone_for_staging();
        match &self.related {
            Some(_) => TableLocation::new(table, &self.related).edit(&mut candidate, |table| {
                table
                    .cell(row, cell)
                    .ok_or_else(|| rdocx::Error::Other("cell index out of range".to_owned()))?
                    .try_set_text(value)
            }),
            None => candidate.try_set_cell_text(table, row, cell, value),
        }
        .map_err(|error| rdocx_to_pyerr(py, error))?;
        let prepared = document
            .revisions
            .prepare_scopes("Cell.text", &[], &[])
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document
            .revisions
            .commit_scopes(prepared)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        document.inner = candidate;
        Ok(())
    }
    #[getter]
    fn paragraphs(&self, py: Python<'_>) -> PyResult<Py<PyCellParagraphCollection>> {
        self.validate(py)?;
        Py::new(
            py,
            PyCellParagraphCollection::new(
                self.document.clone_ref(py),
                self.path.clone(),
                self.related.clone(),
            ),
        )
    }
    fn add_paragraph(&self, py: Python<'_>, text: &str) -> PyResult<Py<PyParagraph>> {
        let (table, row, cell) = self.validate(py)?;
        let result = {
            let mut document = self.document.borrow_mut(py);
            let mut candidate = document.inner.clone_for_staging();
            let paragraph = TableLocation::new(table, &self.related)
                .edit(&mut candidate, |table| {
                    let mut cell = table
                        .cell(row, cell)
                        .ok_or_else(|| rdocx::Error::Other("cell index out of range".to_owned()))?;
                    let paragraph = cell.paragraph_count();
                    cell.add_paragraph(text);
                    Ok(paragraph)
                })
                .map_err(|error| rdocx_to_pyerr(py, error))?;
            let prepared = document
                .revisions
                .prepare_scopes("Cell.add_paragraph", &[], &[])
                .map_err(oxml_py_support::revision_to_pyerr)?;
            let path = document
                .revisions
                .capture_prepared(
                    &prepared,
                    smallvec![
                        PathSeg::Body(table),
                        PathSeg::Row(row),
                        PathSeg::Cell(cell),
                        PathSeg::Para(paragraph)
                    ],
                )
                .map_err(oxml_py_support::revision_to_pyerr)?;
            let paragraph = match &self.related {
                Some(location) => {
                    let location = TableLocation::refreshed(&candidate, location)
                        .map_err(|error| rdocx_to_pyerr(py, error))?;
                    let location = candidate
                        .story_table_cell_paragraph_location(&location, row, cell, paragraph)
                        .map_err(|error| rdocx_to_pyerr(py, error))?;
                    PyParagraph::for_story(self.document.clone_ref(py), path, location)
                }
                None => PyParagraph::new(self.document.clone_ref(py), path),
            };
            let result = Py::new(py, paragraph)?;
            document
                .revisions
                .commit_scopes(prepared)
                .map_err(oxml_py_support::revision_to_pyerr)?;
            document.inner = candidate;
            result
        };
        Ok(result)
    }
    #[getter]
    fn width(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |cell| cell.width())?
            .map(|value| length_object(py, value))
            .transpose()
    }
    #[setter]
    fn set_width(&self, py: Python<'_>, value: i64) -> PyResult<()> {
        self.edit(py, |cell| {
            cell.set_width(rdocx::Length::emu(value));
            Ok(())
        })
    }
    #[getter]
    fn vertical_alignment(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |cell| cell.vertical_alignment())?
            .map(|value| enum_object(py, "WD_CELL_VERTICAL_ALIGNMENT", vertical_to_int(value)))
            .transpose()
    }
    #[setter]
    fn set_vertical_alignment(&self, py: Python<'_>, value: i32) -> PyResult<()> {
        let value = vertical_from_int(value)?;
        self.edit(py, |cell| {
            cell.set_vertical_alignment(value);
            Ok(())
        })
    }

    #[getter]
    fn grid_span(&self, py: Python<'_>) -> PyResult<u32> {
        self.read(py, |cell| cell.grid_span().unwrap_or(1))
    }

    #[getter]
    fn vertical_merge(&self, py: Python<'_>) -> PyResult<Option<&'static str>> {
        self.read(py, |cell| cell.v_merge().copied().map(vertical_merge_name))
    }

    #[getter]
    fn shading(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.read(py, |cell| cell.shading_fill().map(str::to_owned))?
            .as_deref()
            .map(|value| color_object(py, value))
            .transpose()
    }

    #[setter]
    fn set_shading(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let value = table_color_hex(value, "Cell.shading")?;
        self.edit(py, |cell| cell.set_shading_checked(&value))
    }

    fn border(&self, py: Python<'_>, edge: &str) -> PyResult<Option<BorderSnapshot>> {
        let edge = cell_border_edge(edge)?;
        self.read(py, |cell| {
            cell.border(edge).map(|border| border_snapshot(py, border))
        })?
        .transpose()
    }

    #[pyo3(signature = (edge, style, *, size, color))]
    fn set_border(
        &self,
        py: Python<'_>,
        edge: &str,
        style: &str,
        size: u32,
        color: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let color = table_color_hex(color, "Cell.set_border(color)")?;
        let edge = cell_border_edge(edge)?;
        let style = border_style_from_name(style)?;
        self.edit(py, |cell| {
            cell.set_border_checked(edge, style, size, &color)
        })
    }

    #[getter]
    fn margins(&self, py: Python<'_>) -> PyResult<Option<MarginSnapshot>> {
        self.read(py, |cell| cell.margins())?
            .map(|margins| margin_snapshot(py, margins))
            .transpose()
    }

    #[pyo3(signature = (*, top, right, bottom, left))]
    fn set_margins(
        &self,
        py: Python<'_>,
        top: i64,
        right: i64,
        bottom: i64,
        left: i64,
    ) -> PyResult<()> {
        self.edit(py, |cell| {
            cell.set_margins_checked(
                rdocx::Length::emu(top),
                rdocx::Length::emu(right),
                rdocx::Length::emu(bottom),
                rdocx::Length::emu(left),
            )
        })
    }
}

#[pyclass(name = "CellParagraphCollection")]
pub struct PyCellParagraphCollection {
    document: Py<PyDocument>,
    cell_path: ContentPath,
    related: Option<rdocx::ContentLocation>,
}

impl PyCellParagraphCollection {
    fn new(
        document: Py<PyDocument>,
        cell_path: ContentPath,
        related: Option<rdocx::ContentLocation>,
    ) -> Self {
        Self {
            document,
            cell_path,
            related,
        }
    }
    fn validate(&self, py: Python<'_>) -> PyResult<(usize, usize, usize)> {
        let document = self.document.borrow(py);
        self.cell_path
            .validate_scopes(
                &document.revisions,
                "cell paragraph collection",
                "Re-fetch it with cell.paragraphs.",
            )
            .map_err(|error| stale_to_pyerr(py, error))?;
        Ok((
            table_index(&self.cell_path)?,
            row_index(&self.cell_path)?,
            cell_index(&self.cell_path)?,
        ))
    }
    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let (table, row, cell) = self.validate(py)?;
        TableLocation::new(table, &self.related)
            .read(&self.document.borrow(py).inner, |table| {
                table.cell(row, cell).map(|cell| cell.paragraph_count())
            })
            .map_err(|error| rdocx_to_pyerr(py, error))?
            .ok_or_else(|| PyIndexError::new_err("cell index out of range"))
    }
    fn item(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyParagraph>> {
        self.validate(py)?;
        let mut segments = self.cell_path.segs.clone();
        segments.push(PathSeg::Para(index));
        let path = self.document.borrow(py).revisions.capture(segments);
        let related = match &self.related {
            Some(location) => {
                let (_, row, cell) = self.validate(py)?;
                let document = self.document.borrow(py);
                let location = TableLocation::refreshed(&document.inner, location)
                    .map_err(|error| rdocx_to_pyerr(py, error))?;
                Some(
                    document
                        .inner
                        .story_table_cell_paragraph_location(&location, row, cell, index)
                        .map_err(|error| rdocx_to_pyerr(py, error))?,
                )
            }
            None => None,
        };
        let paragraph = match related {
            Some(location) => PyParagraph::for_story(self.document.clone_ref(py), path, location),
            None => PyParagraph::new(self.document.clone_ref(py), path),
        };
        Py::new(py, paragraph)
    }
}

#[pymethods]
impl PyCellParagraphCollection {
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.len(py)
    }
    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let len = self.len(py)?;
        if let Ok(index) = key.extract::<isize>() {
            return Ok(self
                .item(py, normalize_index(index, len, "paragraph")?)?
                .into_any());
        }
        if key.is_instance_of::<PySlice>() {
            let (start, stop, step): (isize, isize, isize) =
                key.call_method1("indices", (len,))?.extract()?;
            let items = PyList::empty(py);
            let mut index = start;
            while if step > 0 { index < stop } else { index > stop } {
                items.append(self.item(py, index as usize)?)?;
                index += step;
            }
            return Ok(items.into_any().unbind());
        }
        Err(PyTypeError::new_err(
            "paragraph indices must be integers or slices",
        ))
    }
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyCellParagraphIterator>> {
        self.validate(py)?;
        Py::new(
            py,
            PyCellParagraphIterator {
                document: self.document.clone_ref(py),
                cell_path: self.cell_path.clone(),
                related: self.related.clone(),
                index: 0,
            },
        )
    }
}

#[pyclass]
struct PyCellParagraphIterator {
    document: Py<PyDocument>,
    cell_path: ContentPath,
    index: usize,
    related: Option<rdocx::ContentLocation>,
}

#[pymethods]
impl PyCellParagraphIterator {
    fn __iter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyParagraph>>> {
        let collection = PyCellParagraphCollection::new(
            self.document.clone_ref(py),
            self.cell_path.clone(),
            self.related.clone(),
        );
        if self.index >= collection.len(py)? {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        collection.item(py, index).map(Some)
    }
}
