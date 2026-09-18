from pathlib import Path
from typing import TYPE_CHECKING, assert_type

from rdocx import (
    BoundingBox,
    Cell,
    CellCollection,
    CellParagraphCollection,
    Comment,
    ComparisonDiagnostic,
    ContentFragment,
    Document,
    Font,
    HeaderFooterVariant,
    Hyperlink,
    Inches,
    LayoutFragment,
    LayoutBackedFieldUpdateReport,
    LayoutPage,
    RGBColor,
    Paragraph,
    ParagraphCollection,
    ParagraphFormat,
    Revision,
    Row,
    RowCollection,
    Run,
    RunCollection,
    RunPosition,
    RunRange,
    Section,
    Story,
    StoryItem,
    StoryRunPosition,
    StoryRunRange,
    Style,
    Table,
    TableCollection,
    TocRebuildReport,
)


def exercise_rdocx_types(path: Path) -> None:
    document = Document(path)
    opened: Document = Document.open(path)
    loaded: Document = Document.from_bytes(b"")
    paragraph: Paragraph = document.add_paragraph("typed")
    run: Run = paragraph.add_run(" run")
    paragraph.style = "Heading1"
    paragraph.numbering = (1, 2)
    assert_type(paragraph.style, str)
    assert_type(paragraph.numbering, tuple[int, int])
    run.style_id = "Strong"
    assert_type(run.style_id, str)
    font: Font = run.font
    font.bold = True
    font.size = Inches(1)
    font.highlight = "yellow"
    font.shading = "FFFF00"
    color = RGBColor(1, 2, 3)
    assert_type(color[0], int)
    channels: tuple[int, int, int] = color
    paragraph_format: ParagraphFormat = paragraph.paragraph_format
    paragraph_format.keep_together = None
    split_boundary: int = document.split_run(0, 0, 1)
    paragraphs: ParagraphCollection = document.paragraphs
    first: Paragraph = paragraphs[0]
    sliced: list[Paragraph] = paragraphs[:]
    for item in paragraphs:
        item.text
    table: Table = document.add_table(1, 1)
    row: Row = table.rows[0]
    cloned_row: Row = table.clone_row(0)
    table.remove_row(0)
    cell: Cell = row.cells[0]
    cell.text = first.text
    package_bytes: bytes = loaded.to_bytes()
    pdf_bytes: bytes = opened.to_pdf()
    pages: list[bytes] = opened.render_all_pages()
    maybe_page: bytes | None = opened.render_page_to_png(0)
    document.save(path)
    document.remove_content(0)
    position = RunPosition(body_index=0, run_index=0)
    range_ = RunRange(start=position, end=RunPosition(body_index=0, run_index=1))
    comment_id: int = document.add_comment(
        range_,
        author="Ada",
        text="review",
        initials=None,
        date="2026-09-16T10:15:30Z",
    )
    reply_id: int = document.reply_to(
        comment_id,
        author="Grace",
        text="done",
        date="2026-09-16T11:00:00+01:00",
    )
    comments: tuple[Comment, ...] = document.comments
    sections: tuple[Section, ...] = document.sections
    styles: tuple[Style, ...] = document.styles
    stories: tuple[Story, ...] = document.stories
    image_data: bytes | None = document.image_data("rId1")
    document.replace_image("rId1", b"image")
    document.replace_image_for_story(stories[0], "rId1", b"image")
    story_items: tuple[StoryItem, ...] = document.story_items
    inserted_picture: StoryItem = document.add_picture(
        b"png", "image.png", Inches(1), Inches(1), after=story_items[0]
    )
    story_position = StoryRunPosition(item=story_items[0], run_index=0)
    story_range = StoryRunRange(start=story_position, end=story_position)
    story_comment_id: int = document.add_comment(
        story_range, author="Ada", text="story review"
    )
    direct_body_index: int | None = story_items[0].direct_body_index if story_items else None
    variants: tuple[HeaderFooterVariant, ...] = document.header_footer_variants
    hyperlinks: tuple[Hyperlink, ...] = document.hyperlinks
    resolved: bool = document.resolve_comment(comment_id)
    removed: bool = document.remove_comment(reply_id)
    diagnostics: tuple[ComparisonDiagnostic, ...] = document.compare(
        opened, author="Ada", timestamp="2026-09-14T09:00:00Z"
    )
    fragments: tuple[LayoutFragment, ...] = document.layout()
    maybe_layout_page: LayoutPage | None = document.layout_page(0)
    report: TocRebuildReport = document.rebuild_toc()
    update_fields_on_open: bool | None = document.update_fields_on_open
    document.update_fields_on_open = True
    document.update_fields_on_open = None
    content_index: int = document.find_content_index(first)
    inserted: Paragraph = document.insert_paragraph(content_index, "inserted")
    fragment: ContentFragment = document.pop_content(content_index)
    fragment_kind: str = fragment.kind
    document.insert_content(content_index, fragment)
    document.clone_content(inserted, content_index)
    document.move_content(table, content_index)
    replacement_count: int = document.try_replace_text("old", "new")
    regex_count: int = document.replace_all_regex([("old", "new")])
    revisions: tuple[Revision, ...] = document.revisions
    accepted: int = document.accept_all()
    dated: int = document.reject_revisions_in_date_range(
        start="2026-01-01T00:00:00Z", end="2026-12-31T00:00:00Z"
    )
    replaced: int = document.try_replace_text("{{name}}", "Ada")
    matched: int = document.replace_all_regex([(r"\d", "#")])
    updated: int = document.update_fields(
        file_name="report.docx", merge_fields={"Name": "Ada"}
    )
    assert_type(revisions[0].timestamp, str | None)
    document.set_header("Header")
    document.set_footer("Footer")
    document.set_story_text(story_items[0], "edited")
    document.add_hyperlink_to_story(stories[0], "home", "https://example.com/")
    link_run: Run = first.add_hyperlink("docs", "https://example.com/docs")
    assert_type(story_items[0].xml, bytes)
    compatible_story_item = StoryItem(
        story=stories[0],
        kind="paragraph",
        index_path=(0,),
        text=None,
        xml=None,
    )
    assert_type(compatible_story_item.xml, bytes)
    text_content_index: int = document.find_content_index("typed")
    text_content_indices: tuple[int, ...] = document.find_content_indices("typed")
    page_fields_updated: int = document.update_page_fields()
    layout_field_report: LayoutBackedFieldUpdateReport = (
        document.update_layout_backed_fields()
    )
    if fragments:
        bounds: BoundingBox = fragments[0].bounds
        assert_type(bounds.width, float)
    assert_type(comments[0].date, str | None)
    assert_type(sections[0].page_width, int | None)
    assert_type(styles[0].style_type, str)
    assert_type(stories[0].owner_index, int)
    assert_type(story_items[0].index_path, tuple[int, ...])
    assert_type(story_items[0].revision, int)
    assert_type(variants[0].story, Story | None)
    assert_type(hyperlinks[0].url, str | None)
    assert_type(report.entry_count, int)
    assert_type(report.diagnostics, tuple[str, ...])
    assert_type(report.diagnostic_count, int)
    (
        package_bytes,
        pdf_bytes,
        pages,
        maybe_page,
        sliced,
        channels,
        cloned_row,
        update_fields_on_open,
        replacement_count,
        regex_count,
        image_data,
        fragment_kind,
        inserted_picture,
        story_comment_id,
    )
    accepted, dated, replaced, matched, updated


if TYPE_CHECKING:
    Cell()  # type: ignore[call-arg]
    CellCollection()  # type: ignore[call-arg]
    CellParagraphCollection()  # type: ignore[call-arg]
    Font()  # type: ignore[call-arg]
    Paragraph()  # type: ignore[call-arg]
    ParagraphCollection()  # type: ignore[call-arg]
    ParagraphFormat()  # type: ignore[call-arg]
    Row()  # type: ignore[call-arg]
    RowCollection()  # type: ignore[call-arg]
    Run()  # type: ignore[call-arg]
    RunCollection()  # type: ignore[call-arg]
    Table()  # type: ignore[call-arg]
    TableCollection()  # type: ignore[call-arg]
