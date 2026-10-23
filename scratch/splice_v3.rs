/// One direct body item that a splice inserts, with what the owner
/// inventory reads for it.
pub(crate) enum SplicedBodyItem {
    /// A plain paragraph's accepted text and run paths.
    Paragraph(String, Vec<rdocx_oxml::text::AcceptedRunPath>),
    /// A table's namespace-complete XML and its owner snapshots in a body
    /// that holds only that table.
    Table(Vec<u8>, WordOwnerSnapshots),
}

/// Plain direct body paragraphs removed from a body slot, or one item
/// inserted at it.
pub(crate) struct BodySplice {
    pub(crate) slot: usize,
    pub(crate) removed: usize,
    pub(crate) inserted: Option<SplicedBodyItem>,
}

/// A splice of the current owner inventory for one direct body edit.
///
/// The inserted item is one body story item, a plain paragraph or a table
/// with no nested table, text box or link, and the removed items are plain
/// paragraphs. So only body routes after the slot move, by the items inserted
/// less those removed, and only the cell stories after an inserted table,
/// by its cells. Every other owner keeps its place in the inventory.
pub(crate) struct BodySplicePlan {
    slot: usize,
    removed: usize,
    // The body story item index of the splice slot.
    first: usize,
    // The inserted owners, in blocks that each precede a current inventory
    // index, with next-inventory parents.
    blocks: Vec<(usize, Vec<WordOwnerAddress>)>,
    // The current inventory range of the removed paragraphs and their runs.
    removed_owners: std::ops::Range<usize>,
    // Cell story owners from the first index on move by the second.
    cells: (usize, usize),
    inserted_paragraph: bool,
    inserted_table: bool,
    // The facade paragraph and table indexes of the splice slot.
    pub(crate) paragraph: usize,
    pub(crate) table: usize,
    // The inserted table's route and the routes of its cell paragraphs.
    table_entry: Option<(rdocx::ContentLocation, Vec<CellParagraphs>)>,
}

/// The facade coordinates of a cell and the routes of its paragraphs.
type CellParagraphs = ((usize, usize), Vec<rdocx::ContentLocation>);

impl BodySplicePlan {
    fn added(&self) -> usize {
        self.blocks.iter().map(|(_, block)| block.len()).sum()
    }

    /// The body story item index that `item` moves to.
    fn moved(&self, item: usize) -> usize {
        if item >= self.first + self.removed {
            item + usize::from(self.inserted_paragraph || self.inserted_table) - self.removed
        } else {
            item
        }
    }

    /// The next inventory index of current owner `index`, unless removed.
    fn next_index(&self, index: usize) -> Option<usize> {
        if self.removed_owners.contains(&index) {
            return None;
        }
        let mut next = index;
        if index >= self.removed_owners.end {
            next -= self.removed_owners.len();
        }
        for (at, block) in &self.blocks {
            if index >= *at {
                next += block.len();
            }
        }
        Some(next)
    }

    /// The next inventory, in order: kept current owners and inserted ones.
    fn next_owners(&self, current: usize) -> impl Iterator<Item = NextWordOwner<'_>> + '_ {
        (0..=current).flat_map(move |index| {
            self.blocks
                .iter()
                .filter(move |(at, _)| *at == index)
                .flat_map(|(_, block)| block.iter().map(NextWordOwner::Inserted))
                .chain(
                    (index < current && !self.removed_owners.contains(&index))
                        .then_some(NextWordOwner::Kept(index)),
                )
        })
    }

