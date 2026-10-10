"""Text-related public enumerations."""

from enum import IntEnum


class WD_ALIGN_PARAGRAPH(IntEnum):
    """Paragraph horizontal alignment."""

    LEFT = 0
    CENTER = 1
    RIGHT = 2
    JUSTIFY = 3


class WD_UNDERLINE(IntEnum):
    """Underline styles supported by the rdocx run facade."""

    NONE = 0
    SINGLE = 1
    WORDS = 2
    DOUBLE = 3
    DOTTED = 4
    THICK = 6
    DASH = 7
    DOT_DASH = 9
    DOT_DOT_DASH = 10
    WAVY = 11


class WD_TAB_ALIGNMENT(IntEnum):
    """Tab stop alignments supported by the rdocx paragraph facade."""

    LEFT = 0
    CENTER = 1
    RIGHT = 2
    DECIMAL = 3


class WD_TAB_LEADER(IntEnum):
    """Tab leader characters supported by the rdocx paragraph facade."""

    SPACES = 0
    DOTS = 1
    DASHES = 2
    LINES = 3


class WD_BREAK(IntEnum):
    """Break types for ``Run.add_break``, with python-docx's values."""

    LINE = 6
    PAGE = 7
    COLUMN = 8
    LINE_CLEAR_LEFT = 9
    LINE_CLEAR_RIGHT = 10
    LINE_CLEAR_ALL = 11
    TEXT_WRAPPING = 11


WD_BREAK_TYPE = WD_BREAK


class WD_COLOR_INDEX(IntEnum):
    """Highlight colors, numbered as python-docx numbers them."""

    AUTO = 0
    BLACK = 1
    BLUE = 2
    TURQUOISE = 3
    BRIGHT_GREEN = 4
    PINK = 5
    RED = 6
    YELLOW = 7
    WHITE = 8
    DARK_BLUE = 9
    TEAL = 10
    GREEN = 11
    VIOLET = 12
    DARK_RED = 13
    DARK_YELLOW = 14
    GRAY_50 = 15
    GRAY_25 = 16


WD_COLOR = WD_COLOR_INDEX


__all__ = [
    "WD_ALIGN_PARAGRAPH",
    "WD_BREAK",
    "WD_BREAK_TYPE",
    "WD_COLOR",
    "WD_COLOR_INDEX",
    "WD_TAB_ALIGNMENT",
    "WD_TAB_LEADER",
    "WD_UNDERLINE",
]
