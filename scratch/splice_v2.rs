/// Plain direct body paragraphs removed from a body slot, or one inserted at
/// it with its accepted text and run paths.
pub(crate) struct PlainBodySplice {
    slot: usize,
    removed: usize,
    inserted: Option<(String, Vec<rdocx_oxml::text::AcceptedRunPath>)>,
}

/// A plain body splice of the current owner inventory.
///
/// A plain direct paragraph is one body story item and owns no other story,
/// so only body routes after the splice move, by the paragraphs inserted
/// less those removed, and the inventory keeps every other owner in place.
pub(crate) struct PlainBodyPlan {
    slot: usize,
    removed: usize,
    // The body story item index of the splice slot.
    first: usize,
    // The route of the inserted paragraph.
    location: rdocx::ContentLocation,
    inserted: Option<(String, Vec<rdocx_oxml::text::AcceptedRunPath>)>,
    // The current inventory range of the removed paragraphs and their runs.
    removed_owners: std::ops::Range<usize>,
    // The current inventory index that the inserted owners precede.
    insert_at: usize,
    ordinal: usize,
    // The facade paragraph index of the splice slot.
    pub(crate) paragraph: usize,
}

impl PlainBodyPlan {
    fn added(&self) -> usize {
        self.inserted.as_ref().map_or(0, |(_, runs)| runs.len() + 1)
    }

    /// The body story item index that `item` moves to.
    fn moved(&self, item: usize) -> usize {
        if item >= self.first + self.removed {
            item + usize::from(self.inserted.is_some()) - self.removed
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
        if index >= self.insert_at {
            next += self.added();
        }
        Some(next)
    }

    /// The scopes of the inserted owners, paragraph first.
    fn inserted_scopes(&self) -> impl Iterator<Item = RevisionScope> + '_ {
        self.inserted.iter().flat_map(|(_, runs)| {
            WordOwnerLocation::Paragraph(self.location.clone())
                .scopes()
                .iter()
                .copied()
                .chain(runs.iter().flat_map(|_| {
                    [RevisionScope::Owner, RevisionScope::Text]
                }))
        })
    }
}

/// A prepared plain body splice: its revision and the scope states of the
/// next inventory, in order.
pub(crate) struct PreparedPlainBodySplice {
    pub(crate) revision: PreparedRevision,
    pub(crate) plan: PlainBodyPlan,
    states: Vec<RevisionState>,
    // The index in `states` of the inserted paragraph's owner scope.
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
    pub(crate) fn plan_plain_body_splice(
        &self,
        splice: PlainBodySplice,
    ) -> PyResult<Option<PlainBodyPlan>> {
        let Some(body_paragraphs) = &self.indexes.body_paragraphs else {
            return Ok(None);
        };
        let content_count = self.inner.content_count();
        let Some(end_slot) = splice.slot.checked_add(splice.removed) else {
            return Ok(None);
        };
        if end_slot > content_count
            || (splice.removed == 0) == splice.inserted.is_none()
            || body_paragraphs.iter().any(Option::is_none)
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
        let removed_items = first..first + splice.removed;
        let mut removed_owners = None::<std::ops::Range<usize>>;
        let mut insert_at = None;
        let mut ordinal = 0;
        for (index, owner) in self.owners.iter().enumerate() {
            note_fx260_word(1);
            let item = word_owner_body_item(&owner.location);
            let body_paragraph = matches!(
                &owner.location,
                WordOwnerLocation::Paragraph(location)
                    if location.story().kind() == rdocx::StoryKind::Body
            );
            let later = match &owner.location {
                WordOwnerLocation::Paragraph(_) => !body_paragraph || item >= Some(first),
                WordOwnerLocation::Section(_) | WordOwnerLocation::Hyperlink(..) => true,
                _ => false,
            };
            if later && insert_at.is_none() {
                insert_at = Some(index);
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
                removed_owners = Some(removed_owners.map_or(index..index + 1, |range| {
                    range.start..index + 1
                }));
                continue;
            }
            if owner
                .parent
                .is_some_and(|parent| removed_owners.as_ref().is_some_and(|range| range.contains(&parent)))
            {
                return Ok(None);
            }
            if body_paragraph && item < Some(first) {
                ordinal += 1;
            }
        }
        if splice.removed > 0 && removed_owners.is_none() {
            return Ok(None);
        }
        // Facade paragraphs and body tables keep source order, so the splice
        // is one contiguous run of their entries.
        let paragraph = body_paragraphs.partition_point(|location| {
            location
                .as_ref()
                .and_then(|location| location.index_path().first())
                .is_some_and(|item| *item < first)
        });
        for (offset, location) in body_paragraphs[paragraph..].iter().enumerate() {
            let Some(location) = location else {
                return Ok(None);
            };
            let fits = if offset < splice.removed {
                location.index_path() == [first + offset]
            } else {
                location.index_path()[0] >= removed_items.end
            };
            if !fits {
                return Ok(None);
            }
        }
        if self.indexes.body_tables.iter().flatten().any(|location| {
            location.story().kind() == rdocx::StoryKind::Body
                && removed_items.contains(&location.index_path()[0])
        }) {
            return Ok(None);
        }
        Ok(Some(PlainBodyPlan {
            slot: splice.slot,
            removed: splice.removed,
            first,
            location: rdocx::ContentLocation::new(
                story,
                rdocx::StoryItemKind::Paragraph,
                vec![first],
            ),
            inserted: splice.inserted,
            removed_owners: removed_owners.unwrap_or(0..0),
            insert_at: insert_at.unwrap_or(self.owners.len()),
            ordinal,
            paragraph,
        }))
    }