    /// Move the cell stories of a kept owner's routes.
    fn move_cell_stories(&self, location: &mut WordOwnerLocation, part: &str) {
        let (first, count) = self.cells;
        if count == 0 {
            return;
        }
        let mut route = |location: &mut rdocx::ContentLocation| {
            let story = location.story();
            if story.kind() == rdocx::StoryKind::TableCell
                && story.part_name() == part
                && story.owner_index() >= first
            {
                let owner_index = story.owner_index() + count;
                location.set_story_owner_index(owner_index);
            }
        };
        match location {
            WordOwnerLocation::Paragraph(location)
            | WordOwnerLocation::Table(location)
            | WordOwnerLocation::Run(location, _)
            | WordOwnerLocation::Hyperlink(location, _) => route(location),
            WordOwnerLocation::Cell { table, story, .. } => {
                route(table);
                if story.kind() == rdocx::StoryKind::TableCell
                    && story.part_name() == part
                    && story.owner_index() >= first
                {
                    story.set_owner_index(story.owner_index() + count);
                }
            }
            WordOwnerLocation::Section(_) => {}
        }
    }
}

/// One owner of the next inventory.
enum NextWordOwner<'a> {
    Kept(usize),
    Inserted(&'a WordOwnerAddress),
}

/// A prepared body splice: its revision and the scope states of the next
/// inventory, in order.
pub(crate) struct PreparedBodySplice {
    pub(crate) revision: PreparedRevision,
    pub(crate) plan: BodySplicePlan,
    states: Vec<RevisionState>,
    // The index in `states` of the first inserted owner's scopes.
    inserted_state: usize,
    snapshot_scope: RevisionState,
}

/// The body story item index of an owner's content route.
fn word_owner_body_item(location: &WordOwnerLocation) -> Option<usize> {
    let route = match location {
        WordOwnerLocation::Paragraph(location)
        | WordOwnerLocation::Table(location)
        | WordOwnerLocation::Run(location, _)
        | WordOwnerLocation::Hyperlink(location, _) => location,
        WordOwnerLocation::Cell { table, .. } => table,
        WordOwnerLocation::Section(_) => return None,
    };
    (route.story().kind() == rdocx::StoryKind::Body)
        .then(|| route.index_path().first().copied())
        .flatten()
}

/// Move the body story item index of an owner's content route.
fn move_word_owner_body_item(location: &mut WordOwnerLocation, item: usize) {
    match location {
        WordOwnerLocation::Paragraph(location)
        | WordOwnerLocation::Table(location)
        | WordOwnerLocation::Run(location, _)
        | WordOwnerLocation::Hyperlink(location, _)
        | WordOwnerLocation::Cell {
            table: location, ..
        } => location.set_index_path_head(item),
        WordOwnerLocation::Section(_) => {}
    }
}

/// The story of an owner's content route.
fn word_owner_story(location: &WordOwnerLocation) -> Option<&rdocx::StoryId> {
    match location {
        WordOwnerLocation::Paragraph(location)
        | WordOwnerLocation::Table(location)
        | WordOwnerLocation::Run(location, _)
        | WordOwnerLocation::Hyperlink(location, _) => Some(location.story()),
        WordOwnerLocation::Cell { table, .. } => Some(table.story()),
        WordOwnerLocation::Section(_) => None,
    }
}

/// Whether `owners` has the order a rebuild gives it: tables each followed
/// by its cells, paragraphs each followed by its runs with the body story
/// first and in source order, then sections, then hyperlinks. A local edit
/// appends the runs it creates, which a later rebuild puts back in place.
fn word_inventory_is_canonical(owners: &[WordOwner]) -> bool {
    let mut family = 0;
    let mut table = None;
    let mut paragraph = None;
    let mut body_item = Some(0);
    for (index, owner) in owners.iter().enumerate() {
        let (next, parent) = match &owner.location {
            WordOwnerLocation::Table(_) => {
                table = Some(index);
                (0, owner.parent)
            }
            WordOwnerLocation::Cell { .. } => (0, table),
            WordOwnerLocation::Paragraph(location) => {
                paragraph = Some(index);
                if location.story().kind() == rdocx::StoryKind::Body {
                    let item = location.index_path().first().copied();
                    if body_item.is_none() || item < body_item {
                        return false;
                    }
                    body_item = item;
                } else {
                    body_item = None;
                }
                (1, owner.parent)
            }
            WordOwnerLocation::Run(..) => (1, paragraph),
            WordOwnerLocation::Section(_) => (2, None),
            WordOwnerLocation::Hyperlink(..) => (3, None),
        };
        let child = matches!(
            owner.location,
            WordOwnerLocation::Cell { .. } | WordOwnerLocation::Run(..)
        );
        if next < family || (child && (parent.is_none() || owner.parent != parent)) {
            return false;
        }
        family = next;
    }
    true
}

