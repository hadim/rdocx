import datetime as _datetime
import os as _os
from collections.abc import (
    Iterator as _Iterator,
    Mapping as _Mapping,
    MutableMapping as _MutableMapping,
    Sequence as _Sequence,
)
from typing import IO as _IO, Any as _Any, BinaryIO as _BinaryIO
from typing import Literal as _Literal, NoReturn as _Never, final as _final, overload as _overload

from . import shared as _shared
from .enum import dml as _dml
from .enum import table as _table
from .enum import text as _text

_Path = str | _os.PathLike[str]
_Image = bytes | str | _os.PathLike[str] | _BinaryIO
_PictureWrap = _Literal[
    "inline", "square", "tight", "through", "top_and_bottom", "behind", "in_front"
]
_HorizontalAlign = _Literal["left", "center", "right", "inside", "outside"]
_VerticalAlign = _Literal["top", "center", "bottom", "inside", "outside"]
_HorizontalFrom = _Literal[
    "page", "margin", "column", "character", "left_margin", "right_margin",
    "inside_margin", "outside_margin",
]
_VerticalFrom = _Literal[
    "page", "margin", "paragraph", "line", "top_margin", "bottom_margin",
    "inside_margin", "outside_margin",
]
_RevisionView = _Literal["accepted", "tracked"]
_Xml = str | bytes | bytearray
_BorderStyle = _Literal[
    "none", "single", "thick", "double", "dotted", "dashed", "dotDash", "wave"
]
_BorderEdge = _Literal["top", "bottom", "left", "right", "insideH", "insideV"]
_Color = _shared.RGBColor | tuple[int, int, int] | str
_ParagraphBorderEdge = _Literal["top", "bottom", "left", "right", "between", "bar"]
_Margins = tuple[
    _shared.Length | None, _shared.Length | None, _shared.Length | None, _shared.Length | None
]
__all__ = [
    "Bookmark", "BoundingBox", "Cell", "CellCollection", "CellParagraphCollection", "ColorFormat",
    "Column",
    "Comment", "ComparisonDiagnostic", "ContentFragment", "CoreProperties", "Document", "Font",
    "HeaderFooter", "HeaderFooterCell", "HeaderFooterRow", "HeaderFooterTable", "HeaderFooterVariant", "Hyperlink", "LayoutBackedFieldUpdateReport", "LayoutFragment", "LayoutPage", "ListLevel", "Paragraph", "ParagraphCollection",
    "ParagraphFormat", "Picture", "Revision", "Row", "RowCollection", "Run", "RunCollection", "RunPosition",
    "RunRange", "Section", "Story", "StoryItem", "StoryRunPosition", "StoryRunRange", "Style",
    "SvgDiagnostic", "SvgRenderResult", "Table", "TableCollection", "TabStop", "TabStops",
    "TocRebuildReport",
    "AppProperties", "ContentControl", "CustomProperties", "DocumentFragment", "Equation",
    "Settings", "ValidationReport",
]
_CustomPropertyValue = str | int | float | bool | _datetime.datetime
_ConflictPolicy = _Literal["reuse_equivalent", "rename"]
_ContentControlType = _Literal[
    "rich_text", "plain_text", "picture", "checkbox", "combo_box", "dropdown_list", "date",
    "document_part_list", "document_part_object", "group", "repeating_section",
    "repeating_section_item", "citation", "equation", "bibliography",
]


@_final
class RunPosition:
    def __new__(cls, *, body_index: int, run_index: int) -> RunPosition: ...
    @property
    def body_index(self) -> int: ...
    @property
    def run_index(self) -> int: ...


@_final
class RunRange:
    def __new__(cls, *, start: RunPosition, end: RunPosition) -> RunRange: ...
    @property
    def start(self) -> RunPosition: ...
    @property
    def end(self) -> RunPosition: ...


@_final
class Bookmark:
    """A bookmark, or a marker problem, read by ``Document.bookmarks``.

    ``range`` counts paragraphs through tables and block content controls.
    ``direct_range`` uses the direct body index ``add_bookmark`` takes and is
    ``None`` when a marker sits in a table cell or a block content control.
    """

    def __new__(
        cls,
        *,
        id: int | None,
        name: str | None,
        range: RunRange | None,
        direct_range: RunRange | None,
        text: str,
        issue: str | None,
    ) -> Bookmark: ...
    @property
    def id(self) -> int | None: ...
    @property
    def name(self) -> str | None: ...
    @property
    def range(self) -> RunRange | None: ...
    @property
    def direct_range(self) -> RunRange | None: ...
    @property
    def text(self) -> str: ...
    @property
    def issue(self) -> str | None: ...


@_final
class Comment:
    def __new__(
        cls,
        *,
        id: int,
        author: str | None,
        initials: str | None,
        date: str | None,
        text: str,
        parent_id: int | None,
        resolved: bool,
        anchor_text: str | None = None,
        anchor: StoryRunRange | None = None,
    ) -> Comment: ...
    @property
    def id(self) -> int: ...
    @property
    def author(self) -> str | None: ...
    @property
    def initials(self) -> str | None: ...
    @property
    def date(self) -> str | None: ...
    @property
    def text(self) -> str: ...
    @property
    def parent_id(self) -> int | None: ...
    @property
    def resolved(self) -> bool: ...
    @property
    def anchor_text(self) -> str | None: ...
    @property
    def anchor(self) -> StoryRunRange | None: ...


@_final
class ComparisonDiagnostic:
    def __new__(cls, *, location: str, message: str) -> ComparisonDiagnostic: ...
    @property
    def location(self) -> str: ...
    @property
    def message(self) -> str: ...


@_final
class SvgDiagnostic:
    def __new__(cls, *, path: str, message: str) -> SvgDiagnostic: ...
    @property
    def path(self) -> str: ...
    @property
    def message(self) -> str: ...


@_final
class SvgRenderResult:
    def __new__(
        cls, *, svg: str, diagnostics: _Sequence[SvgDiagnostic]
    ) -> SvgRenderResult: ...
    @property
    def svg(self) -> str: ...
    @property
    def diagnostics(self) -> tuple[SvgDiagnostic, ...]: ...


@_final
class BoundingBox:
    def __new__(
        cls, *, x: float, y: float, width: float, height: float
    ) -> BoundingBox: ...
    @property
    def x(self) -> float: ...
    @property
    def y(self) -> float: ...
    @property
    def width(self) -> float: ...
    @property
    def height(self) -> float: ...


@_final
class LayoutFragment:
    def __new__(
        cls,
        *,
        body_index: int,
        physical_page: int,
        displayed_page: int,
        bounds: BoundingBox,
    ) -> LayoutFragment: ...
    @property
    def body_index(self) -> int: ...
    @property
    def physical_page(self) -> int: ...
    @property
    def displayed_page(self) -> int: ...
    @property
    def bounds(self) -> BoundingBox: ...


@_final
class LayoutPage:
    def __new__(
        cls,
        *,
        page_number: int,
        displayed_page_number: int,
        width: float,
        height: float,
    ) -> LayoutPage: ...
    @property
    def page_number(self) -> int: ...
    @property
    def displayed_page_number(self) -> int: ...
    @property
    def width(self) -> float: ...
    @property
    def height(self) -> float: ...


@_final
class LayoutBackedFieldUpdateReport:
    def __new__(
        cls,
        *,
        page_fields: int,
        num_pages_fields: int,
        page_reference_fields: int,
        diagnostics: tuple[str, ...],
        section_fields: int = 0,
        section_pages_fields: int = 0,
    ) -> LayoutBackedFieldUpdateReport: ...
    @property
    def page_fields(self) -> int: ...
    @property
    def num_pages_fields(self) -> int: ...
    @property
    def page_reference_fields(self) -> int: ...
    @property
    def section_fields(self) -> int: ...
    @property
    def section_pages_fields(self) -> int: ...
    @property
    def updated_count(self) -> int: ...
    @property
    def diagnostics(self) -> tuple[str, ...]: ...
    @property
    def diagnostic_count(self) -> int: ...


@_final
class TocRebuildReport:
    def __new__(
        cls, *, entry_count: int, bookmark_count: int, diagnostics: tuple[str, ...]
    ) -> TocRebuildReport: ...
    @property
    def entry_count(self) -> int: ...
    @property
    def bookmark_count(self) -> int: ...
    @property
    def diagnostics(self) -> tuple[str, ...]: ...
    @property
    def diagnostic_count(self) -> int: ...


@_final
class Revision:
    def __new__(
        cls,
        *,
        id: int,
        author: str,
        timestamp: str | None,
        kind: str,
        story: Story | None = None,
    ) -> Revision: ...
    @property
    def id(self) -> int: ...
    @property
    def author(self) -> str: ...
    @property
    def timestamp(self) -> str | None: ...
    @property
    def kind(self) -> str: ...
    @property
    def story(self) -> Story | None: ...


@_final
class Story:
    def __new__(cls, *, kind: str, part_name: str, owner_index: int) -> Story: ...
    @property
    def kind(self) -> str: ...
    @property
    def part_name(self) -> str: ...
    @property
    def owner_index(self) -> int: ...


