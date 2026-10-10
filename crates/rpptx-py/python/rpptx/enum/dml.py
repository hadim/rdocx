"""DrawingML enumerations, mirroring python-pptx `pptx.enum.dml`."""

from enum import IntEnum


class MSO_FILL_TYPE(IntEnum):
    """The kind of a fill, as reported by `FillFormat.type`."""

    BACKGROUND = 5
    GRADIENT = 3
    GROUP = 101
    PATTERNED = 2
    PICTURE = 6
    SOLID = 1
    TEXTURED = 4


MSO_FILL = MSO_FILL_TYPE


class MSO_LINE_DASH_STYLE(IntEnum):
    """The preset dash of a line, as `LineFormat.dash_style` reads and writes it.

    The first nine members carry the python-pptx 1.0.2 values and write the
    same `a:prstDash` value python-pptx writes. python-pptx cannot read
    `dot`, `sysDashDot` or `sysDashDotDot`, which PowerPoint for Mac's
    scripting interface writes, so rpptx adds `DOT`, `SYSTEM_DASH_DOT` and
    `SYSTEM_DASH_DOT_DOT` to read every preset back. `SYSTEM_DASH_DOT` takes
    the value that scripting interface gives `sysDashDot`, the other two take
    values outside `MsoLineDashStyle`.
    `DASH_STYLE_MIXED` is never read and cannot be written.
    """

    DASH = 4
    DASH_DOT = 5
    DASH_DOT_DOT = 6
    LONG_DASH = 7
    LONG_DASH_DOT = 8
    ROUND_DOT = 3
    SOLID = 1
    SQUARE_DOT = 2
    DASH_STYLE_MIXED = -2
    SYSTEM_DASH_DOT = 12
    DOT = 13
    SYSTEM_DASH_DOT_DOT = 14


MSO_LINE = MSO_LINE_DASH_STYLE


class MSO_ARROWHEAD_STYLE(IntEnum):
    """The shape of a line end, the `type` of `a:headEnd` or `a:tailEnd`.

    Values follow `MsoArrowheadStyle`. `OPEN` is the `arrow` end.
    """

    NONE = 1
    TRIANGLE = 2
    OPEN = 3
    STEALTH = 4
    DIAMOND = 5
    OVAL = 6


class MSO_ARROWHEAD_WIDTH(IntEnum):
    """The width of a line end, its `w` of `sm`, `med` or `lg`."""

    NARROW = 1
    MEDIUM = 2
    WIDE = 3


class MSO_ARROWHEAD_LENGTH(IntEnum):
    """The length of a line end, its `len` of `sm`, `med` or `lg`."""

    SHORT = 1
    MEDIUM = 2
    LONG = 3


class MSO_THEME_COLOR(IntEnum):
    """Supported Office theme colours with their pinned standard values."""

    NOT_THEME_COLOR = 0
    DARK_1 = 1
    LIGHT_1 = 2
    DARK_2 = 3
    LIGHT_2 = 4
    ACCENT_1 = 5
    ACCENT_2 = 6
    ACCENT_3 = 7
    ACCENT_4 = 8
    ACCENT_5 = 9
    ACCENT_6 = 10
    HYPERLINK = 11
    FOLLOWED_HYPERLINK = 12
    TEXT_1 = 13
    BACKGROUND_1 = 14
    TEXT_2 = 15
    BACKGROUND_2 = 16
    MIXED = -2


__all__ = [
    "MSO_THEME_COLOR",
    "MSO_ARROWHEAD_LENGTH",
    "MSO_ARROWHEAD_STYLE",
    "MSO_ARROWHEAD_WIDTH",
    "MSO_FILL",
    "MSO_FILL_TYPE",
    "MSO_LINE",
    "MSO_LINE_DASH_STYLE",
]
