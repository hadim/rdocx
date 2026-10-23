import sys
p=sys.argv[1]
s=open(p).read()
anchor="""    #[cfg(feature = "default-template")]
    #[test]
    fn issue403_a_commit_reuses_unchanged_records_and_keeps_edits_made_in_place() {"""
assert s.count(anchor)==1
test='''    /// Fitting the text of a frame measures its own slide only: a slide
    /// whose layout relationship is missing, or a notes master without its
    /// theme, which fail the reports of the whole deck, do not fail it.
    #[cfg(all(feature = "default-template", feature = "render"))]
    #[test]
    fn issue403_fit_text_does_not_fail_on_another_broken_slide() {
        let mut deck = Presentation::new().unwrap();
        for index in 0..3 {
            deck.add_slide(6).unwrap();
            deck.slide_mut(index)
                .unwrap()
                .add_textbox(Emu(0), Emu(0), Emu(3_000_000), Emu(1_000_000))
                .unwrap()
                .set_text(&"fit me ".repeat(20))
                .unwrap();
        }
        deck.set_notes_text(1, "notes").unwrap();
        let bytes = deck.to_bytes().unwrap();
        let broken = |owner: &str, rel_type: &str| {
            let mut package = OpcPackage::from_reader(Cursor::new(&bytes)).unwrap();
            package
                .get_part_rels_mut(owner)
                .unwrap()
                .items
                .retain(|relationship| relationship.rel_type != rel_type);
            let mut output = Cursor::new(Vec::new());
            package.write_to(&mut output).unwrap();
            Presentation::from_bytes(output.get_ref()).unwrap()
        };
        let notes_master = deck.notes_master_part.clone().unwrap();
        for mut broken in [
            broken(&deck.slides[2].part_name, rel_types::SLIDE_LAYOUT),
            broken(&notes_master, rel_types::THEME),
        ] {
            assert!(broken.fit_report(&[]).is_err());
            let last = broken.slide(0).unwrap().shapes().count() - 1;
            let location = broken.text_frame_location(0, &[last], None).unwrap();
            broken.fit_text_at(&location, None, 18, &[]).unwrap();
        }
    }

'''
s=s.replace(anchor, test+anchor)
open(p,'w').write(s)