@_final
class StoryItem:
    def __new__(
        cls,
        *,
        story: Story,
        kind: str,
        index_path: tuple[int, ...],
        text: str | None,
        xml: bytes | None = None,
        direct_body_index: int | None = None,
        revision: int = 0,
    ) -> StoryItem: ...
    @property
    def story(self) -> Story: ...
    @property
    def kind(self) -> str: ...
    @property
    def index_path(self) -> tuple[int, ...]: ...
    @property
    def direct_body_index(self) -> int | None: ...
    @property
    def text(self) -> str | None: ...
    @property
    def xml(self) -> bytes: ...
    @property
    def revision(self) -> int: ...


@_final
class StoryRunPosition:
    @_overload
    def __new__(
        cls, *, item: StoryItem, run_index: int, paragraph: None = None
    ) -> StoryRunPosition: ...
    @_overload
    def __new__(
        cls, *, item: None = None, run_index: int, paragraph: Paragraph
    ) -> StoryRunPosition: ...
    @property
    def item(self) -> StoryItem: ...
    @property
    def run_index(self) -> int: ...


@_final
class StoryRunRange:
    def __new__(
        cls, *, start: StoryRunPosition, end: StoryRunPosition
    ) -> StoryRunRange: ...
    @property
    def start(self) -> StoryRunPosition: ...
    @property
    def end(self) -> StoryRunPosition: ...


@_final
class ContentFragment:
    def __new__(cls, *, _private: _Never) -> ContentFragment: ...
    @property
    def kind(
        self,
    ) -> _Literal["paragraph", "table", "content_control", "field", "drawing", "preserved_node"]: ...


@_final
class Picture:
    def __new__(cls, *, _private: _Never) -> Picture: ...
    @property
    def relationship_id(self) -> str: ...
    @property
    def name(self) -> str | None: ...
    @property
    def description(self) -> str | None: ...
    @property
    def title(self) -> str | None: ...
    @property
    def decorative(self) -> bool: ...
    @property
    def width(self) -> _shared.Length: ...
    @property
    def height(self) -> _shared.Length: ...
    @property
    def inline(self) -> bool: ...
    @property
    def content_type(self) -> str | None: ...
    @property
    def filename(self) -> str | None: ...
    @property
    def blob(self) -> bytes | None: ...


@_final
class Hyperlink:
    def __new__(
        cls,
        *,
        story: Story,
        index_path: tuple[int, ...],
        text: str,
        url: str | None,
        anchor: str | None,
        relationship_id: str | None,
        tooltip: str | None = None,
    ) -> Hyperlink: ...
    @property
    def story(self) -> Story: ...
    @property
    def index_path(self) -> tuple[int, ...]: ...
    @property
    def text(self) -> str: ...
    @property
    def url(self) -> str | None: ...
    @property
    def anchor(self) -> str | None: ...
    @property
    def relationship_id(self) -> str | None: ...
    @property
    def tooltip(self) -> str | None: ...


@_final
class HeaderFooterVariant:
    def __new__(
        cls,
        *,
        section_index: int,
        kind: str,
        variant: str,
        story: Story | None,
        source_section: int | None,
        inherited: bool,
    ) -> HeaderFooterVariant: ...
    @property
    def section_index(self) -> int: ...
    @property
    def kind(self) -> str: ...
    @property
    def variant(self) -> str: ...
    @property
    def story(self) -> Story | None: ...
    @property
    def source_section(self) -> int | None: ...
    @property
    def inherited(self) -> bool: ...


@_final
class Section:
    def __new__(
        cls,
        *,
        ordinal: int,
        is_final: bool,
        orientation: str | None,
        page_width: int | None,
        page_height: int | None,
        margin_top: int | None,
        margin_right: int | None,
        margin_bottom: int | None,
        margin_left: int | None,
        gutter: int | None,
        column_count: int | None,
        column_spacing: int | None,
        page_number_start: int | None,
        header_distance: int | None,
        footer_distance: int | None,
        different_first_page: bool | None,
        break_type: str | None,
    ) -> Section: ...
    @property
    def ordinal(self) -> int: ...
    @property
    def is_final(self) -> bool: ...
    @property
    def orientation(self) -> str | None: ...
    @property
    def page_width(self) -> int | None: ...
    @property
    def page_height(self) -> int | None: ...
    @property
    def margin_top(self) -> int | None: ...
    @property
    def margin_right(self) -> int | None: ...
    @property
    def margin_bottom(self) -> int | None: ...
    @property
    def margin_left(self) -> int | None: ...
    @property
    def gutter(self) -> int | None: ...
    @property
    def column_count(self) -> int | None: ...
    @property
    def column_spacing(self) -> int | None: ...
    @property
    def page_number_start(self) -> int | None: ...
    @property
    def header_distance(self) -> int | None: ...
    @property
    def footer_distance(self) -> int | None: ...
    @property
    def different_first_page(self) -> bool | None: ...
    @property
    def break_type(self) -> str | None: ...
    # The page geometry above is a snapshot taken when the section was read.
    # Change it with Document.update_section. The members below act on the
    # document a section read from Document.sections belongs to.
    @property
    def header(self) -> HeaderFooter: ...
    @property
    def footer(self) -> HeaderFooter: ...
    @property
    def first_page_header(self) -> HeaderFooter: ...
    @property
    def first_page_footer(self) -> HeaderFooter: ...
    @property
    def even_page_header(self) -> HeaderFooter: ...
    @property
    def even_page_footer(self) -> HeaderFooter: ...
    @property
    def different_first_page_header_footer(self) -> bool:
        """Whether the first page shows the first-page header and footer.

        Read from and written to the document, unlike the snapshot fields.
        """
    @different_first_page_header_footer.setter
    def different_first_page_header_footer(self, value: bool) -> None: ...


@_final
class HeaderFooter:
    """A section's header or footer, python-docx's ``section.header``.

    The handle names a section and a variant, so it stays valid across edits.
    Reading ``paragraphs`` or adding content gives the story a definition when
    no section up to this one has one, as python-docx does. Writing into a
    linked story edits the earlier section's story. Writing into a first-page
    story turns the section's different first page on, and writing into an
    even-page story turns ``Settings.odd_and_even_pages_header_footer`` on.
    """

    def __new__(cls, *, _private: _Never) -> HeaderFooter: ...
    @property
    def kind(self) -> _Literal["header", "footer"]: ...
    @property
    def variant(self) -> _Literal["default", "first", "even"]: ...
    @property
    def section_index(self) -> int: ...
    @property
    def is_linked_to_previous(self) -> bool: ...
    @is_linked_to_previous.setter
    def is_linked_to_previous(self, value: bool) -> None: ...
    @property
    def paragraphs(self) -> list[Paragraph]: ...
    def add_paragraph(self, text: str = "", style: str | None = None) -> Paragraph: ...
    def add_table(
        self, rows: int, cols: int, width: int | None = None
    ) -> HeaderFooterTable: ...
    @property
    def tables(self) -> list[HeaderFooterTable]: ...
    def add_page_number(
        self,
        template: str = "Page {PAGE} of {NUMPAGES}",
        *,
        alignment: _text.WD_ALIGN_PARAGRAPH | int | None = _text.WD_ALIGN_PARAGRAPH.CENTER,
    ) -> Paragraph:
        """Append a paragraph showing the page number, such as "Page 3 of 7".

        ``template`` mixes literal text with ``{PAGE}``, ``{NUMPAGES}`` and
        ``{SECTIONPAGES}`` fields that Word, Google Docs and rdocx fill in on
        every page.
        """


@_final
class HeaderFooterTable:
    def __new__(cls, *, _private: _Never) -> HeaderFooterTable: ...
    @property
    def row_count(self) -> int: ...
    @property
    def column_count(self) -> int: ...
    @property
    def rows(self) -> list[HeaderFooterRow]: ...
    @property
    def style(self) -> str | None: ...
    @style.setter
    def style(self, value: str) -> None: ...
    def cell(self, row: int, col: int) -> HeaderFooterCell: ...


@_final
class HeaderFooterRow:
    def __new__(cls, *, _private: _Never) -> HeaderFooterRow: ...
    @property
    def cells(self) -> list[HeaderFooterCell]: ...


@_final
class HeaderFooterCell:
    def __new__(cls, *, _private: _Never) -> HeaderFooterCell: ...
    @property
    def text(self) -> str: ...
    @text.setter
    def text(self, value: str) -> None: ...
    @property
    def paragraphs(self) -> list[Paragraph]: ...
    def add_paragraph(self, text: str = "") -> Paragraph: ...


@_final
class Style:
    def __new__(
        cls,
        *,
        style_id: str,
        name: str | None,
        based_on: str | None,
        style_type: str,
        linked_style: str | None,
        next_style: str | None,
        priority: int | None,
        auto_redefine: bool | None,
        hidden: bool | None,
        semi_hidden: bool | None,
        unhide_when_used: bool | None,
        quick_format: bool | None,
        locked: bool | None,
        is_default: bool,
    ) -> Style: ...
    @property
    def style_id(self) -> str: ...
    @property
    def name(self) -> str | None: ...
    @property
    def based_on(self) -> str | None: ...
    @property
    def style_type(self) -> str: ...
    @property
    def linked_style(self) -> str | None: ...
    @property
    def next_style(self) -> str | None: ...
    @property
    def priority(self) -> int | None: ...
    @property
    def auto_redefine(self) -> bool | None: ...
    @property
    def hidden(self) -> bool | None: ...
    @property
    def semi_hidden(self) -> bool | None: ...
    @property
    def unhide_when_used(self) -> bool | None: ...
    @property
    def quick_format(self) -> bool | None: ...
    @property
    def locked(self) -> bool | None: ...
    @property
    def is_default(self) -> bool: ...


