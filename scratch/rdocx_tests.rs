
    /// A document whose body holds `body` before the final section, opened
    /// from bytes so its part root is the one `root` gives.
    fn issue407_document(body: &str, root_w14: bool) -> Document {
        let mut package =
            OpcPackage::from_reader(Cursor::new(Document::new().to_bytes().unwrap())).unwrap();
        let mut xml =
            String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
        let at = xml.find("<w:sectPr").unwrap();
        xml.insert_str(at, body);
        if root_w14 {
            let root = xml.find("<w:document").unwrap() + "<w:document".len();
            xml.insert_str(
                root,
                r#" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#,
            );
        }
        package.set_part("/word/document.xml", xml.into_bytes());
        let mut bytes = Cursor::new(Vec::new());
        package.write_to(&mut bytes).unwrap();
        Document::from_bytes(bytes.get_ref()).unwrap()
    }

    /// Paragraphs of six runs, each with the run properties Google Docs
    /// writes, as in the #407 report.
    fn issue407_runs(paragraphs: usize) -> String {
        const RPR: &str = r#"<w:rPr><w:rFonts w:ascii="Arial" w:cs="Arial" w:eastAsia="Arial" w:hAnsi="Arial"/><w:b w:val="0"/><w:i w:val="0"/><w:color w:val="000000"/><w:sz w:val="20"/><w:u w:val="none"/></w:rPr>"#;
        (0..paragraphs)
            .map(|paragraph| {
                let runs = (0..6)
                    .map(|run| {
                        format!(
                            r#"<w:r>{RPR}<w:t xml:space="preserve">run {run} of paragraph {paragraph} </w:t></w:r>"#
                        )
                    })
                    .collect::<String>();
                format!("<w:p>{runs}</w:p>")
            })
            .collect()
    }

    /// Paragraphs of varied content, then a table and a block control.
    fn issue407_varied_body(other_w14: &str) -> String {
        [
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:rPr><w:b/><w:color w:val="FF0000"/></w:rPr><w:t>heading</w:t></w:r></w:p>"#,
            r#"<w:p><w:r><w:t xml:space="preserve">see </w:t></w:r><w:hyperlink w:anchor="target"><w:r><w:t>the target</w:t></w:r></w:hyperlink></w:p>"#,
            r#"<w:p><w:fldSimple w:instr=" PAGE "><w:r><w:t>1</w:t></w:r></w:fldSimple><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> DATE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>today</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#,
            r#"<w:p><w:bookmarkStart w:id="1" w:name="target"/><w:r><w:t>marked</w:t></w:r><w:bookmarkEnd w:id="1"/></w:p>"#,
            r#"<w:p><w:sdt><w:sdtPr><w:alias w:val="inline"/></w:sdtPr><w:sdtContent><w:r><w:t>controlled</w:t></w:r></w:sdtContent></w:sdt></w:p>"#,
            r#"<w:p><w:ins w:id="2" w:author="a" w:date="2026-01-01T00:00:00Z"><w:r><w:t>inserted</w:t></w:r></w:ins><w:del w:id="3" w:author="a" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>deleted</w:delText></w:r></w:del></w:p>"#,
            r#"<w:p><w:r><w:t>tab</w:t></w:r><w:r><w:tab/><w:t>bed</w:t><w:br/></w:r></w:p>"#,
            other_w14,
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
            r#"<w:sdt><w:sdtPr><w:alias w:val="block"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>block one</w:t></w:r></w:p><w:p><w:r><w:t>block two</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
            r#"<w:p><w:r><w:t>last</w:t></w:r></w:p>"#,
        ]
        .concat()
    }

    #[test]
    fn issue407_direct_body_items_read_as_their_story_source() {
        const PARA_ID: &str = r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="388B6825"><w:r><w:t>identified</w:t></w:r></w:p>"#;
        const TEXT: &str = r#"<w:p><w:r><w:t xml:space="preserve">a w14:literal</w:t></w:r></w:p>"#;
        // The part root declares `w14` when it or any part byte uses it, so a
        // paragraph that does not use `w14`, under a root that does not
        // declare it, cannot read its own closure alone.
        for (root_w14, other, decided) in [
            (true, "", true),
            (true, PARA_ID, true),
            (false, "", false),
            (false, PARA_ID, false),
            (false, TEXT, false),
        ] {
            let document = issue407_document(&issue407_varied_body(other), root_w14);
            let case = format!("root_w14={root_w14} other={other:?}");
            let snapshots = document.story_item_snapshots().unwrap();
            let mut paragraphs = 0;
            for (slot, content) in document.document.body.content.iter().enumerate() {
                // Story items count per direct body slot, nested ones included.
                let items = snapshots
                    .iter()
                    .filter(|item| {
                        item.location().story().kind() == StoryKind::Body
                            && item.direct_body_index() == Some(slot)
                    })
                    .count();
                assert_eq!(
                    document.direct_body_story_item_count(slot).unwrap(),
                    Some(items),
                    "{case} slot={slot}"
                );
                match content {
                    BodyContent::Paragraph(paragraph) => {
                        let location = document.paragraph_story_location(paragraphs).unwrap().unwrap();
                        paragraphs += 1;
                        let expected = document.paragraph_element_xml(&location).unwrap();
                        let uses_w14 = String::from_utf8_lossy(&expected).contains(" w14:");
                        match document.direct_body_paragraph_xml(slot).unwrap() {
                            Some(actual) => assert_eq!(actual, expected, "{case} slot={slot}"),
                            None => assert!(!decided && !uses_w14, "{case} slot={slot}"),
                        }
                        let parsed = CT_P::from_xml_fragment(&expected).unwrap();
                        for path in parsed.accepted_run_paths() {
                            assert_eq!(
                                document.direct_body_run_xml(slot, &path).unwrap(),
                                Some(document.run_element_xml(&location, &path).unwrap()),
                                "{case} slot={slot} run={path:?}"
                            );
                        }
                        if paragraph_is_plain(paragraph) {
                            assert_eq!(
                                document.direct_body_paragraph_facts(paragraph).unwrap(),
                                Some((parsed.accepted_text(), parsed.accepted_run_paths())),
                                "{case} slot={slot}"
                            );
                        }
                    }
                    BodyContent::Table(_) => {
                        let item = snapshots
                            .iter()
                            .find(|item| {
                                item.location().item_kind() == StoryItemKind::Table
                                    && item.direct_body_index() == Some(slot)
                            })
                            .unwrap();
                        let expected = document.table_element_xml(item.location()).unwrap();
                        assert_eq!(
                            document.direct_body_table_xml(slot).unwrap(),
                            decided.then_some(expected),
                            "{case} slot={slot}"
                        );
                    }
                    BodyContent::ContentControl(control) => {
                        paragraphs += control.content.len();
                    }
                    BodyContent::RawXml(_) => {}
                }
            }
            assert!(paragraphs > 8, "{case}");
        }
    }

    #[test]
    fn issue407_direct_body_reads_and_plain_edits_do_not_grow_with_the_body() {
        let mut work = Vec::new();
        for count in [100usize, 400] {
            let mut document = issue407_document(&issue407_runs(count), true);
            let counters = || {
                (
                    STORY_SOURCE_BUILDS.get(),
                    STORY_OWNER_SCANS.get(),
                    FX237_ITEM_SCANS.get(),
                    FX211_PROFILE.get()[1],
                    FX237_PREFIX_NAMESPACE_BYTES.get(),
                    STANDALONE_BODY_ITEM_BYTES.get(),
                    fx260_body_visits(),
                )
            };
            let reset = || {
                STORY_SOURCE_BUILDS.set(0);
                STORY_OWNER_SCANS.set(0);
                FX237_ITEM_SCANS.set(0);
                FX211_PROFILE.set([0; 12]);
                FX237_PREFIX_NAMESPACE_BYTES.set(0);
                STANDALONE_BODY_ITEM_BYTES.set(0);
                reset_fx260_body_visits();
            };
            let mut sample = Vec::new();
            reset();
            assert!(document.direct_body_paragraph_xml(20).unwrap().is_some());
            let path = CT_P::from_xml_fragment(&document.direct_body_paragraph_xml(20).unwrap().unwrap())
                .unwrap()
                .accepted_run_paths()
                .remove(3);
            assert!(document.direct_body_run_xml(20, &path).unwrap().is_some());
            sample.push(counters());
            reset();
            let paragraph = Document::detached_text_paragraph("x");
            assert!(document.direct_body_paragraph_facts(&paragraph).unwrap().is_some());
            document.insert_detached_paragraph(10, paragraph);
            sample.push(counters());
            reset();
            assert!(document.try_remove_content(10).unwrap());
            sample.push(counters());
            reset();
            let last = document.content_count() - 1;
            assert_eq!(document.direct_body_story_item_count(last).unwrap(), Some(1));
            document.push_detached_paragraph(Document::detached_text_paragraph("y"));
            sample.push(counters());
            reset();
            assert_eq!(document.remove_content_range(10, 12).unwrap(), 2);
            sample.push(counters());
            // No body serialization and no body scan, as in a larger body.
            assert!(
                sample.iter().all(|counts| counts.0 == 0 && counts.1 <= 1 && counts.2 <= 1),
                "count={count} {sample:?}"
            );
            work.push(sample);
        }
        assert_eq!(work[0], work[1]);
    }
