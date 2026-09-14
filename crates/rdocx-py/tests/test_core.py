import io
import zipfile

import pytest


def _replace_document_body(document, body):
    source = io.BytesIO(document.to_bytes())
    result = io.BytesIO()
    with zipfile.ZipFile(source) as source_zip:
        with zipfile.ZipFile(result, "w") as result_zip:
            for info in source_zip.infolist():
                data = source_zip.read(info.filename)
                if info.filename == "word/document.xml":
                    start = data.index(b"<w:body>") + len(b"<w:body>")
                    end = data.index(b"</w:body>")
                    data = data[:start] + body.encode() + data[end:]
                result_zip.writestr(info, data)
    return type(document).from_bytes(result.getvalue())


def _document_xml(document):
    with zipfile.ZipFile(io.BytesIO(document.to_bytes())) as archive:
        return archive.read("word/document.xml")


def test_stale_paragraph_after_structural_removal_raises_named_error():
    import rdocx

    doc = rdocx.Document()
    for text in ("zero", "one", "two", "three", "four"):
        doc.add_paragraph(text)

    held = doc.paragraphs[3]
    assert doc.remove_content(1) is True

    with pytest.raises(
        rdocx.StaleElementError,
        match=(
            r"paragraph handle was created at document revision 5, but the document "
            r"is now at revision 6"
        ),
    ):
        _ = held.text


def test_lazy_collections_support_index_slice_and_iteration():
    import rdocx

    doc = rdocx.Document()
    for text in ("alpha", "beta", "gamma"):
        doc.add_paragraph(text)

    paragraphs = doc.paragraphs
    assert len(paragraphs) == 3
    assert paragraphs[-1].text == "gamma"
    assert [paragraph.text for paragraph in paragraphs[0:3:2]] == ["alpha", "gamma"]
    assert [paragraph.text for paragraph in paragraphs] == ["alpha", "beta", "gamma"]

    paragraph = doc.paragraphs[0]
    paragraph.add_run(" one")
    paragraph = doc.paragraphs[0]
    paragraph.add_run(" two")
    runs = doc.paragraphs[0].runs
    assert len(runs) == 3
    assert runs[-1].text == " two"
    assert [run.text for run in runs[0:3:2]] == ["alpha", " two"]
    assert [run.text for run in runs] == ["alpha", " one", " two"]


def test_failed_removal_does_not_stale_live_handles():
    import rdocx

    doc = rdocx.Document()
    doc.add_paragraph("live")
    held = doc.paragraphs[0]

    assert doc.remove_content(99) is False
    assert held.text == "live"


def test_core_text_mutations_survive_bytes_round_trip():
    import rdocx

    doc = rdocx.Document()
    paragraph = doc.add_paragraph("Hello")
    run = paragraph.add_run(" world")
    reopened = rdocx.Document.from_bytes(doc.to_bytes())

    assert reopened.paragraphs[0].text == "Hello world"
    reopened.paragraphs[0].runs[1].text = " Rust"
    reopened_again = rdocx.Document.from_bytes(reopened.to_bytes())
    assert reopened_again.paragraphs[0].text == "Hello Rust"


def test_constructor_accepts_an_optional_input_path(tmp_path):
    import rdocx

    assert len(rdocx.Document().paragraphs) == 0

    path = tmp_path / "input.docx"
    source = rdocx.Document()
    source.add_paragraph("opened by constructor")
    source.save(path)

    reopened = rdocx.Document(path)
    assert reopened.paragraphs[0].text == "opened by constructor"