@_final
class ListLevel:
    def __new__(
        cls,
        *,
        format: str = "decimal",
        text: str | None = None,
        start: int | None = None,
        left_indent: int | None = None,
        hanging_indent: int | None = None,
        font: str | None = None,
    ) -> ListLevel: ...
    @property
    def format(self) -> str: ...
    @property
    def text(self) -> str | None: ...
    @property
    def start(self) -> int | None: ...
    @property
    def left_indent(self) -> int | None: ...
    @property
    def hanging_indent(self) -> int | None: ...
    @property
    def font(self) -> str | None:
        """The marker font, written to every `w:rFonts` slot of the level."""
    @staticmethod
    def checklist(
        checked: bool = False,
        *,
        left_indent: int | None = None,
        hanging_indent: int | None = None,
    ) -> ListLevel:
        """A bullet level whose glyph is an empty box, or a checked box, in Segoe UI Symbol."""


@_final
class CoreProperties:
    def __new__(cls, *, _private: _Never) -> CoreProperties: ...
    @property
    def author(self) -> str: ...
    @author.setter
    def author(self, value: str | None) -> None: ...
    @property
    def category(self) -> str: ...
    @category.setter
    def category(self, value: str | None) -> None: ...
    @property
    def comments(self) -> str: ...
    @comments.setter
    def comments(self, value: str | None) -> None: ...
    @property
    def content_status(self) -> str: ...
    @content_status.setter
    def content_status(self, value: str | None) -> None: ...
    @property
    def created(self) -> _datetime.datetime | None: ...
    @created.setter
    def created(self, value: _datetime.datetime | None) -> None: ...
    @property
    def identifier(self) -> str: ...
    @identifier.setter
    def identifier(self, value: str | None) -> None: ...
    @property
    def keywords(self) -> str: ...
    @keywords.setter
    def keywords(self, value: str | None) -> None: ...
    @property
    def language(self) -> str: ...
    @language.setter
    def language(self, value: str | None) -> None: ...
    @property
    def last_modified_by(self) -> str: ...
    @last_modified_by.setter
    def last_modified_by(self, value: str | None) -> None: ...
    @property
    def last_printed(self) -> _datetime.datetime | None: ...
    @last_printed.setter
    def last_printed(self, value: _datetime.datetime | None) -> None: ...
    @property
    def modified(self) -> _datetime.datetime | None: ...
    @modified.setter
    def modified(self, value: _datetime.datetime | None) -> None: ...
    @property
    def revision(self) -> int: ...
    @revision.setter
    def revision(self, value: int | None) -> None: ...
    @property
    def subject(self) -> str: ...
    @subject.setter
    def subject(self, value: str | None) -> None: ...
    @property
    def title(self) -> str: ...
    @title.setter
    def title(self, value: str | None) -> None: ...
    @property
    def version(self) -> str: ...
    @version.setter
    def version(self, value: str | None) -> None: ...


# ---- Document-level views, templates, assembly, properties, content
# controls and validation.


@_final
class ValidationReport:
    """The findings of `Document.validate()`, the same as `rdocx validate`."""

    def __new__(cls, *, _private: _Never) -> ValidationReport: ...
    @property
    def errors(self) -> tuple[str, ...]:
        """Structural errors: Word may refuse or repair the file."""
    @property
    def warnings(self) -> tuple[str, ...]:
        """Advisory findings, such as empty paragraphs or a missing title."""
    @property
    def ok(self) -> bool:
        """`True` when there is no error. Warnings do not count."""


@_final
class DocumentFragment:
    """Story items copied out of a document with their styles, lists,
    pictures and other dependencies, from `Document.copy_fragment`."""

    def __new__(cls, *, _private: _Never) -> DocumentFragment: ...


@_final
class ContentControl:
    """One content control of the document body (headers and footers are
    not listed)."""

    def __new__(cls, *, _private: _Never) -> ContentControl: ...
    @property
    def tag(self) -> str | None: ...
    @property
    def alias(self) -> str | None:
        """The title Word shows on the control."""
    @property
    def id(self) -> int | None: ...
    @property
    def type(self) -> _ContentControlType:
        """A control without a type element is a rich text control."""
    @property
    def text(self) -> str: ...


@_final
class Settings:
    """The document settings, `word/settings.xml`."""

    def __new__(cls, *, _private: _Never) -> Settings: ...
    @property
    def track_revisions(self) -> bool:
        """Whether Word records edits as tracked changes ("Track Changes").

        Turning it on does not make rdocx edits tracked: rdocx does not author
        tracked insertions or deletions.
        """
    @track_revisions.setter
    def track_revisions(self, value: bool) -> None: ...
    @property
    def odd_and_even_pages_header_footer(self) -> bool: ...
    @odd_and_even_pages_header_footer.setter
    def odd_and_even_pages_header_footer(self, value: bool) -> None: ...


@_final
class CustomProperties(_MutableMapping[str, _CustomPropertyValue | bytes | None]):
    """The custom document properties, `docProps/custom.xml`, as a mapping.

    Values are typed: `str`, `int` (32 bits), `float`, `bool` and
    `datetime.datetime` (stored in UTC, a naive value is taken as UTC). A
    value of a type rdocx does not model reads as its XML `bytes`. Google
    Docs drops custom properties when it imports a file.
    """

    def __new__(cls, *, _private: _Never) -> CustomProperties: ...
    def __len__(self) -> int: ...
    def __contains__(self, key: object, /) -> bool: ...
    def __iter__(self) -> _Iterator[str]: ...
    def __getitem__(self, key: str, /) -> _CustomPropertyValue | bytes | None: ...
    def __setitem__(  # type: ignore[override]
        self, key: str, value: _CustomPropertyValue, /
    ) -> None: ...
    def __delitem__(self, key: str, /) -> None: ...
    def get(  # type: ignore[override]
        self, key: object, default: object = None
    ) -> _CustomPropertyValue | bytes | object | None: ...
    def keys(self) -> list[str]: ...  # type: ignore[override]
    def values(self) -> list[_CustomPropertyValue | bytes | None]: ...  # type: ignore[override]
    def items(  # type: ignore[override]
        self,
    ) -> list[tuple[str, _CustomPropertyValue | bytes | None]]: ...


@_final
class AppProperties:
    """The application properties, `docProps/app.xml`.

    Text properties read as an empty string when absent. Word writes the
    counts when it saves, so they are read-only and can be stale or `None`:
    `Document.word_count()`, `character_count()` and `page_count()` compute
    them.
    """

    def __new__(cls, *, _private: _Never) -> AppProperties: ...
    @property
    def company(self) -> str: ...
    @company.setter
    def company(self, value: str | None) -> None: ...
    @property
    def manager(self) -> str: ...
    @manager.setter
    def manager(self, value: str | None) -> None: ...
    @property
    def template(self) -> str: ...
    @template.setter
    def template(self, value: str | None) -> None: ...
    @property
    def application(self) -> str: ...
    @application.setter
    def application(self, value: str | None) -> None: ...
    @property
    def pages(self) -> int | None: ...
    @property
    def words(self) -> int | None: ...
    @property
    def characters(self) -> int | None: ...
    @property
    def characters_with_spaces(self) -> int | None: ...
    @property
    def lines(self) -> int | None: ...
    @property
    def paragraphs(self) -> int | None: ...


@_final
class Equation:
    """One equation of a paragraph, from `Paragraph.equations`."""

    def __new__(cls, *, _private: _Never) -> Equation: ...
    @property
    def latex(self) -> str: ...
    @property
    def mathml(self) -> str:
        """Presentation MathML, a `<math>` element."""
    @property
    def display(self) -> bool:
        """`True` for a display equation, on its own line."""
    @property
    def diagnostics(self) -> tuple[str, ...]:
        """What the LaTeX or MathML form could not represent."""