impl PyDocument {
    /// Plan `splice` from the current inventory alone. `None` keeps the
    /// rebuild for every case this does not prove, and debug builds check
    /// each committed splice against the rebuild.
    pub(crate) fn plan_body_splice(&self, splice: BodySplice) -> PyResult<Option<BodySplicePlan>> {
        let Some(body_paragraphs) = &self.indexes.body_paragraphs else {
            return Ok(None);
        };
        let content_count = self.inner.content_count();
        let Some(end_slot) = splice.slot.checked_add(splice.removed) else {
            return Ok(None);
        };
        let inserted_table = matches!(splice.inserted, Some(SplicedBodyItem::Table(..)));
        if end_slot > content_count
            || (splice.removed == 0) == splice.inserted.is_none()
            || body_paragraphs.iter().any(Option::is_none)
            || (inserted_table && self.indexes.body_tables.iter().any(Option::is_none))
            || !(splice.slot..end_slot)
                .all(|slot| self.inner.plain_direct_paragraph_without_section(slot))
            || !word_inventory_is_canonical(&self.owners)
        {
            return Ok(None);
        }
        note_fx260_word(1);
        // The body story, and the item index of every owned direct body slot.
        let mut story = None;
        let mut slot_items = HashMap::new();
        for owner in &self.owners {
            let (WordOwnerLocation::Paragraph(location) | WordOwnerLocation::Table(location)) =
                &owner.location
            else {
                continue;
            };
            if location.story().kind() != rdocx::StoryKind::Body {
                continue;
            }
            story.get_or_insert_with(|| location.story().clone());
            if let (Some(slot), Some(item)) = (owner.body_slot, location.index_path().first()) {
                slot_items.entry(slot).or_insert(*item);
            }
        }
        let Some(story) = story else {
            return Ok(None);
        };
        let part = story.part_name().to_owned();
        let first = if let Some(item) = slot_items.get(&splice.slot) {
            *item
        } else if splice.removed == 0 && splice.slot == content_count && splice.slot > 0 {
            let Some(previous) = slot_items.get(&(splice.slot - 1)) else {
                return Ok(None);
            };
            let Some(count) = self
                .inner
                .direct_body_story_item_count(splice.slot - 1)
                .ok()
                .flatten()
            else {
                return Ok(None);
            };
            previous + count
        } else {
            return Ok(None);
        };
        // Each removed slot is the direct paragraph owner at its own item.
        for offset in 0..splice.removed {
            let Some(owner) = self
                .indexes
                .direct_body_paragraphs
                .get(&(splice.slot + offset))
                .and_then(|index| self.owners.get(*index))
            else {
                return Ok(None);
            };
            match &owner.location {
                WordOwnerLocation::Paragraph(location)
                    if location.index_path() == [first + offset] => {}
                _ => return Ok(None),
            }
        }
        let Some(cells_before) = (if inserted_table {
            self.cell_stories_before(first, &part)
        } else {
            Some(0)
        }) else {
            return Ok(None);
        };
        let removed_items = first..first + splice.removed;
        let mut removed_owners = None::<std::ops::Range<usize>>;
        let mut paragraph_at = None;
        let mut table_at = None;
        let mut cells_at = None;
        let mut paragraph_ordinal = 0;
        let mut table_ordinal = 0;
        for (index, owner) in self.owners.iter().enumerate() {
            note_fx260_word(1);
            let item = word_owner_body_item(&owner.location);
            let story = word_owner_story(&owner.location);
            let body = story.is_some_and(|story| story.kind() == rdocx::StoryKind::Body);
            let (paragraph, table) = match &owner.location {
                WordOwnerLocation::Paragraph(_) => (true, false),
                WordOwnerLocation::Table(_) => (false, true),
                _ => (false, false),
            };
            let later = matches!(
                owner.location,
                WordOwnerLocation::Section(_) | WordOwnerLocation::Hyperlink(..)
            );
            if paragraph_at.is_none() && (later || (paragraph && !(body && item < Some(first)))) {
                paragraph_at = Some(index);
            }
            if table_at.is_none()
                && (later || paragraph || matches!(owner.location, WordOwnerLocation::Run(..))
                    || (table && !(body && item < Some(first))))
            {
                table_at = Some(index);
            }
            let later_cell_story = story.is_some_and(|story| {
                story.part_name() != part
                    || (story.kind() == rdocx::StoryKind::TableCell
                        && story.owner_index() >= cells_before)
            });
            if cells_at.is_none() && (later || (paragraph && later_cell_story)) {
                cells_at = Some(index);
            }
            if item.is_some_and(|item| removed_items.contains(&item)) {
                // Only the removed paragraphs and their runs, contiguously.
                let contiguous = removed_owners.as_ref().is_none_or(|range| range.end == index);
                if !contiguous
                    || !matches!(
                        owner.location,
                        WordOwnerLocation::Paragraph(_) | WordOwnerLocation::Run(..)
                    )
                {
                    return Ok(None);
                }
                removed_owners = Some(
                    removed_owners.map_or(index..index + 1, |range| range.start..index + 1),
                );
                continue;
            }
            if owner.parent.is_some_and(|parent| {
                removed_owners
                    .as_ref()
                    .is_some_and(|range| range.contains(&parent))
            }) {
                return Ok(None);
            }
            if body && item < Some(first) {
                paragraph_ordinal += usize::from(paragraph);
                table_ordinal += usize::from(table);
            }
        }
        if splice.removed > 0 && removed_owners.is_none() {
            return Ok(None);
        }
        let current = self.owners.len();
        // Facade paragraphs and body tables keep source order, so the splice
        // is one contiguous run of their entries.
        let head = |location: &Option<rdocx::ContentLocation>| {
            location
                .as_ref()
                .and_then(|location| location.index_path().first().copied())
        };
        let paragraph = body_paragraphs.partition_point(|location| head(location) < Some(first));
        for (offset, location) in body_paragraphs[paragraph..].iter().enumerate() {
            let fits = if offset < splice.removed {
                location.as_ref().map(rdocx::ContentLocation::index_path) == Some(&[first + offset])
            } else {
                head(location) >= Some(removed_items.end)
            };
            if !fits {
                return Ok(None);
            }
        }
        let table = self
            .indexes
            .body_tables
            .partition_point(|location| head(location) < Some(first));
        if self.indexes.body_tables.iter().any(|location| {
            location.as_ref().is_some_and(|location| {
                location.story().kind() == rdocx::StoryKind::Body
                    && removed_items.contains(&location.index_path()[0])
            })
        }) {
            return Ok(None);
        }
        let mut plan = BodySplicePlan {
            slot: splice.slot,
            removed: splice.removed,
            first,
            blocks: Vec::new(),
            removed_owners: removed_owners.unwrap_or(0..0),
            cells: (cells_before, 0),
            inserted_paragraph: false,
            inserted_table: false,
            paragraph,
            table,
            table_entry: None,
        };
        match splice.inserted {
            None => {}
            Some(SplicedBodyItem::Paragraph(text, runs)) => {
                let at = paragraph_at.unwrap_or(current);
                let location =
                    rdocx::ContentLocation::new(story, rdocx::StoryItemKind::Paragraph, vec![first]);
                let mut block = vec![WordOwnerAddress {
                    location: WordOwnerLocation::Paragraph(location.clone()),
                    parent: None,
                    body_slot: Some(splice.slot),
                    projection_ordinal: Some(paragraph_ordinal),
                    read_facts: WordReadFacts::Paragraph(text),
                }];
                block.extend(runs.into_iter().map(|path| WordOwnerAddress {
                    location: WordOwnerLocation::Run(location.clone(), path),
                    parent: Some(at),
                    body_slot: None,
                    projection_ordinal: None,
                    read_facts: WordReadFacts::None,
                }));
                plan.blocks.push((at, block));
                plan.inserted_paragraph = true;
            }
            Some(SplicedBodyItem::Table(xml, snapshots)) => {
                let table_at = table_at.unwrap_or(current);
                let cells_at = cells_at.unwrap_or(current);
                let Some((blocks, cells, entry)) = spliced_table_blocks(
                    xml,
                    snapshots,
                    &story,
                    (splice.slot, first, table_ordinal),
                    (table_at, cells_at, cells_before),
                ) else {
                    return Ok(None);
                };
                plan.blocks = blocks;
                plan.cells = (cells_before, cells);
                plan.inserted_table = true;
                plan.table_entry = Some(entry);
            }
        }
        Ok(Some(plan))
    }

