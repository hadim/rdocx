"""Python bindings for rpptx."""

from .enum.dml import MSO_THEME_COLOR, MSO_PATTERN_TYPE, MSO_PATTERN
from .dml.color import RGBColor
from .enum.shapes import MSO_SHAPE, PP_PLACEHOLDER, PP_PLACEHOLDER_TYPE
from .enum.text import MSO_ANCHOR, MSO_AUTO_SIZE, MSO_UNDERLINE, PP_ALIGN
from .util import Inches, Length, Pt


class RpptxError(Exception):
    """Base class for errors raised by rpptx."""


class PackageError(RpptxError):
    """An OPC package, file, or presentation operation failed."""


class XmlError(RpptxError):
    """PresentationML could not be parsed or serialized."""


class StaleElementError(RpptxError):
    """A held content handle was invalidated by structural mutation."""


class ReplacementCountError(RpptxError):
    """A counted replacement matched a different number of times than expected."""

    def __init__(self, message: str, expected: int, found: int) -> None:
        super().__init__(message, expected, found)
        self.expected = expected
        self.found = found
        self.index: int | None = None
        self.placeholder: str | None = None

    def __str__(self) -> str:
        return str(self.args[0])


from ._rpptx import Element, Comment, CommentAuthor, CommentReply, Presentation, CoreProperties, AppProperties, CustomProperties, CustomProperty, CustomFileTime, Section, MediaInfo, GradientStops, GradientStop
from ._rpptx import SlideTransition, HeaderFooter, PlaceholderFormat, SlideMaster, SlideMasterCollection, Theme, ThemeFonts, ThemeFontSet, ThemeColors, MasterTextStyles, TextStyleList
from ._rpptx import FitReport, FitFrameReport, FitTrial, AutofitRefreshResult, AutofitUpdate, BoundingBox, TextFrameLayout, TextLineLayout, ValidationIssue

__all__ = [
    "Element",
    "MSO_PATTERN_TYPE", "MSO_PATTERN", "CoreProperties", "AppProperties", "CustomProperties", "CustomProperty", "CustomFileTime", "Section", "MediaInfo", "GradientStops", "GradientStop",
    "MSO_THEME_COLOR",
    "BoundingBox",
    "FitReport",
    "FitFrameReport",
    "FitTrial",
    "AutofitRefreshResult",
    "AutofitUpdate",
    "Inches",
    "Length",
    "MSO_ANCHOR",
    "MSO_AUTO_SIZE",
    "MSO_SHAPE",
    "PP_PLACEHOLDER",
    "PP_PLACEHOLDER_TYPE",
    "HeaderFooter",
    "SlideTransition",
    "PlaceholderFormat",
    "SlideMaster",
    "SlideMasterCollection",
    "Theme", "ThemeColors", "MasterTextStyles", "TextStyleList",
    "ThemeFonts",
    "ThemeFontSet",
    "MSO_UNDERLINE",
    "PP_ALIGN",
    "PackageError",
    "Comment",
    "CommentAuthor",
    "CommentReply",
    "Presentation",
    "Pt",
    "RGBColor",
    "ReplacementCountError",
    "RpptxError",
    "StaleElementError",
    "TextFrameLayout",
    "TextLineLayout",
    "ValidationIssue",
    "XmlError",
]