@_final
class Document:
    def __new__(cls, path: _Path | _IO[bytes] | None = None) -> Document:
        """Open a path or a binary file-like object, or start from the default template."""
    @staticmethod
    def open(path: _Path) -> Document: ...
    @staticmethod
    def from_bytes(bytes: bytes) -> Document: ...
    def save(self, path: _Path | _IO[bytes]) -> None:
        """Save to a path or write to a binary file-like object.

        A path whose extension names another format, such as ``.pdf``, raises
        ``ValueError`` naming the call that writes it, and so does a table
        without rows, which Word cannot open.
        """
    def to_bytes(self) -> bytes: ...
    def image_data(self, relationship_id: str) -> bytes | None: ...
    def replace_image(self, relationship_id: str, data: bytes) -> None: ...
    def replace_image_for_story(
        self, story: Story, relationship_id: str, data: bytes
    ) -> None: ...
    # A Picture resizes only itself, a relationship ID every picture showing it.
    def set_picture_size(
        self, picture: str | Picture, width: int, height: int
    ) -> int: ...
    @property
    def pictures(self) -> tuple[Picture, ...]: ...
    def split_run(
        self, body_index: int | Paragraph, run_index: int, character_offset: int
    ) -> int: ...
    def to_pdf(
        self,
        *,
        fonts: _Sequence[tuple[str, bytes]] | None = None,
        font_dir: _Path | None = None,
        revision_view: _RevisionView = "accepted",
    ) -> bytes:
        """Render the document to PDF bytes.

        ``fonts`` gives ``(family, font bytes)`` pairs and ``font_dir`` a
        directory whose ``.ttf``, ``.otf`` and ``.ttc`` files are named after
        their family. When both are given, the directory fonts follow the
        ``fonts`` pairs. Those fonts come before every other font source, as
        ``rdocx convert --font-dir`` does, and a family they do not provide
        resolves as without them. A missing ``font_dir`` raises
        ``FileNotFoundError`` and a file ``NotADirectoryError``. The PNG, image
        and SVG render methods, ``layout`` and ``layout_page`` take the same two
        arguments. ``to_pdfa_deterministic`` does not.
        """
    def to_pdfa_deterministic(
        self, profile: _Literal["pdfa-2b", "pdfa-3b"] = "pdfa-2b"
    ) -> bytes: ...
    def render_page_to_png(
        self,
        page_index: int,
        dpi: float = 150.0,
        *,
        revision_view: _RevisionView = "accepted",
        fonts: _Sequence[tuple[str, bytes]] | None = None,
        font_dir: _Path | None = None,
    ) -> bytes | None: ...
    def render_page_to_svg(
        self,
        page_index: int,
        *,
        fonts: _Sequence[tuple[str, bytes]] | None = None,
        font_dir: _Path | None = None,
    ) -> SvgRenderResult | None: ...
    def render_all_pages(
        self,
        dpi: float = 150.0,
        *,
        revision_view: _RevisionView = "accepted",
        fonts: _Sequence[tuple[str, bytes]] | None = None,
        font_dir: _Path | None = None,
    ) -> list[bytes]: ...
    def render_pages(
        self,
        *,
        dpi: float = 150.0,
        format: str = "png",
        quality: int = 90,
        transparent: bool = False,
        pages: list[int] | None = None,
        revision_view: _RevisionView = "accepted",
        fonts: _Sequence[tuple[str, bytes]] | None = None,
        font_dir: _Path | None = None,
    ) -> list[bytes] | bytes: ...
    def compare(
        self,
        edited: Document,
        author: str,
        timestamp: str,
        *,
        granularity: _Literal["run", "word", "character"] = "run",
        ignore_formatting: bool = False,
        ignore_whitespace: bool = False,
        ignore_fields: bool = False,
        ignore_comments: bool = False,
        ignored_stories: _Sequence[
            _Literal["body", "header", "footer", "comment", "text_box", "footnote", "endnote"]
        ] | None = None,
    ) -> tuple[ComparisonDiagnostic, ...]:
        """Record how ``edited`` differs from this document as tracked changes.

        ``granularity`` selects the unit of a text change. The default ``"run"``
        replaces a changed run whole, while ``"word"`` and ``"character"`` mark
        only the changed words or characters. ``ignored_stories`` takes
        ``Story.kind`` names. The ignore options and ``ignored_stories`` are
        left-biased: an ignored difference or story keeps this document's
        content. With ``ignore_comments=True`` the result keeps this document's
        comments and anchors, and the comments of ``edited`` are not carried
        over. An unknown option value raises ``RdocxError``.
        """
    @property
    def comments(self) -> tuple[Comment, ...]: ...
    @property
    def sections(self) -> tuple[Section, ...]: ...
    def update_section(
        self,
        index: int,
        *,
        orientation: _Literal["portrait", "landscape"] | None = None,
        page_width: int | None = None,
        page_height: int | None = None,
        margin_top: int | None = None,
        margin_right: int | None = None,
        margin_bottom: int | None = None,
        margin_left: int | None = None,
        gutter: int | None = None,
        column_count: int | None = None,
        column_spacing: int | None = None,
        page_number_start: int | None = None,
        header_distance: int | None = None,
        footer_distance: int | None = None,
        different_first_page: bool | None = None,
        break_type: _Literal[
            "nextPage", "continuous", "evenPage", "oddPage", "nextColumn"
        ] | None = None,
    ) -> Section:
        """Change one section's page setup and return its new snapshot.

        Lengths are EMU: pass a Length such as ``Inches(8.5)``, ``Pt(612)``,
        ``Twips(12240)`` or ``Cm(21)``. A bare int is read as EMU, so a page
        under a tenth of an inch, the mark of a twips or points value, raises
        ``ValueError``. Switching the orientation swaps the width and height
        to match it.
        """
    def insert_section(self, index: int) -> None: ...
    def remove_section(self, index: int) -> None: ...
    @property
    def styles(self) -> _shared._Styles:
        """The styles, a tuple also indexed by style name or ID: ``styles['Normal']``."""
    def add_section(self, start_type: int = 2) -> Section:
        """Append a section starting as ``start_type`` (a ``WD_SECTION`` member) says."""
    def section_xml(self, index: int) -> bytes:
        """Section ``index``'s ``w:sectPr`` as standalone XML."""
    def replace_section_xml(self, index: int, xml: _Xml) -> None:
        """Replace section ``index``'s ``w:sectPr``, checked as ``Paragraph.replace_xml`` checks."""
    def add_style(
        self,
        name: str,
        style_type: _Literal["paragraph", "character", "table"] = "paragraph",
        *,
        style_id: str | None = None,
        based_on: str | None = None,
        next_style: str | None = None,
        font_name: str | None = None,
        font_size: int | None = None,
        bold: bool | None = None,
        italic: bool | None = None,
        color: _Color | None = None,
        space_before: int | None = None,
        space_after: int | None = None,
        left_indent: int | None = None,
        right_indent: int | None = None,
        first_line_indent: int | None = None,
        underline: bool | _text.WD_UNDERLINE | None = None,
        alignment: _text.WD_ALIGN_PARAGRAPH | None = None,
        line_spacing: int | float | None = None,
        keep_with_next: bool | None = None,
        keep_together: bool | None = None,
        page_break_before: bool | None = None,
        shading: _Color | None = None,
        borders: _Mapping[_ParagraphBorderEdge, tuple[_BorderStyle, int, _Color]] | None = None,
        tab_stops: _Sequence[
            tuple[int, _text.WD_TAB_ALIGNMENT]
            | tuple[int, _text.WD_TAB_ALIGNMENT, _text.WD_TAB_LEADER]
        ] | None = None,
    ) -> Style:
        """Create a style as python-docx's `styles.add_style` does.

        Unless `style_id` is given, the ID keeps the ASCII letters, digits and
        hyphens of the name, as Word derives one, or is `a`, `a0` and so on
        when none is left.
        `based_on` and `next_style` take an ID or a name. Lengths and the font
        size are EMU. `line_spacing` is an exact `Length` or a float multiple
        of single spacing, as `ParagraphFormat.line_spacing` takes it.
        `shading` and the border colours follow the colour rule (`auto`
        accepted), `borders` maps an edge to
        `(style, size in eighths of a point, color)`, and `tab_stops`
        lists `(position, alignment[, leader])`. Raises `KeyError` when a
        base or next style names no style, and `ValueError` for a duplicate
        ID or name or any other invalid argument.
        """
    def set_style(
        self,
        style: str,
        *,
        based_on: str | None = None,
        next_style: str | None = None,
        font_name: str | None = None,
        font_size: int | None = None,
        bold: bool | None = None,
        italic: bool | None = None,
        color: _Color | None = None,
        space_before: int | None = None,
        space_after: int | None = None,
        left_indent: int | None = None,
        right_indent: int | None = None,
        first_line_indent: int | None = None,
        underline: bool | _text.WD_UNDERLINE | None = None,
        alignment: _text.WD_ALIGN_PARAGRAPH | None = None,
        line_spacing: int | float | None = None,
        keep_with_next: bool | None = None,
        keep_together: bool | None = None,
        page_break_before: bool | None = None,
        shading: _Color | None = None,
        borders: _Mapping[_ParagraphBorderEdge, tuple[_BorderStyle, int, _Color]] | None = None,
        tab_stops: _Sequence[
            tuple[int, _text.WD_TAB_ALIGNMENT]
            | tuple[int, _text.WD_TAB_ALIGNMENT, _text.WD_TAB_LEADER]
        ] | None = None,
    ) -> Style:
        """Update the supplied formatting on a style selected by ID or name.

        Unspecified properties keep their existing values, and `borders` or
        `tab_stops` replace the style's whole set. Lengths and font size are
        EMU. Invalid style references or graph changes leave the document
        unchanged.
        """
    def remove_style(self, style: str) -> bool: ...
    def set_default_style(self, style: str) -> None: ...
    def add_numbering_definition(self, levels: _Sequence[ListLevel]) -> int: ...
    def add_numbering_instance(
        self, definition_id: int, *, start: int | None = None, level: int = 0
    ) -> int:
        """Create a list instance of a numbering definition.

        Word continues the count across instances of one definition. `start`
        restarts this instance at that value with a `w:startOverride` on
        `level`. `restart_numbering` does this for a paragraph in one call.
        """
    def add_bullet_list_item(self, text: str, level: int = 0) -> Paragraph:
        """Append an item to the document's plain bullet list.

        It continues the last body paragraph on that list's definition, so a
        checklist or another custom list is never continued.
        """
    def add_numbered_list_item(
        self, text: str, level: int = 0, *, restart: bool = False
    ) -> Paragraph:
        """Append an item to the document's plain decimal list.

        It continues the last body paragraph on that list's definition, so a
        numbered heading or another custom list is never continued.
        `restart=True` starts a new count at 1 from this item.
        """
    def restart_numbering(self, paragraph: Paragraph, start: int = 1) -> int:
        """Restart the list at a paragraph, as Word's "Restart at 1" does.

        The paragraph and every later paragraph of its list, in the body or a
        table cell, move to a new list instance whose level starts at `start`.
        Returns its ID.
        """
    @property
    def default_font_name(self) -> str | None:
        """The font name in `w:docDefaults`, which styles without one use."""
    @default_font_name.setter
    def default_font_name(self, value: str | None) -> None: ...
    @property
    def default_font_size(self) -> _shared.Length | None:
        """The font size in `w:docDefaults`, which styles without one use."""
    @default_font_size.setter
    def default_font_size(self, value: int | None) -> None: ...
    def link_style_to_numbering(self, style: str, num_id: int, level: int) -> None: ...
    @property
    def stories(self) -> tuple[Story, ...]: ...
    @property
    def story_items(self) -> tuple[StoryItem, ...]: ...
    @property
    def header_footer_variants(self) -> tuple[HeaderFooterVariant, ...]: ...
    def create_section_story(
        self,
        section_index: int,
        kind: _Literal["header", "footer"],
        variant: _Literal["default", "first", "even"],
    ) -> Story: ...
    def link_section_story(
        self,
        section_index: int,
        kind: _Literal["header", "footer"],
        variant: _Literal["default", "first", "even"],
        story: Story,
    ) -> Story: ...
    def unlink_section_story(
        self,
        section_index: int,
        kind: _Literal["header", "footer"],
        variant: _Literal["default", "first", "even"],
    ) -> Story: ...
    @property
    def hyperlinks(self) -> tuple[Hyperlink, ...]: ...
    @property
    def core_properties(self) -> CoreProperties: ...
    @property
    def update_fields_on_open(self) -> bool | None: ...
    @update_fields_on_open.setter
    def update_fields_on_open(self, value: bool | None) -> None: ...
    # ---- Document-level views, templates, assembly, properties, content
    # controls and validation.
    def text(self) -> str:
        """The plain text of every story, as `rdocx text` prints it.

        The body comes first, then each text box, header, footer, footnote,
        endnote and comment part with text under a `--- kind (part) ---`
        line. Warns with `ConversionWarning` and returns the body only when a
        story part cannot be read.
        """
    def to_markdown(self) -> str:
        """The Markdown that `rdocx convert --to md` writes.

        Headings, lists, tables, links, bold and italic in a CommonMark
        subset, then the text boxes, headers, footers and notes after a `---`
        rule. Comments are left out.
        """
    def to_html(self) -> str:
        """The HTML document that `rdocx convert --to html` writes."""
    def to_odt(self) -> bytes:
        """An OpenDocument text file. Content ODT cannot hold emits a
        `ConversionWarning`."""
    def to_rtf(self) -> bytes:
        """An RTF file. Content RTF cannot hold emits a `ConversionWarning`."""
    def to_epub(self) -> bytes:
        """An EPUB 3 publication. Content EPUB cannot hold emits a
        `ConversionWarning`."""
    def word_count(self) -> int:
        """Whitespace-separated words of the body and its tables, tracked
        deletions left out."""
    def character_count(self, *, include_spaces: bool = True) -> int:
        """Characters of the paragraphs `word_count()` reads, as Word counts
        them with or without spaces."""
    def page_count(self) -> int:
        """The number of pages of the rdocx layout."""
    def validate(self) -> ValidationReport:
        """Validate the document as it would be saved now, with the checks
        of `rdocx validate`."""
    @staticmethod
    def validate_file(path: _Path) -> ValidationReport:
        """Validate a file exactly as `rdocx validate` does, even a file
        that does not open."""
    def render_template(self, data: _Mapping[str, _Any]) -> int:
        """Render `{{ path.to.value }}` tags and `{% for item in path %}` /
        `{% if path %}` blocks from `data` and return the number of tags.

        A tag may cross run boundaries and keeps the formatting of its first
        run. Values are `str`, `int`, `float`, `bool`, `None` (empty text),
        `dict` and `list`, rendered as JSON writes them: `True` gives `true`
        and `2.0` gives `2.0`, so pass a formatted `str` for any other form.
        Data deeper than 256 levels or containing itself raises `ValueError`. A `{% ... %}` marker must be alone in its body
        paragraph or table row, closed by `{% endfor %}` or `{% endif %}`.
        A missing path or a malformed tag raises `RdocxError` naming the tag
        and its line, and leaves the document unchanged.
        """
    def insert_document(
        self,
        other: Document,
        at: int | StoryItem | Story | None = None,
        *,
        conflict: _ConflictPolicy = "reuse_equivalent",
    ) -> None:
        """Insert the body of `other` before body index `at` (the end of the
        body by default, a negative index counts from the end), with its
        styles, lists, pictures and links.

        `conflict="reuse_equivalent"` reuses a style, list or media part
        identical to one already here and renames the others,
        `conflict="rename"` renames every one. The page setup of this
        document is kept, and the headers and footers of `other` are not
        copied.
        """
    def copy_fragment(
        self,
        start: int | StoryItem,
        end: int | StoryItem | Story | None = None,
    ) -> DocumentFragment:
        """Copy the story items from `start` up to `end` excluded (the end of
        the story by default) with their dependencies. Negative body indexes
        count from the end."""
    def import_fragment(
        self,
        fragment: DocumentFragment,
        at: int | StoryItem | Story | None = None,
        *,
        conflict: _ConflictPolicy = "reuse_equivalent",
    ) -> None:
        """Insert a fragment of another document, as `insert_document` does."""
    @property
    def custom_properties(self) -> CustomProperties: ...
    @property
    def app_properties(self) -> AppProperties: ...
    @property
    def settings(self) -> Settings: ...
    @property
    def content_controls(self) -> tuple[ContentControl, ...]: ...
    def set_content_control_value(
        self, value: str, *, tag: str | None = None, alias: str | None = None
    ) -> int:
        """Set the text of every body content control with this tag, or this
        alias, and of the custom XML it is bound to. Returns the count.

        A checkbox takes ``"true"`` or ``"false"`` (also ``"1"``/``"0"``,
        ``"yes"``/``"no"``) and shows its checked or unchecked glyph.

        Raises `KeyError` when no control matches and `ValueError` for a
        picture or group control, which holds no text value, or for a value
        the control does not take.
        """
    def add_comment(
        self,
        range: RunRange | StoryRunRange,
        *,
        author: str,
        text: str,
        initials: str | None = None,
        date: str | None = None,
    ) -> int: ...
    def add_comment_on_text(
        self,
        anchor: str,
        *,
        author: str,
        text: str,
        occurrence: int = 0,
        initials: str | None = None,
        date: str | None = None,
    ) -> int: ...
    def move_comment(self, id: int, range: StoryRunRange) -> None:
        """Move an existing root thread to a checked current story range."""
        ...
    def move_comment_to_text(self, id: int, anchor: str, *, occurrence: int = 0) -> None:
        """Move an existing root thread onto a literal main-story occurrence."""
        ...
    def reply_to(
        self, parent_id: int, *, author: str, text: str, date: str | None = None
    ) -> int: ...
    def resolve_comment(self, id: int, *, resolved: bool = True) -> bool: ...
    def remove_comment(self, id: int) -> bool: ...
    @property
    def bookmarks(self) -> tuple[Bookmark, ...]: ...
    def add_bookmark(self, name: str, range: RunRange) -> int: ...
    def layout(
        self,
        *,
        fonts: _Sequence[tuple[str, bytes]] | None = None,
        font_dir: _Path | None = None,
    ) -> tuple[LayoutFragment, ...]: ...
    def layout_page(
        self,
        page_index: int,
        *,
        fonts: _Sequence[tuple[str, bytes]] | None = None,
        font_dir: _Path | None = None,
    ) -> LayoutPage | None: ...
    def rebuild_toc(self) -> TocRebuildReport: ...
    def replace_text_at(
        self, item: StoryItem, old: str, new: str, *, expect: int | None = None
    ) -> int: ...
    def try_replace_text(
        self, placeholder: str, replacement: str, *, expect: int | None = None
    ) -> int: ...
    def replace_all(
        self, pairs: _Sequence[tuple[str, str] | tuple[str, str, int | None]]
    ) -> tuple[int, ...]: ...
    def replace_all_regex(self, patterns: list[tuple[str, str]]) -> int: ...
    @property
    def revisions(self) -> tuple[Revision, ...]: ...
    def accept_all(self) -> int: ...
    def reject_all(self) -> int: ...
    def accept_revisions_by_author(self, author: str) -> int: ...
    def reject_revisions_by_author(self, author: str) -> int: ...
    def accept_revisions_in_date_range(self, *, start: str, end: str) -> int: ...
    def reject_revisions_in_date_range(self, *, start: str, end: str) -> int: ...
    def accept_revision_id(self, id: int) -> int: ...
    def reject_revision_id(self, id: int) -> int: ...
    def update_fields(
        self,
        *,
        now: _datetime.datetime | None = None,
        file_name: str | None = None,
        file_path: str | None = None,
        merge_fields: dict[str, str] | None = None,
        included_text: dict[str, str] | None = None,
        merge_record_number: int | None = None,
        merge_sequence_number: int | None = None,
    ) -> int: ...
    def set_header(self, text: str) -> None: ...
    def set_footer(self, text: str) -> None: ...
    def set_story_text(self, item: StoryItem, text: str) -> None: ...
    def add_hyperlink_to_story(self, story: Story, text: str, url: str) -> None: ...
    def set_hyperlink_url(self, hyperlink: Hyperlink, url: str) -> None: ...
    def remove_hyperlink(self, hyperlink: Hyperlink) -> None: ...
    def add_picture(
        self,
        image: _Image,
        filename: str | int | None = None,
        width: int | None = None,
        height: int | None = None,
        *,
        after: StoryItem | None = None,
        description: str | None = None,
        title: str | None = None,
        decorative: bool = False,
        name: str | None = None,
        crop: tuple[float, float, float, float] | None = None,
        wrap: _PictureWrap = "inline",
        position: tuple[int | _HorizontalAlign, int | _VerticalAlign] | None = None,
        relative_to: tuple[_HorizontalFrom, _VerticalFrom] | None = None,
    ) -> StoryItem: ...
    def update_page_fields(self) -> int: ...
    def update_layout_backed_fields(self) -> LayoutBackedFieldUpdateReport: ...
    def insert_toc(self, index: int, max_level: int = 3) -> None: ...
    @property
    def paragraphs(self) -> ParagraphCollection: ...
    @property
    def tables(self) -> TableCollection: ...
    def add_paragraph(self, text: str = "", style: str | Style | None = None) -> Paragraph:
        """Append a paragraph. Every held handle stays valid.

        ``style`` is a style name or ID. A python-docx list style such as
        ``'List Bullet'`` or ``'List Number 2'`` that the document lacks is
        added, linked to a list definition its siblings share.
        """
    def add_heading(self, text: str = "", level: int = 1) -> Paragraph:
        """Append a heading: level 0 is the Title style, 1 to 9 the Heading styles."""
    def add_page_break(self) -> Paragraph:
        """Append a paragraph holding only a page break."""
    def add_table(self, rows: int, cols: int, style: str | None = None) -> Table: ...
    def remove_content(self, index: int) -> bool: ...
    @_overload
    def find_content_index(self, content: Paragraph | Table) -> int: ...
    @_overload
    def find_content_index(self, content: str) -> int: ...
    def find_content_indices(self, text: str) -> tuple[int, ...]: ...
    def insert_paragraph(self, index: int, text: str) -> Paragraph: ...
    def insert_table(self, index: int, rows: int, cols: int) -> Table: ...
    def pop_content(self, index: int | StoryItem) -> ContentFragment: ...
    def insert_content(
        self, destination: int | StoryItem | Story, fragment: ContentFragment
    ) -> None: ...
    def clone_content(self, source: Paragraph | Table, destination: int) -> None: ...
    def move_content(self, source: Paragraph | Table, destination: int) -> None: ...
    def add_footnote(self, target: Paragraph | Run, text: str) -> int:
        """Add a footnote whose reference ends a body paragraph and return its ID.

        ``target`` is the paragraph, or its last run.
        """
    def add_endnote(self, target: Paragraph | Run, text: str) -> int: ...
    def remove_footnote(self, id: int) -> None: ...
    def remove_endnote(self, id: int) -> None: ...
    def set_note_numbering(
        self,
        kind: _Literal["footnote", "endnote"],
        *,
        number_format: _Literal[
            "decimal", "upperRoman", "lowerRoman", "upperLetter", "lowerLetter"
        ] = "decimal",
        start: int = 1,
        restart: _Literal["continuous", "eachSect", "eachPage"] = "continuous",
        placement: _Literal["pageBottom", "beneathText", "sectEnd", "docEnd"] | None = None,
        section: int | None = None,
    ) -> None: ...
    @property
    def page_color(self) -> _shared.RGBColor | None: ...
    @page_color.setter
    def page_color(self, value: _shared.RGBColor | str | None) -> None: ...
    def set_text_watermark(self, text: str) -> None: ...
    def set_image_watermark(
        self, data: bytes, filename: str, width: int, height: int
    ) -> None: ...
    def set_page_borders(
        self,
        section: int,
        style: str | None = "single",
        *,
        width: int = 6350,
        color: _shared.RGBColor | str | None = None,
        space: int = 304800,
        offset_from: _Literal["page", "text"] = "page",
    ) -> None: ...


