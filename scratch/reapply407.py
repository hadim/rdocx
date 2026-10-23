p='crates/rdocx-py/src/document.rs'
s=open(p).read()
def rep(old, new, count=1):
    global s
    assert s.count(old)==count, old[:90]
    s=s.replace(old,new)

# --- item 5: building moved identities
rep('''        let route = |location: &mut rdocx::ContentLocation| {
            let story = location.story();
            if story.kind() == rdocx::StoryKind::TableCell
                && story.part_name() == part
                && story.owner_index() >= first
            {
                let owner_index = story.owner_index() + count;
                location.set_story_owner_index(owner_index);
            }
        };''','''        let route = |location: &mut rdocx::ContentLocation| {
            let story = location.story();
            if story.kind() == rdocx::StoryKind::TableCell
                && story.part_name() == part
                && story.owner_index() >= first
            {
                let mut story = story.clone();
                story.set_owner_index(story.owner_index() + count);
                *location = rdocx::ContentLocation::new(
                    story,
                    location.item_kind(),
                    location.index_path().to_vec(),
                );
            }
        };''')
rep('''    let moved = |location: &rdocx::ContentLocation| {
        let mut location = location.clone();
        location.set_story_owner_index(location.story().owner_index() + cells_before);
        location
    };''','''    let moved = |location: &rdocx::ContentLocation| {
        let mut story = location.story().clone();
        story.set_owner_index(story.owner_index() + cells_before);
        rdocx::ContentLocation::new(story, location.item_kind(), location.index_path().to_vec())
    };''')

# --- item 6/7: flows through the public API, plain removable paragraphs
rep('''            let mut inserted = rdocx::Document::detached_text_paragraph(text);
            if let Some(style) = style {
                rdocx::Paragraph::detached(&mut inserted).set_style(&style);
            }
            let splice = document.inner.direct_body_paragraph_facts(&inserted);''','''            // The same edit builds the paragraph read for the inventory and
            // the paragraph placed.
            let edit = |paragraph: &mut rdocx::Paragraph<'_>| {
                if let Some(style) = &style {
                    paragraph.set_style(style);
                }
            };
            let mut inserted = rdocx::Document::detached_text_paragraph(text);
            edit(&mut rdocx::Paragraph::detached(&mut inserted));
            let splice = document.inner.direct_body_paragraph_facts(&inserted);''')
rep('''                document.inner.insert_detached_paragraph(index, inserted);
                debug_assert_eq!(''','''                edit(&mut document.inner.insert_paragraph(index, text));
                debug_assert_eq!(''')
rep('''            candidate.insert_detached_paragraph(index, inserted);''','''            edit(&mut candidate.insert_paragraph(index, text));''')
rep('''                document.inner.insert_detached_table(index, inserted);''','''                document.inner.insert_table(index, rows, cols);''')
rep('''            candidate.insert_detached_table(index, inserted);''','''            candidate.insert_table(index, rows, cols);''')
rep('''                document.inner.push_detached_table(appended);''','''                document.inner.add_table(rows, cols);''')
rep('''        let mut appended = rdocx::Document::detached_text_paragraph(text);
        {
            let mut paragraph = rdocx::Paragraph::detached(&mut appended);
            if let Some(style) = style {
                paragraph.set_style(&style);
            }
            if page_break {
                paragraph.add_run("").add_break(rdocx::BreakKind::Page);
            }
        }''','''        // The same edit builds the paragraph read for the inventory and the
        // paragraph placed.
        let edit = |paragraph: &mut rdocx::Paragraph<'_>| {
            if let Some(style) = &style {
                paragraph.set_style(style);
            }
            if page_break {
                paragraph.add_run("").add_break(rdocx::BreakKind::Page);
            }
        };
        let mut appended = rdocx::Document::detached_text_paragraph(text);
        edit(&mut rdocx::Paragraph::detached(&mut appended));''')
rep('''            document.inner.push_detached_paragraph(appended);''','''            edit(&mut document.inner.add_paragraph(text));''')
rep('''        candidate.push_detached_paragraph(appended);''','''        edit(&mut candidate.add_paragraph(text));''')
rep('''                .all(|slot| self.inner.plain_direct_paragraph_without_section(slot))''','''                .all(|slot| self.inner.plain_removable_paragraph(slot))''')