def test_priority_word_operations_return_typed_snapshots_and_remain_atomic():
    import rdocx

    position = rdocx.RunPosition(body_index=0, run_index=0)
    range_ = rdocx.RunRange(
        start=position,
        end=rdocx.RunPosition(body_index=0, run_index=1),
    )
    with pytest.raises(AttributeError):
        position.run_index = 2

    commented = rdocx.Document()
    commented.add_paragraph("review this")
    held_before_comment = commented.paragraphs[0]
    comment_id = commented.add_comment(
        range_, author="Ada", text="Please revise", initials="AL"
    )
    with pytest.raises(
        rdocx.StaleElementError, match=r"revision 1, but the document is now at revision 2"
    ):
        _ = held_before_comment.text
    held_before_reply = commented.paragraphs[0]
    reply_id = commented.reply_to(comment_id, author="Grace", text="Done")
    with pytest.raises(
        rdocx.StaleElementError, match=r"revision 2, but the document is now at revision 3"
    ):
        _ = held_before_reply.text
    held_before_resolve = commented.paragraphs[0]
    assert commented.resolve_comment(comment_id, resolved=True) is True
    with pytest.raises(
        rdocx.StaleElementError, match=r"revision 3, but the document is now at revision 4"
    ):
        _ = held_before_resolve.text
    assert commented.comments == (
        rdocx.Comment(
            id=comment_id,
            author="Ada",
            initials="AL",
            date=None,
            text="Please revise",
            parent_id=None,
            resolved=True,
        ),
        rdocx.Comment(
            id=reply_id,
            author="Grace",
            initials=None,
            date=None,
            text="Done",
            parent_id=comment_id,
            resolved=False,
        ),
    )
    reopened_comments = rdocx.Document.from_bytes(commented.to_bytes())
    assert reopened_comments.comments == commented.comments

    live = reopened_comments.paragraphs[0]
    before_failure = reopened_comments.to_bytes()
    invalid = rdocx.RunRange(
        start=rdocx.RunPosition(body_index=99, run_index=0),
        end=rdocx.RunPosition(body_index=99, run_index=1),
    )
    with pytest.raises(rdocx.RdocxError):
        reopened_comments.add_comment(invalid, author="Ada", text="invalid")
    assert live.text == "review this"
    assert reopened_comments.to_bytes() == before_failure
    held_before_remove = reopened_comments.paragraphs[0]
    assert reopened_comments.remove_comment(reply_id) is True
    with pytest.raises(
        rdocx.StaleElementError, match=r"revision 0, but the document is now at revision 1"
    ):
        _ = held_before_remove.text
    live_after_noop = reopened_comments.paragraphs[0]
    assert reopened_comments.remove_comment(reply_id) is False
    assert live_after_noop.text == "review this"

    original = rdocx.Document()
    original.add_paragraph("before")
    edited = rdocx.Document()
    edited.add_paragraph("after")
    held_before_compare = original.paragraphs[0]
    diagnostics = original.compare(
        edited, author="Ada", timestamp="2026-09-14T09:00:00Z"
    )
    assert isinstance(diagnostics, tuple)
    assert all(isinstance(item, rdocx.ComparisonDiagnostic) for item in diagnostics)
    with pytest.raises(
        rdocx.StaleElementError, match=r"revision 1, but the document is now at revision 2"
    ):
        _ = held_before_compare.text
    reopened_redline = rdocx.Document.from_bytes(original.to_bytes())
    assert b"before" in _document_xml(reopened_redline)
    assert b"after" in _document_xml(reopened_redline)
    with pytest.raises(rdocx.RdocxError, match="existing modeled revisions"):
        reopened_redline.compare(
            edited, author="Ada", timestamp="2026-09-14T09:01:00Z"
        )

    failed_compare = rdocx.Document()
    failed_compare.add_paragraph("stable")
    live_after_failure = failed_compare.paragraphs[0]
    before_failure = failed_compare.to_bytes()
    with pytest.raises(rdocx.RdocxError):
        failed_compare.compare(edited, author="Ada", timestamp="not-a-timestamp")
    assert live_after_failure.text == "stable"
    assert failed_compare.to_bytes() == before_failure

    unchanged = rdocx.Document()
    unchanged.add_paragraph("same")
    identical = rdocx.Document.from_bytes(unchanged.to_bytes())
    held_before_identical_compare = unchanged.paragraphs[0]
    assert unchanged.compare(
        identical, author="Ada", timestamp="2026-09-14T09:02:00Z"
    ) == ()
    with pytest.raises(
        rdocx.StaleElementError, match=r"revision 1, but the document is now at revision 2"
    ):
        _ = held_before_identical_compare.text

    laid_out = rdocx.Document()
    laid_out.add_paragraph("A deterministic layout fragment")
    fragments = laid_out.layout()
    assert fragments
    assert fragments[0].body_index == 0
    assert fragments[0].physical_page == 1
    assert fragments[0].displayed_page == 1
    assert isinstance(fragments[0].bounds, rdocx.BoundingBox)
    assert fragments[0].bounds.width > 0.0
    page = laid_out.layout_page(0)
    assert page == rdocx.LayoutPage(
        page_number=1,
        displayed_page_number=1,
        width=page.width,
        height=page.height,
    )
    assert laid_out.layout_page(1) is None

    toc_source = rdocx.Document()
    toc_source.add_paragraph("placeholder")
    toc = _replace_document_body(
        toc_source,
        """
        <w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>TOC \\o "1-1"</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r></w:p>
        <w:p><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>
        <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Heading</w:t></w:r></w:p>
        """,
    )
    held_before_toc = toc.paragraphs[-1]
    report = toc.rebuild_toc()
    assert report == rdocx.TocRebuildReport(
        entry_count=1, bookmark_count=1, diagnostic_count=0
    )
    with pytest.raises(
        rdocx.StaleElementError, match=r"revision 0, but the document is now at revision 1"
    ):
        _ = held_before_toc.text
    reopened_toc = rdocx.Document.from_bytes(toc.to_bytes())
    assert "Heading" in [paragraph.text for paragraph in reopened_toc.paragraphs]

    no_toc = rdocx.Document()
    no_toc.add_paragraph("no table of contents")
    live_after_noop = no_toc.paragraphs[0]
    assert no_toc.rebuild_toc() == rdocx.TocRebuildReport(
        entry_count=0, bookmark_count=0, diagnostic_count=0
    )
    assert live_after_noop.text == "no table of contents"