@_final
class Paragraph:
    def __new__(cls, *, _private: _Never) -> Paragraph: ...
    def replace_text(
        self, old: str, new: str, *, expect: int | None = None
    ) -> int: ...
    @property
    def text(self) -> str: ...
    @text.setter
    def text(self, value: str | None) -> None: ...
    @property
    def runs(self) -> RunCollection: ...
    @property
    def alignment(self) -> _text.WD_ALIGN_PARAGRAPH | None: ...
    @alignment.setter
    def alignment(self, value: _text.WD_ALIGN_PARAGRAPH | None) -> None: ...
    def add_run(self, text: str) -> Run: ...
    # url for a web link, anchor for a bookmark name or a body heading
    # paragraph, which gets a bookmark when it has none. A url starting with
    # "#" names a bookmark.
    def add_hyperlink(
        self,
        text: str,
        url: str | None = None,
        *,
        anchor: str | Paragraph | None = None,
        tooltip: str | None = None,
    ) -> Run: ...
    @property
    def xml(self) -> bytes:
        """This paragraph's ``w:p`` as standalone XML."""
    def replace_xml(self, xml: _Xml) -> None:
        """Replace this paragraph with one ``w:p`` given as XML.

        Prefixes the document root declares need no declaration, and CDATA is
        read as the text it holds. Raises ``ValueError``, leaving the document
        unchanged, when the XML is malformed or has a DOCTYPE, holds another
        element or several, names an element rdocx would drop, places a ``w:``
        element where Word refuses it or uses a namespace mc:Ignorable does
        not cover, leaves a cell without a paragraph or a row without a cell,
        references a relationship id the document part lacks or one of the
        wrong type, or adds or removes a section break. This handle stays
        valid and its run handles retire.
        """
    def insert_paragraph_before(self, text: str = "", style: str | None = None) -> Paragraph:
        """Insert a body paragraph before this one. Every held handle stays valid."""
    @property
    def paragraph_format(self) -> ParagraphFormat: ...
    @property
    def style(self) -> str | None: ...
    @style.setter
    def style(self, value: str | None) -> None:
        """Assign a paragraph style by ID, or by name as python-docx does.

        The ID is tried first, then the exact name, then the name regardless of
        case. Raises `KeyError` when no style has that ID or name, and
        `ValueError` when only a character, table or numbering style does.
        `None` removes the paragraph style.
        """
    @property
    def numbering(self) -> tuple[int, int] | None: ...
    @numbering.setter
    def numbering(self, value: tuple[int, int] | None) -> None: ...
    # ---- Equations.
    def add_equation(
        self, latex: str | None = None, *, mathml: str | None = None, display: bool = False
    ) -> None:
        """Append an equation after the last run, from LaTeX or from MathML
        (a `<math>` element). `display=True` writes a display equation.

        Raises `ValueError` when the source does not parse or when the
        equation would lose part of it.
        """
    @property
    def equations(self) -> tuple[Equation, ...]: ...


