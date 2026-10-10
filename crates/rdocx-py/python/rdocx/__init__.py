"""Python bindings for rdocx."""

from .enum.dml import MSO_COLOR_TYPE, MSO_THEME_COLOR
from .enum.table import WD_CELL_VERTICAL_ALIGNMENT, WD_ROW_HEIGHT_RULE, WD_TABLE_ALIGNMENT
from .enum.text import WD_ALIGN_PARAGRAPH, WD_BREAK, WD_TAB_ALIGNMENT, WD_TAB_LEADER, WD_UNDERLINE
from .shared import Cm, Emu, Inches, Length, Mm, Pt, RGBColor, Twips


class RdocxError(Exception):
    """Base class for errors raised by rdocx."""


class PackageError(RdocxError):
    """An OPC package, file, or document-part operation failed."""


class XmlError(RdocxError):
    """WordprocessingML could not be parsed or serialized."""


class StaleElementError(RdocxError):
    """A held content handle was invalidated by structural mutation."""


class LayoutError(RdocxError):
    """Document layout or rendering failed."""


class ReplacementCountError(RdocxError):
    """A counted replacement matched a different number of times than expected.

    ``index`` is the position of the failing pair in a ``Document.replace_all``
    batch, and ``None`` for ``Document.try_replace_text``.
    """

    def __init__(
        self, message: str, expected: int, found: int, index: "int | None" = None
    ) -> None:
        super().__init__(message, expected, found, index)
        self.expected = expected
        self.found = found
        self.index = index

    def __str__(self) -> str:
        return str(self.args[0])


class ConversionWarning(UserWarning):
    """An export or a text view left out content it could not represent."""


from ._rdocx import (
    AppProperties,
    Bookmark,
    BoundingBox,
    Cell,
    CellCollection,
    CellParagraphCollection,
    ColorFormat,
    Column,
    Comment,
    ComparisonDiagnostic,
    ContentControl,
    ContentFragment,
    CoreProperties,
    CustomProperties,
    Document,
    DocumentFragment,
    Equation,
    Font,
    HeaderFooter,
    HeaderFooterCell,
    HeaderFooterRow,
    HeaderFooterTable,
    HeaderFooterVariant,
    Hyperlink,
    LayoutFragment,
    LayoutBackedFieldUpdateReport,
    LayoutPage,
    ListLevel,
    Paragraph,
    ParagraphCollection,
    ParagraphFormat,
    Picture,
    Revision,
    Row,
    RowCollection,
    Run,
    RunCollection,
    RunPosition,
    RunRange,
    Section,
    Settings,
    Story,
    StoryItem,
    StoryRunPosition,
    StoryRunRange,
    Style,
    SvgDiagnostic,
    SvgRenderResult,
    Table,
    TableCollection,
    TabStop,
    TabStops,
    TocRebuildReport,
    ValidationReport,
)

# The custom properties behave as a mutable mapping of names to typed values.
import collections.abc as _abc

_abc.MutableMapping.register(CustomProperties)

__all__ = [
    "AppProperties",
    "Bookmark",
    "BoundingBox",
    "Cm",
    "Cell",
    "CellCollection",
    "CellParagraphCollection",
    "ColorFormat",
    "Column",
    "Comment",
    "ComparisonDiagnostic",
    "ContentControl",
    "ContentFragment",
    "ConversionWarning",
    "CoreProperties",
    "CustomProperties",
    "Document",
    "DocumentFragment",
    "Emu",
    "Equation",
    "Font",
    "HeaderFooter",
    "HeaderFooterCell",
    "HeaderFooterRow",
    "HeaderFooterTable",
    "HeaderFooterVariant",
    "Hyperlink",
    "Inches",
    "LayoutError",
    "Length",
    "LayoutFragment",
    "LayoutBackedFieldUpdateReport",
    "LayoutPage",
    "ListLevel",
    "MSO_COLOR_TYPE",
    "MSO_THEME_COLOR",
    "Mm",
    "PackageError",
    "Paragraph",
    "ParagraphCollection",
    "ParagraphFormat",
    "Picture",
    "Pt",
    "RGBColor",
    "RdocxError",
    "ReplacementCountError",
    "Revision",
    "Row",
    "RowCollection",
    "Run",
    "RunCollection",
    "RunPosition",
    "RunRange",
    "Section",
    "Settings",
    "StaleElementError",
    "Story",
    "StoryItem",
    "StoryRunPosition",
    "StoryRunRange",
    "Style",
    "SvgDiagnostic",
    "SvgRenderResult",
    "Table",
    "TableCollection",
    "TabStop",
    "TabStops",
    "TocRebuildReport",
    "Twips",
    "ValidationReport",
    "WD_ALIGN_PARAGRAPH",
    "WD_BREAK",
    "WD_CELL_VERTICAL_ALIGNMENT",
    "WD_ROW_HEIGHT_RULE",
    "WD_TAB_ALIGNMENT",
    "WD_TAB_LEADER",
    "WD_TABLE_ALIGNMENT",
    "WD_UNDERLINE",
    "XmlError",
]