    /// Prepare a conservative or append publication of `plan`, with the
    /// retirements and births, in inventory order, that the rebuild would
    /// give, before the native edit.
    pub(crate) fn prepare_plain_body_splice(
        &mut self,
        plan: PlainBodyPlan,
        operation: &'static str,
        publication: &WordPublication,
    ) -> PyResult<PreparedPlainBodySplice> {
        // An append keeps every owner. A conservative publication retires
        // every owner and gives the next inventory new scopes.
        let keep = match publication {
            WordPublication::AppendBody => true,
            WordPublication::Conservative => false,
            _ => return Err(PyValueError::new_err("plain body splice publication")),
        };
        let fail =
            |_| pyo3::exceptions::PyMemoryError::new_err("Word owner splice allocation failed");
        let current = self
            .owners
            .iter()
            .map(|owner| owner.location.scopes().len())
            .sum::<usize>();
        let inserted = plan.inserted_scopes().count();
        let mut births = Vec::new();
        births
            .try_reserve(if keep { inserted } else { current + inserted } + 1)
            .map_err(fail)?;
        let mut retired = Vec::new();
        retired
            .try_reserve(if keep { 1 } else { current + 1 })
            .map_err(fail)?;
        for (index, owner) in self.owners.iter().enumerate() {
            note_fx260_word(1);
            if index == plan.insert_at {
                births.extend(plan.inserted_scopes());
            }
            if !keep {
                retired.extend(owner.scope_states().cloned());
                if !plan.removed_owners.contains(&index) {
                    births.extend_from_slice(owner.location.scopes());
                }
            }
        }
        if plan.insert_at == self.owners.len() {
            births.extend(plan.inserted_scopes());
        }
        retired.push(self.snapshot_scope.clone());
        births.push(RevisionScope::Text);
        let revision = self
            .revisions
            .prepare_scopes(operation, &retired, &births)
            .map_err(revision_to_pyerr)?;
        let mut states = Vec::new();
        states
            .try_reserve_exact(births.len() - 1 + if keep { current } else { 0 })
            .map_err(fail)?;
        let mut born = 0;
        let mut birth = |states: &mut Vec<RevisionState>, count: usize| {
            for _ in 0..count {
                states.push(revision.born(born).expect("prepared owner birth"));
                born += 1;
            }
        };
        let mut inserted_state = 0;
        for (index, owner) in self.owners.iter().enumerate() {
            if index == plan.insert_at {
                inserted_state = states.len();
                birth(&mut states, inserted);
            }
            if plan.removed_owners.contains(&index) {
                continue;
            }
            if keep {
                states.extend(owner.scope_states().cloned());
            } else {
                birth(&mut states, owner.location.scopes().len());
            }
        }
        if plan.insert_at == self.owners.len() {
            inserted_state = states.len();
            birth(&mut states, inserted);
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
        Ok(PreparedPlainBodySplice {
            revision,
            plan,
            states,
            inserted_state,
            snapshot_scope,
        })
    }

    /// Capture the inserted paragraph of a prepared splice, as
    /// [`Self::capture_prepared_owner`] captures a prepared owner.
    pub(crate) fn capture_prepared_splice_paragraph(
        &self,
        prepared: &PreparedPlainBodySplice,
        segments: smallvec::SmallVec<[PathSeg; 5]>,
    ) -> PyResult<(ContentPath, RevisionState)> {
        self.revisions
            .capture_prepared(&prepared.revision, smallvec![])
            .map_err(revision_to_pyerr)?;
        let plan = &prepared.plan;
        let at = prepared.inserted_state;
        let (Some(_), Some([owner, content, properties])) = (
            &plan.inserted,
            prepared.states.get(at..at + 3).map(|states| [&states[0], &states[1], &states[2]]),
        ) else {
            return Err(PyValueError::new_err(
                "element has no current checked physical owner",
            ));
        };
        let paragraph = WordOwner {
            location: WordOwnerLocation::Paragraph(plan.location.clone()),
            parent: None,
            body_slot: Some(plan.slot),
            projection_ordinal: Some(plan.ordinal),
            read_facts: WordReadFacts::None,
            owner: owner.clone(),
            content: Some(content.clone()),
            properties: Some(properties.clone()),
            grid: None,
        };
        let (mut path, owner) = word_owner_capture(
            &self.revisions,
            std::slice::from_ref(&paragraph),
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
    pub(crate) fn commit_plain_body_splice(
        &mut self,
        prepared: PreparedPlainBodySplice,
        publication: &WordPublication,
    ) -> PyResult<()> {
        #[cfg(debug_assertions)]
        if matches!(publication, WordPublication::AppendBody) {
            self.assert_plain_body_append_matches(&prepared.plan, publication);
        }
        #[cfg(not(debug_assertions))]
        let _ = publication;
        let PreparedPlainBodySplice {
            revision,
            plan,
            states,
            inserted_state,
            snapshot_scope,
        } = prepared;
        self.revisions
            .commit_scopes(revision)
            .map_err(revision_to_pyerr)?;
        let mut block = Vec::with_capacity(plan.added());
        if let Some((text, runs)) = &plan.inserted {
            let state = |offset: usize| states[inserted_state + offset].clone();
            block.push(WordOwner {
                location: WordOwnerLocation::Paragraph(plan.location.clone()),
                parent: None,
                body_slot: Some(plan.slot),
                projection_ordinal: Some(plan.ordinal),
                read_facts: WordReadFacts::Paragraph(text.clone()),
                owner: state(0),
                content: Some(state(1)),
                properties: Some(state(2)),
                grid: None,
            });
            let next = plan.insert_at;
            for (offset, path) in runs.iter().enumerate() {
                block.push(WordOwner {
                    location: WordOwnerLocation::Run(plan.location.clone(), path.clone()),
                    parent: Some(next),
                    body_slot: None,
                    projection_ordinal: None,
                    read_facts: WordReadFacts::None,
                    owner: state(3 + 2 * offset),
                    content: Some(state(4 + 2 * offset)),
                    properties: None,
                    grid: None,
                });
            }
        }
        let inserted = block.len();
        self.owners.drain(plan.removed_owners.clone());
        let insert_at = plan.insert_at;
        self.owners.splice(insert_at..insert_at, block);
        let after = plan.first + plan.removed;
        let moved_slot = |slot: usize| slot + usize::from(plan.inserted.is_some()) - plan.removed;
        let mut cursor = 0;
        for (index, owner) in self.owners.iter_mut().enumerate() {
            note_fx260_word(1);
            if (insert_at..insert_at + inserted).contains(&index) {
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
                if matches!(owner.location, WordOwnerLocation::Paragraph(_)) {
                    owner.projection_ordinal = owner.projection_ordinal.map(moved_slot);
                }
            }
            if body_route {
                owner.body_slot = owner.body_slot.map(|slot| {
                    if slot >= plan.slot + plan.removed {
                        moved_slot(slot)
                    } else {
                        slot
                    }
                });
            }
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
        if let Some(paragraphs) = &mut self.indexes.body_paragraphs {
            paragraphs.drain(plan.paragraph..plan.paragraph + plan.removed);
            if plan.inserted.is_some() {
                paragraphs.insert(plan.paragraph, Some(plan.location.clone()));
            }
            let moved = plan.paragraph + usize::from(plan.inserted.is_some());
            for location in paragraphs[moved..].iter_mut().flatten() {
                let item = location.index_path()[0];
                location.set_index_path_head(plan.moved(item));
            }
        }
        for location in self.indexes.body_tables.iter_mut().flatten() {
            if location.story().kind() == rdocx::StoryKind::Body {
                let item = location.index_path()[0];
                location.set_index_path_head(plan.moved(item));
            }
        }
        self.snapshot_scope = snapshot_scope;
        #[cfg(debug_assertions)]
        self.assert_word_inventory_rebuilds();
        Ok(())
    }

    /// Under debug assertions, prove that an append keeps exactly the owners
    /// the rebuild's matcher keeps. Runs after the native edit.
    #[cfg(debug_assertions)]
    fn assert_plain_body_append_matches(&self, plan: &PlainBodyPlan, publication: &WordPublication) {
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

    /// Under debug assertions, prove the inventory and its document indexes
    /// equal their rebuild from the current document, story fingerprints
    /// aside, which local edits leave stale in the inventory too.
    #[cfg(debug_assertions)]
    fn assert_word_inventory_rebuilds(&self) {
        #[cfg(test)]
        let visits = fx260_word_visits();
        let address = |location: &WordOwnerLocation,
                       parent: Option<usize>,
                       body_slot: Option<usize>,
                       projection_ordinal: Option<usize>,
                       read_facts: &WordReadFacts| WordOwnerAddress {
            location: unfingerprinted_owner_location(location),
            parent,
            body_slot,
            projection_ordinal,
            read_facts: read_facts.clone(),
        };
        let actual = self
            .owners
            .iter()
            .map(|owner| {
                address(
                    &owner.location,
                    owner.parent,
                    owner.body_slot,
                    owner.projection_ordinal,
                    &owner.read_facts,
                )
            })
            .collect::<Vec<_>>();
        let expected = word_owner_locations(&self.inner, None)
            .expect("a spliced inventory has a rebuild")
            .iter()
            .map(|owner| {
                address(
                    &owner.location,
                    owner.parent,
                    owner.body_slot,
                    owner.projection_ordinal,
                    &owner.read_facts,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "spliced Word owner inventory");
        let indexes = WordOwnerIndexes::prepare(&self.owners, &self.inner)
            .expect("a spliced inventory has rebuilt indexes");
        let locations = |locations: &[Option<rdocx::ContentLocation>]| {
            locations
                .iter()
                .map(|location| location.as_ref().map(unfingerprinted_location))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            self.indexes.body_paragraphs.as_deref().map(locations),
            indexes.body_paragraphs.as_deref().map(locations),
            "spliced body paragraph index"
        );
        assert_eq!(
            locations(&self.indexes.body_tables),
            locations(&indexes.body_tables),
            "spliced body table index"
        );
        let cells = |cells: &HashMap<(usize, usize, usize), Vec<rdocx::ContentLocation>>| {
            cells
                .iter()
                .map(|(key, locations)| {
                    (
                        *key,
                        locations
                            .iter()
                            .map(unfingerprinted_location)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        assert_eq!(
            cells(&self.indexes.cell_paragraphs),
            cells(&indexes.cell_paragraphs),
            "spliced cell paragraph index"
        );
        assert_eq!(
            self.indexes.direct_body_paragraphs, indexes.direct_body_paragraphs,
            "spliced direct body paragraph index"
        );
        #[cfg(test)]
        FX260_WORD_VISITS.with(|cell| cell.set(visits));
    }
}