@_final
class ParagraphCollection:
    def __new__(cls, *, _private: _Never) -> ParagraphCollection: ...
    def __len__(self) -> int: ...
    @_overload
    def __getitem__(self, key: int, /) -> Paragraph: ...
    @_overload
    def __getitem__(self, key: slice, /) -> list[Paragraph]: ...
    def __iter__(self) -> _Iterator[Paragraph]: ...


@_final
class Run:
    def __new__(cls, *, _private: _Never) -> Run: ...
    def add_picture(
        self,
        image: _Image,
        width: int | None = None,
        height: int | None = None,
        *,
        filename: str | None = None,
        description: str | None = None,
        title: str | None = None,
        decorative: bool = False,
        name: str | None = None,
        crop: tuple[float, float, float, float] | None = None,
        wrap: _PictureWrap = "inline",
        position: tuple[int | _HorizontalAlign, int | _VerticalAlign] | None = None,
        relative_to: tuple[_HorizontalFrom, _VerticalFrom] | None = None,
    ) -> Picture: ...
    @property
    def text(self) -> str: ...
    @text.setter
    def text(self, value: str) -> None: ...
    def remove(self) -> None: ...
    @property
    def bold(self) -> bool | None: ...
    @bold.setter
    def bold(self, value: bool | None) -> None: ...
    @property
    def italic(self) -> bool | None: ...
    @italic.setter
    def italic(self, value: bool | None) -> None: ...
    @property
    def underline(self) -> bool | _text.WD_UNDERLINE | None: ...
    @underline.setter
    def underline(self, value: bool | _text.WD_UNDERLINE | None) -> None: ...
    @property
    def xml(self) -> bytes:
        """This run's ``w:r`` as standalone XML."""
    def replace_xml(self, xml: _Xml) -> None:
        """Replace this run with one ``w:r``, checked as ``Paragraph.replace_xml`` checks."""
    @property
    def font(self) -> Font: ...
    @property
    def style_id(self) -> str | None: ...
    @style_id.setter
    def style_id(self, value: str | None) -> None: ...
    def add_tab(self) -> None: ...
    def add_field(self, instruction: str, cached_result: str = "") -> None: ...
    def add_break(self, break_type: _text.WD_BREAK | int = _text.WD_BREAK.LINE) -> None: ...