    /// The number of cell stories of the main part before body story item
    /// `first`, when the inventory holds every cell story of that part and
    /// can place each one: no text box story, and every cell under a body
    /// table, so that cell story owner indexes count exactly those cells.
    fn cell_stories_before(&self, first: usize, part: &str) -> Option<usize> {
        let mut cells = Vec::new();
        let mut referenced = std::collections::HashSet::new();
        for owner in &self.owners {
            note_fx260_word(1);
            if let Some(story) = word_owner_story(&owner.location)
                && story.part_name() == part
            {
                match story.kind() {
                    rdocx::StoryKind::TextBox => return None,
                    rdocx::StoryKind::TableCell => {
                        referenced.insert(story.owner_index());
                    }
                    _ => {}
                }
            }
            let WordOwnerLocation::Cell { story, .. } = &owner.location else {
                continue;
            };
            if story.part_name() != part {
                continue;
            }
            // The body table that holds this cell, through nested tables.
            let mut table = owner.parent;
            let root = loop {
                let holder = self.owners.get(table?)?;
                match &holder.location {
                    WordOwnerLocation::Table(location)
                        if location.story().kind() == rdocx::StoryKind::Body =>
                    {
                        break location.index_path().first().copied()?;
                    }
                    WordOwnerLocation::Table(_) => {
                        table = self.owners.get(holder.parent?)?.parent;
                    }
                    _ => return None,
                }
            };
            cells.push((story.owner_index(), root));
        }
        cells.sort_unstable();
        let complete = cells
            .iter()
            .enumerate()
            .all(|(index, (owner_index, _))| index == *owner_index)
            && referenced.iter().all(|owner_index| *owner_index < cells.len());
        let before = cells.iter().filter(|(_, root)| *root < first).count();
        (complete && cells.iter().all(|(owner_index, root)| (*root < first) == (*owner_index < before)))
            .then_some(before)
    }

