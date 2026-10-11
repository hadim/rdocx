"""Chart enumerations, mirroring python-pptx `pptx.enum.chart`."""

from enum import IntEnum


class XL_CHART_TYPE(IntEnum):
    """Chart types `add_chart` can author, with python-pptx values.

    Each member is the type python-pptx reads back from the same chart in a
    deck. The other python-pptx chart types, such as stacked, 3-D or bar
    (horizontal) charts, are absent because rdocx cannot author them.
    """

    AREA = 1
    COLUMN_CLUSTERED = 51
    DOUGHNUT = -4120
    LINE_MARKERS = 65
    PIE = 5
    RADAR_MARKERS = 81
    XY_SCATTER_LINES = 74


__all__ = ["XL_CHART_TYPE"]