@_final
class RunCollection:
    def __new__(cls, *, _private: _Never) -> RunCollection: ...
    def __len__(self) -> int: ...
    @_overload
    def __getitem__(self, key: int, /) -> Run: ...
    @_overload
    def __getitem__(self, key: slice, /) -> list[Run]: ...
    def __iter__(self) -> _Iterator[Run]: ...


@_final
class Font:
    def __new__(cls, *, _private: _Never) -> Font: ...
    @property
    def name(self) -> str | None: ...
    @name.setter
    def name(self, value: str | None) -> None: ...
    @property
    def size(self) -> _shared.Length | None: ...
    @size.setter
    def size(self, value: int | None) -> None: ...
    @property
    def color(self) -> ColorFormat: ...
    @color.setter
    def color(self, value: _Color | None) -> None: ...
    @property
    def bold(self) -> bool | None: ...
    @bold.setter
    def bold(self, value: bool | None) -> None: ...
    @property
    def italic(self) -> bool | None: ...
    @italic.setter
    def italic(self, value: bool | None) -> None: ...
    @property
    def underline(self) -> bool | _text.WD_UNDERLINE | None: ...
    @underline.setter
    def underline(self, value: bool | _text.WD_UNDERLINE | None) -> None: ...
    @property
    def strike(self) -> bool | None: ...
    @strike.setter
    def strike(self, value: bool | None) -> None: ...
    @property
    def highlight(self) -> _Literal[
        "black", "blue", "cyan", "darkBlue", "darkCyan", "darkGray", "darkGreen",
        "darkMagenta", "darkRed", "darkYellow", "green", "lightGray", "magenta",
        "none", "red", "white", "yellow",
    ] | None: ...
    @highlight.setter
    def highlight(self, value: _Literal[
        "black", "blue", "cyan", "darkBlue", "darkCyan", "darkGray", "darkGreen",
        "darkMagenta", "darkRed", "darkYellow", "green", "lightGray", "magenta",
        "none", "red", "white", "yellow",
    ] | None) -> None: ...
    @property
    def shading(self) -> _shared.RGBColor | None: ...
    @shading.setter
    def shading(self, value: _Color | None) -> None: ...
    @property
    def highlight_color(self) -> _text.WD_COLOR_INDEX | None: ...
    @highlight_color.setter
    def highlight_color(self, value: _text.WD_COLOR_INDEX | None) -> None: ...
    @property
    def superscript(self) -> bool | None: ...
    @superscript.setter
    def superscript(self, value: bool | None) -> None: ...
    @property
    def subscript(self) -> bool | None: ...
    @subscript.setter
    def subscript(self, value: bool | None) -> None: ...
    @property
    def all_caps(self) -> bool | None: ...
    @all_caps.setter
    def all_caps(self, value: bool | None) -> None: ...
    @property
    def small_caps(self) -> bool | None: ...
    @small_caps.setter
    def small_caps(self, value: bool | None) -> None: ...
    @property
    def double_strike(self) -> bool | None: ...
    @double_strike.setter
    def double_strike(self, value: bool | None) -> None: ...
    @property
    def hidden(self) -> bool | None: ...
    @hidden.setter
    def hidden(self, value: bool | None) -> None: ...
    @property
    def character_spacing(self) -> _shared.Length | None:
        """Space added between characters, negative to condense them."""
    @character_spacing.setter
    def character_spacing(self, value: int | None) -> None: ...
    @property
    def language(self) -> str | None: ...
    @language.setter
    def language(self, value: str | None) -> None: ...
    @property
    def east_asian_language(self) -> str | None: ...
    @east_asian_language.setter
    def east_asian_language(self, value: str | None) -> None: ...
    @property
    def complex_script_language(self) -> str | None: ...
    @complex_script_language.setter
    def complex_script_language(self, value: str | None) -> None: ...
    @property
    def east_asian_name(self) -> str | None:
        """The `w:eastAsia` font, which `name` also sets."""
    @east_asian_name.setter
    def east_asian_name(self, value: str | None) -> None: ...
    @property
    def complex_script_name(self) -> str | None:
        """The `w:cs` font, which `name` also sets."""
    @complex_script_name.setter
    def complex_script_name(self, value: str | None) -> None: ...
    @property
    def rtl(self) -> bool | None: ...
    @rtl.setter
    def rtl(self, value: bool | None) -> None: ...


@_final
class ColorFormat:
    def __new__(cls, *, _private: _Never) -> ColorFormat: ...
    @property
    def rgb(self) -> _shared.RGBColor | None: ...
    @rgb.setter
    def rgb(self, value: _Color | None) -> None: ...
    @property
    def type(self) -> _dml.MSO_COLOR_TYPE | None: ...
    @property
    def theme_color(self) -> _dml.MSO_THEME_COLOR_INDEX | None: ...
    @theme_color.setter
    def theme_color(self, value: _dml.MSO_THEME_COLOR_INDEX | None) -> None: ...


@_final
class TabStop:
    """A live tab stop, found by its position, as in python-docx."""

    def __new__(cls, *, _private: _Never) -> TabStop: ...
    @property
    def position(self) -> _shared.Length: ...
    @position.setter
    def position(self, value: int) -> None:
        """Move the tab stop, keeping position order. A taken position raises."""
    @property
    def alignment(self) -> _text.WD_TAB_ALIGNMENT | None:
        """`None` for a bar, clear or num tab, which rdocx does not model."""
    @alignment.setter
    def alignment(self, value: _text.WD_TAB_ALIGNMENT | int) -> None: ...
    @property
    def leader(self) -> _text.WD_TAB_LEADER: ...
    @leader.setter
    def leader(self, value: _text.WD_TAB_LEADER | int) -> None: ...


@_final
class TabStops:
    def __new__(cls, *, _private: _Never) -> TabStops: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: int, /) -> TabStop: ...
    def __delitem__(self, index: int, /) -> None: ...
    def __iter__(self) -> _Iterator[TabStop]: ...
    def add_tab_stop(
        self,
        position: int,
        alignment: _text.WD_TAB_ALIGNMENT | int = 0,
        leader: _text.WD_TAB_LEADER | int = 0,
    ) -> TabStop:
        """Add a tab stop in position order, as python-docx does.

        Unlike python-docx, a second tab stop at one position raises
        `ValueError`: change the existing one through `tab_stops[i]`.
        """
    def clear_all(self) -> None: ...


@_final
class ParagraphFormat:
    def __new__(cls, *, _private: _Never) -> ParagraphFormat: ...
    @property
    def alignment(self) -> _text.WD_ALIGN_PARAGRAPH | None: ...
    @alignment.setter
    def alignment(self, value: _text.WD_ALIGN_PARAGRAPH | None) -> None: ...
    @property
    def space_before(self) -> _shared.Length | None: ...
    @space_before.setter
    def space_before(self, value: int | None) -> None: ...
    @property
    def space_after(self) -> _shared.Length | None: ...
    @space_after.setter
    def space_after(self, value: int | None) -> None: ...
    @property
    def left_indent(self) -> _shared.Length | None: ...
    @left_indent.setter
    def left_indent(self, value: int | None) -> None: ...
    @property
    def right_indent(self) -> _shared.Length | None: ...
    @right_indent.setter
    def right_indent(self, value: int | None) -> None: ...
    @property
    def first_line_indent(self) -> _shared.Length | None: ...
    @first_line_indent.setter
    def first_line_indent(self, value: int | None) -> None: ...
    @property
    def line_spacing(self) -> _shared.Length | float | None: ...
    @line_spacing.setter
    def line_spacing(self, value: int | float | None) -> None: ...
    @property
    def keep_with_next(self) -> bool | None: ...
    @keep_with_next.setter
    def keep_with_next(self, value: bool | None) -> None: ...
    @property
    def keep_together(self) -> bool | None: ...
    @keep_together.setter
    def keep_together(self, value: bool | None) -> None: ...
    @property
    def page_break_before(self) -> bool | None: ...
    @page_break_before.setter
    def page_break_before(self, value: bool | None) -> None: ...
    @property
    def widow_control(self) -> bool | None: ...
    @widow_control.setter
    def widow_control(self, value: bool | None) -> None: ...
    @property
    def tab_stops(self) -> TabStops: ...
    @property
    def outline_level(self) -> int | None:
        """The `w:outlineLvl`, 0 for a top-level heading to 9 for body text."""
    @outline_level.setter
    def outline_level(self, value: int | None) -> None: ...
    @property
    def right_to_left(self) -> bool | None:
        """The `w:bidi` paragraph direction."""
    @right_to_left.setter
    def right_to_left(self, value: bool | None) -> None: ...
    @property
    def shading(self) -> _shared.RGBColor | None:
        """The direct shading fill, `None` when absent or `auto`."""
    @shading.setter
    def shading(self, value: _Color | None) -> None: ...
    def border(
        self, edge: _ParagraphBorderEdge
    ) -> tuple[str, int | None, _shared.RGBColor | None] | None: ...
    def set_border(
        self,
        edge: _ParagraphBorderEdge,
        style: _BorderStyle = "single",
        *,
        size: int = 4,
        color: _Color | None = None,
    ) -> None:
        """Set one edge, `size` in eighths of a point, `color` by the colour rule, `None` for `auto`."""
    def remove_border(self, edge: _ParagraphBorderEdge) -> None: ...
    def clear_borders(self) -> None: ...