    /// Prepare a conservative or append publication of `plan`, with the
    /// retirements and births, in inventory order, that the rebuild would
    /// give, before the native edit.
    pub(crate) fn prepare_body_splice(
        &mut self,
        plan: BodySplicePlan,
        operation: &'static str,
        publication: &WordPublication,
    ) -> PyResult<PreparedBodySplice> {
        // An append keeps every owner. A conservative publication retires
        // every owner and gives the next inventory new scopes.
        let keep = match publication {
            WordPublication::AppendBody => true,
            WordPublication::Conservative => false,
            _ => return Err(PyValueError::new_err("body splice publication")),
        };
        let fail =
            |_| pyo3::exceptions::PyMemoryError::new_err("Word owner splice allocation failed");
        let current = self.owners.len();
        let scopes = |next: &NextWordOwner<'_>| match next {
            NextWordOwner::Kept(index) => self.owners[*index].location.scopes().len(),
            NextWordOwner::Inserted(address) => address.location.scopes().len(),
        };
        let kept_scopes = self
            .owners
            .iter()
            .map(|owner| owner.location.scopes().len())
            .sum::<usize>();
        let inserted_scopes = plan
            .blocks
            .iter()
            .flat_map(|(_, block)| block)
            .map(|address| address.location.scopes().len())
            .sum::<usize>();
        let mut births = Vec::new();
        births
            .try_reserve(inserted_scopes + if keep { 0 } else { kept_scopes } + 1)
            .map_err(fail)?;
        let mut retired = Vec::new();
        retired
            .try_reserve(if keep { 1 } else { kept_scopes + 1 })
            .map_err(fail)?;
        if !keep {
            retired.extend(self.owners.iter().flat_map(WordOwner::scope_states).cloned());
        }
        for next in plan.next_owners(current) {
            note_fx260_word(1);
            match next {
                NextWordOwner::Kept(_) if keep => {}
                NextWordOwner::Kept(index) => {
                    births.extend_from_slice(self.owners[index].location.scopes());
                }
                NextWordOwner::Inserted(address) => {
                    births.extend_from_slice(address.location.scopes());
                }
            }
        }
        retired.push(self.snapshot_scope.clone());
        births.push(RevisionScope::Text);
        let revision = self
            .revisions
            .prepare_scopes(operation, &retired, &births)
            .map_err(revision_to_pyerr)?;
        let mut states = Vec::new();
        states
            .try_reserve_exact(inserted_scopes + kept_scopes)
            .map_err(fail)?;
        let mut born = 0;
        let mut inserted_state = None;
        for next in plan.next_owners(current) {
            let count = scopes(&next);
            match next {
                NextWordOwner::Kept(index) if keep => {
                    states.extend(self.owners[index].scope_states().cloned());
                    continue;
                }
                NextWordOwner::Inserted(_) => {
                    inserted_state.get_or_insert(states.len());
                }
                NextWordOwner::Kept(_) => {}
            }
            for _ in 0..count {
                states.push(revision.born(born).expect("prepared owner birth"));
                born += 1;
            }
        }
        let snapshot_scope = revision.born(born).expect("prepared snapshot birth");
        // Capacity for the commit, which only moves and indexes owners.
        let added = plan.added();
        self.owners.try_reserve(added).map_err(fail)?;
        self.indexes
            .by_generation
            .try_reserve(added)
            .map_err(fail)?;
        self.indexes.by_route.try_reserve(added).map_err(fail)?;
        self.indexes
            .direct_body_paragraphs
            .try_reserve(added)
            .map_err(fail)?;
        if let Some(paragraphs) = &mut self.indexes.body_paragraphs {
            paragraphs.try_reserve(1).map_err(fail)?;
        }
        self.indexes.body_tables.try_reserve(1).map_err(fail)?;
        Ok(PreparedBodySplice {
            revision,
            plan,
            states,
            inserted_state: inserted_state.unwrap_or(0),
            snapshot_scope,
        })
    }

    /// Capture the first owner a prepared splice inserts, the paragraph or
    /// table it places, as [`Self::capture_prepared_owner`] captures a
    /// prepared owner.
    pub(crate) fn capture_prepared_splice_owner(
        &self,
        prepared: &PreparedBodySplice,
        segments: smallvec::SmallVec<[PathSeg; 5]>,
    ) -> PyResult<(ContentPath, RevisionState)> {
        self.revisions
            .capture_prepared(&prepared.revision, smallvec![])
            .map_err(revision_to_pyerr)?;
        let unbound = || PyValueError::new_err("element has no current checked physical owner");
        let address = prepared
            .plan
            .blocks
            .first()
            .and_then(|(_, block)| block.first())
            .filter(|address| address.parent.is_none())
            .ok_or_else(unbound)?;
        let at = prepared.inserted_state;
        let states = prepared
            .states
            .get(at..at + address.location.scopes().len())
            .ok_or_else(unbound)?;
        let owners = word_owners_with_states(
            vec![WordOwnerAddress {
                location: address.location.clone(),
                parent: None,
                body_slot: address.body_slot,
                projection_ordinal: address.projection_ordinal,
                read_facts: WordReadFacts::None,
            }],
            states,
        );
        let (mut path, owner) = word_owner_capture(
            &self.revisions,
            &owners,
            (prepared.revision.document_scope(), &self.removal_scope),
            0,
            segments,
            WordView::Owner,
            None,
        )?;
        path.revision = prepared.revision.committed_serial();
        Ok((path, owner))
    }

    /// Publish a prepared splice after its native edit: move the kept owners
    /// in place, place the inserted ones, and index the inventory again.
    pub(crate) fn commit_body_splice(
        &mut self,
        prepared: PreparedBodySplice,
        publication: &WordPublication,
    ) -> PyResult<()> {
        #[cfg(debug_assertions)]
        if matches!(publication, WordPublication::AppendBody) {
            self.assert_body_append_matches(&prepared.plan, publication);
        }
        #[cfg(not(debug_assertions))]
        let _ = publication;
        let PreparedBodySplice {
            revision,
            plan,
            states,
            snapshot_scope,
            ..
        } = prepared;
        self.revisions
            .commit_scopes(revision)
            .map_err(revision_to_pyerr)?;
        let part = self
            .owners
            .iter()
            .find_map(|owner| {
                word_owner_story(&owner.location)
                    .filter(|story| story.kind() == rdocx::StoryKind::Body)
                    .map(|story| story.part_name().to_owned())
            })
            .unwrap_or_default();
        let current = self.owners.len();
        // The scope states of each inserted owner, in inventory order.
        let mut inserted = Vec::with_capacity(plan.added());
        let mut cursor = 0;
        for next in plan.next_owners(current) {
            let (count, address) = match next {
                NextWordOwner::Kept(index) => (self.owners[index].location.scopes().len(), None),
                NextWordOwner::Inserted(address) => (address.location.scopes().len(), Some(address)),
            };
            if let Some(address) = address {
                inserted.extend(word_owners_with_states(
                    vec![WordOwnerAddress {
                        location: address.location.clone(),
                        parent: address.parent,
                        body_slot: address.body_slot,
                        projection_ordinal: address.projection_ordinal,
                        read_facts: address.read_facts.clone(),
                    }],
                    &states[cursor..cursor + count],
                ));
            }
            cursor += count;
        }
        self.owners.drain(plan.removed_owners.clone());
        let mut inserted = inserted.into_iter();
        let mut spans = Vec::with_capacity(plan.blocks.len());
        let mut placed = 0;
        for (at, block) in &plan.blocks {
            let start = *at + placed;
            self.owners
                .splice(start..start, inserted.by_ref().take(block.len()));
            spans.push(start..start + block.len());
            placed += block.len();
        }
        let after = plan.first + plan.removed;
        let paragraphs = usize::from(plan.inserted_paragraph) as isize - plan.removed as isize;
        let moved_ordinal = |ordinal: usize, delta: isize| ordinal.saturating_add_signed(delta);
        let mut cursor = 0;
        for (index, owner) in self.owners.iter_mut().enumerate() {
            note_fx260_word(1);
            if spans.iter().any(|span| span.contains(&index)) {
                cursor += owner.location.scopes().len();
                continue;
            }
            let item = word_owner_body_item(&owner.location);
            let body_route = matches!(
                &owner.location,
                WordOwnerLocation::Paragraph(location) | WordOwnerLocation::Table(location)
                    if location.story().kind() == rdocx::StoryKind::Body
            );
            if let Some(item) = item.filter(|item| *item >= after) {
                move_word_owner_body_item(&mut owner.location, plan.moved(item));
                if body_route {
                    let delta = match owner.location {
                        WordOwnerLocation::Paragraph(_) => paragraphs,
                        _ => isize::from(plan.inserted_table),
                    };
                    owner.projection_ordinal =
                        owner.projection_ordinal.map(|ordinal| moved_ordinal(ordinal, delta));
                }
            }
            if body_route {
                owner.body_slot = owner.body_slot.map(|slot| {
                    if slot >= plan.slot + plan.removed {
                        slot + usize::from(plan.inserted_paragraph || plan.inserted_table)
                            - plan.removed
                    } else {
                        slot
                    }
                });
            }
            plan.move_cell_stories(&mut owner.location, &part);
            owner.parent = owner.parent.map(|parent| {
                plan.next_index(parent)
                    .expect("a kept owner's parent is kept")
            });
            owner.owner = states[cursor].clone();
            cursor += 1;
            for state in [&mut owner.content, &mut owner.properties, &mut owner.grid]
                .into_iter()
                .flatten()
            {
                *state = states[cursor].clone();
                cursor += 1;
            }
        }
        self.indexes.index_owners_again(&self.owners);
        let moved = |location: &mut rdocx::ContentLocation| {
            if location.story().kind() == rdocx::StoryKind::Body {
                let item = location.index_path()[0];
                location.set_index_path_head(plan.moved(item));
            }
        };
        if let Some(paragraphs) = &mut self.indexes.body_paragraphs {
            paragraphs.drain(plan.paragraph..plan.paragraph + plan.removed);
            paragraphs.iter_mut().flatten().for_each(moved);
            if let Some((at, block)) = plan.blocks.first()
                && plan.inserted_paragraph
            {
                let WordOwnerLocation::Paragraph(location) = &block[0].location else {
                    unreachable!("an inserted paragraph block starts with its paragraph");
                };
                let _ = at;
                paragraphs.insert(plan.paragraph, Some(location.clone()));
            }
        }
        self.indexes.body_tables.iter_mut().flatten().for_each(moved);
        if let Some((location, cells)) = &plan.table_entry {
            self.indexes
                .body_tables
                .insert(plan.table, Some(location.clone()));
            let mut shifted = HashMap::with_capacity(self.indexes.cell_paragraphs.len() + cells.len());
            for ((table, row, cell), mut locations) in self.indexes.cell_paragraphs.drain() {
                for location in &mut locations {
                    let mut route = WordOwnerLocation::Paragraph(location.clone());
                    plan.move_cell_stories(&mut route, &part);
                    if let WordOwnerLocation::Paragraph(route) = route {
                        *location = route;
                    }
                }
                let table = if table >= plan.table { table + 1 } else { table };
                shifted.insert((table, row, cell), locations);
            }
            for ((row, cell), locations) in cells {
                shifted.insert((plan.table, *row, *cell), locations.clone());
            }
            self.indexes.cell_paragraphs = shifted;
        }
        self.snapshot_scope = snapshot_scope;
        #[cfg(debug_assertions)]
        self.assert_word_inventory_rebuilds();
        Ok(())
    }

    /// Under debug assertions, prove that an append keeps exactly the owners
    /// the rebuild's matcher keeps. Runs after the native edit.
    #[cfg(debug_assertions)]
    fn assert_body_append_matches(&self, plan: &BodySplicePlan, publication: &WordPublication) {
        #[cfg(test)]
        let visits = fx260_word_visits();
        let rebuilt =
            word_owner_locations(&self.inner, None).expect("a spliced inventory has a rebuild");
        let mut expected = vec![None; self.owners.len() + plan.added()];
        for index in 0..self.owners.len() {
            if let Some(next) = plan.next_index(index) {
                expected[next] = Some(index);
            }
        }
        assert_eq!(
            indexed_inventory_matches(&self.owners, &rebuilt, publication),
            expected,
            "spliced Word owner matches"
        );
        #[cfg(test)]
        FX260_WORD_VISITS.with(|cell| cell.set(visits));
    }