# --- items 1/3: guard and counters
old=s[s.index('/// Under debug assertions, check a direct read against the whole-source\n/// read: the same XML up to root declarations the fragment does not use.'):s.index('#[cfg(debug_assertions)]\nfn unfingerprinted_location(')]
s=s.replace(old,'''/// Under debug assertions, check a direct read against the whole-source
/// read: the same bytes up to the root `w14` declaration, which a direct
/// read decides from the root and the read item alone.
#[cfg(debug_assertions)]
pub(crate) fn assert_direct_read(
    direct: &[u8],
    whole: impl FnOnce() -> PyResult<Vec<u8>>,
    owner: &str,
) {
    let without_w14 = |xml: &[u8]| {
        let xml = String::from_utf8_lossy(xml).into_owned();
        let root = xml.find('>').unwrap_or(xml.len());
        let declaration = r#" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#;
        format!("{}{}", xml[..root].replacen(declaration, "", 1), &xml[root..])
    };
    let whole = guarded(whole).expect("a direct read has a whole-source read");
    assert_eq!(
        without_w14(direct),
        without_w14(&whole),
        "direct {owner} XML"
    );
}

''')
rep('''use oxml_py_support::{''','''// Debug builds count the whole-body work that direct reads and body splices
// avoid, outside the guards that check them against it, so that a test of
// the extension module proves each call took its direct route.
#[cfg(debug_assertions)]
static WORD_INVENTORY_REBUILDS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(debug_assertions)]
static WORD_WHOLE_SOURCE_READS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(debug_assertions)]
thread_local! {
    static WORD_GUARDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Count one owner inventory rebuild from the whole document.
fn note_word_inventory_rebuild() {
    #[cfg(debug_assertions)]
    if !WORD_GUARDING.with(std::cell::Cell::get) {
        WORD_INVENTORY_REBUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Count one element read from its whole story source.
pub(crate) fn note_word_whole_source_read() {
    #[cfg(debug_assertions)]
    if !WORD_GUARDING.with(std::cell::Cell::get) {
        WORD_WHOLE_SOURCE_READS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Run a debug guard's own whole-body work without counting it.
#[cfg(debug_assertions)]
fn guarded<T>(work: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            WORD_GUARDING.with(|cell| cell.set(self.0));
        }
    }
    let _restore = Restore(WORD_GUARDING.with(|cell| cell.replace(true)));
    work()
}

/// The whole-body work counted so far: owner inventory rebuilds and
/// whole-source element reads. Debug builds only.
#[cfg(debug_assertions)]
#[pyfunction]
#[pyo3(name = "_word_whole_body_work")]
pub(crate) fn word_whole_body_work() -> (usize, usize) {
    (
        WORD_INVENTORY_REBUILDS.load(std::sync::atomic::Ordering::Relaxed),
        WORD_WHOLE_SOURCE_READS.load(std::sync::atomic::Ordering::Relaxed),
    )
}

use oxml_py_support::{''')
rep('''fn word_owner_locations(
    document: &rdocx::Document,
    form_sources: Option<&mut FormSources>,
) -> rdocx::Result<Vec<WordOwnerAddress>> {
    note_fx260_word(document.content_count().max(1));''','''fn word_owner_locations(
    document: &rdocx::Document,
    form_sources: Option<&mut FormSources>,
) -> rdocx::Result<Vec<WordOwnerAddress>> {
    note_fx260_word(document.content_count().max(1));
    note_word_inventory_rebuild();''')
rep('''        let rebuilt =
            word_owner_locations(&self.inner, None).expect("a spliced inventory has a rebuild");
        let mut expected = vec![None; self.owners.len() + plan.added()];''','''        let rebuilt = guarded(|| word_owner_locations(&self.inner, None))
            .expect("a spliced inventory has a rebuild");
        let mut expected = vec![None; self.owners.len() + plan.added()];''')
rep('''        let expected = word_owner_locations(&self.inner, None)
            .expect("a spliced inventory has a rebuild")''','''        let expected = guarded(|| word_owner_locations(&self.inner, None))
            .expect("a spliced inventory has a rebuild")''')
open(p,'w').write(s)
print("ok")