@_final
class Table:
    def __new__(cls, *, _private: _Never) -> Table: ...
    @property
    def rows(self) -> RowCollection: ...
    def cell(self, row: int, col: int) -> Cell: ...
    @property
    def xml(self) -> bytes:
        """This table's ``w:tbl`` as standalone XML."""
    def replace_xml(self, xml: _Xml) -> None:
        """Replace this table with one ``w:tbl``, checked as ``Paragraph.replace_xml`` checks."""
    @property
    def style(self) -> str | None: ...
    @style.setter
    def style(self, value: str) -> None: ...
    @property
    def alignment(self) -> _table.WD_TABLE_ALIGNMENT | None: ...
    @alignment.setter
    def alignment(self, value: _table.WD_TABLE_ALIGNMENT) -> None: ...
    @property
    def width(self) -> _shared.Length | None: ...
    @width.setter
    def width(self, value: int) -> None: ...
    @property
    def indent(self) -> _shared.Length | None: ...
    @indent.setter
    def indent(self, value: int | None) -> None: ...
    # False writes a fixed layout, True an autofit one.
    @property
    def autofit(self) -> bool: ...
    @autofit.setter
    def autofit(self, value: bool) -> None: ...
    # The table-style regions w:tblLook selects, with python-pptx names.
    @property
    def first_row(self) -> bool: ...
    @first_row.setter
    def first_row(self, value: bool) -> None: ...
    @property
    def last_row(self) -> bool: ...
    @last_row.setter
    def last_row(self, value: bool) -> None: ...
    @property
    def first_col(self) -> bool: ...
    @first_col.setter
    def first_col(self, value: bool) -> None: ...
    @property
    def last_col(self) -> bool: ...
    @last_col.setter
    def last_col(self, value: bool) -> None: ...
    @property
    def horz_banding(self) -> bool: ...
    @horz_banding.setter
    def horz_banding(self, value: bool) -> None: ...
    @property
    def vert_banding(self) -> bool: ...
    @vert_banding.setter
    def vert_banding(self, value: bool) -> None: ...
    def set_borders(self, style: _BorderStyle, *, size: int, color: _Color) -> None: ...
    def set_border(
        self, edge: _BorderEdge, style: _BorderStyle, *, size: int, color: _Color
    ) -> None: ...
    def border(
        self, edge: _BorderEdge
    ) -> tuple[str, int | None, _shared.RGBColor | None] | None: ...
    @property
    def cell_margins(self) -> _Margins | None: ...
    def set_cell_margins(self, *, top: int, right: int, bottom: int, left: int) -> None: ...
    @property
    def grid_widths(self) -> tuple[_shared.Length, ...]: ...
    @grid_widths.setter
    def grid_widths(self, value: _Sequence[int]) -> None: ...
    def set_column_width(self, column: int, width: int) -> None: ...
    def set_cell_grid_span(self, row: int, col: int, span: int | None) -> None: ...
    def set_cell_vertical_merge(
        self, row: int, col: int, merge: _Literal["restart", "continue"] | None
    ) -> None: ...
    # add_row, add_column and cell text edits keep this table handle and the
    # row and cell handles they cannot move valid.
    def add_row(self) -> Row: ...
    def add_column(self, width: int | None = None) -> Column: ...
    @property
    def columns(self) -> list[Column]: ...
    def insert_column(self, index: int, width: int | None = None) -> None: ...
    # Top-level tables only, as clone_row and remove_row.
    def remove_column(self, index: int) -> None: ...
    def clone_row(self, index: int, at: int | None = None) -> Row: ...
    def remove_row(self, index: int) -> None: ...


@_final
class Column:
    def __new__(cls, *, _private: _Never) -> Column: ...
    @property
    def index(self) -> int: ...
    @property
    def width(self) -> _shared.Length | None: ...
    @width.setter
    def width(self, value: int) -> None: ...
    @property
    def cells(self) -> list[Cell]: ...


@_final
class TableCollection:
    def __new__(cls, *, _private: _Never) -> TableCollection: ...
    def __len__(self) -> int: ...
    @_overload
    def __getitem__(self, key: int, /) -> Table: ...
    @_overload
    def __getitem__(self, key: slice, /) -> list[Table]: ...
    def __iter__(self) -> _Iterator[Table]: ...


@_final
class Row:
    def __new__(cls, *, _private: _Never) -> Row: ...
    @property
    def cells(self) -> CellCollection: ...
    # height and height_rule read None for a w:trHeight with an auto rule,
    # which python-docx reads as its value and AUTO, or with no value.
    # Assigning a height keeps the rule of an exact height and writes a
    # minimum otherwise, so an auto rule, or an exact rule without a value,
    # becomes a minimum.
    @property
    def height(self) -> _shared.Length | None: ...
    @height.setter
    def height(self, value: int) -> None: ...
    @property
    def height_rule(self) -> _table.WD_ROW_HEIGHT_RULE | None: ...
    @height_rule.setter
    def height_rule(self, value: _table.WD_ROW_HEIGHT_RULE) -> None: ...
    @property
    def cant_split(self) -> bool | None: ...
    @cant_split.setter
    def cant_split(self, value: bool | None) -> None: ...
    @property
    def is_header(self) -> bool | None: ...
    @is_header.setter
    def is_header(self, value: bool | None) -> None: ...


@_final
class RowCollection:
    def __new__(cls, *, _private: _Never) -> RowCollection: ...
    def __len__(self) -> int: ...
    @_overload
    def __getitem__(self, key: int, /) -> Row: ...
    @_overload
    def __getitem__(self, key: slice, /) -> list[Row]: ...
    def __iter__(self) -> _Iterator[Row]: ...


@_final
class Cell:
    def __new__(cls, *, _private: _Never) -> Cell: ...
    def replace_text(
        self, old: str, new: str, *, expect: int | None = None
    ) -> int: ...
    @property
    def text(self) -> str: ...
    @text.setter
    def text(self, value: str) -> None: ...
    # Cells of a nested table raise NotImplementedError for paragraphs,
    # add_paragraph and replace_text.
    @property
    def paragraphs(self) -> CellParagraphCollection: ...
    def add_paragraph(self, text: str) -> Paragraph: ...
    @property
    def tables(self) -> list[Table]: ...
    def add_table(self, rows: int, cols: int) -> Table: ...
    def split(self) -> int: ...
    def xml(self) -> bytes:
        """This cell's ``w:tc`` as standalone XML."""
    def replace_xml(self, xml: _Xml) -> None:
        """Replace this cell with one ``w:tc``, checked as ``Paragraph.replace_xml`` checks."""
    @property
    def width(self) -> _shared.Length | None: ...
    @width.setter
    def width(self, value: int) -> None: ...
    @property
    def vertical_alignment(self) -> _table.WD_CELL_VERTICAL_ALIGNMENT | None: ...
    @vertical_alignment.setter
    def vertical_alignment(self, value: _table.WD_CELL_VERTICAL_ALIGNMENT) -> None: ...
    @property
    def grid_span(self) -> int: ...
    @property
    def vertical_merge(self) -> _Literal["restart", "continue"] | None: ...
    @property
    def shading(self) -> _shared.RGBColor | None: ...
    @shading.setter
    def shading(self, value: _Color) -> None: ...
    def border(
        self, edge: _BorderEdge
    ) -> tuple[str, int | None, _shared.RGBColor | None] | None: ...
    def set_border(
        self, edge: _BorderEdge, style: _BorderStyle, *, size: int, color: _Color
    ) -> None: ...
    @property
    def margins(self) -> _Margins | None: ...
    def set_margins(self, *, top: int, right: int, bottom: int, left: int) -> None: ...


@_final
class CellCollection:
    def __new__(cls, *, _private: _Never) -> CellCollection: ...
    def __len__(self) -> int: ...
    @_overload
    def __getitem__(self, key: int, /) -> Cell: ...
    @_overload
    def __getitem__(self, key: slice, /) -> list[Cell]: ...
    def __iter__(self) -> _Iterator[Cell]: ...


@_final
class CellParagraphCollection:
    def __new__(cls, *, _private: _Never) -> CellParagraphCollection: ...
    def __len__(self) -> int: ...
    @_overload
    def __getitem__(self, key: int, /) -> Paragraph: ...
    @_overload
    def __getitem__(self, key: slice, /) -> list[Paragraph]: ...
    def __iter__(self) -> _Iterator[Paragraph]: ...
