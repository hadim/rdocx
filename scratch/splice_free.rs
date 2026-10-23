/// Plain direct body paragraphs removed from a body slot, or one inserted at
/// it with its accepted text and run paths.
pub(crate) struct PlainBodySplice {
    slot: usize,
    removed: usize,
    inserted: Option<(String, Vec<rdocx_oxml::text::AcceptedRunPath>)>,
}

/// The owner inventory after a plain body splice, derived from the current
/// one instead of a rebuild of the edited body.
pub(crate) struct SplicedWordInventory {
    addresses: Vec<WordOwnerAddress>,
    // The current inventory index of each owner the splice keeps.
    previous: Vec<Option<usize>>,
    body_paragraphs: Vec<Option<rdocx::ContentLocation>>,
    body_tables: Vec<Option<rdocx::ContentLocation>>,
    // The facade paragraph index of the splice slot.
    paragraph: usize,
}

/// The direct body content route of an owner, as its body story item index.
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

/// `location` with its body story item index moved from `from` to `to`.
fn shifted_body_location(
    location: &rdocx::ContentLocation,
    to: usize,
) -> rdocx::ContentLocation {
    let mut path = location.index_path().to_vec();
    path[0] = to;
    rdocx::ContentLocation::new(location.story().clone(), location.item_kind(), path)
}

fn shifted_word_owner_location(location: &WordOwnerLocation, to: usize) -> WordOwnerLocation {
    match location {
        WordOwnerLocation::Paragraph(location) => {
            WordOwnerLocation::Paragraph(shifted_body_location(location, to))
        }
        WordOwnerLocation::Table(location) => {
            WordOwnerLocation::Table(shifted_body_location(location, to))
        }
        WordOwnerLocation::Run(location, path) => {
            WordOwnerLocation::Run(shifted_body_location(location, to), path.clone())
        }
        WordOwnerLocation::Hyperlink(location, index) => {
            WordOwnerLocation::Hyperlink(shifted_body_location(location, to), *index)
        }
        WordOwnerLocation::Cell {
            table,
            row,
            cell,
            story,
        } => WordOwnerLocation::Cell {
            table: shifted_body_location(table, to),
            row: *row,
            cell: *cell,
            story: story.clone(),
        },
        WordOwnerLocation::Section(index) => WordOwnerLocation::Section(*index),
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
    /// The owner inventory after `splice`, from the current inventory alone.
    ///
    /// A plain direct paragraph is one body story item and owns no other
    /// story, so only body routes at and after the slot move, by the number of
    /// paragraphs inserted less those removed. `None` keeps the rebuild for
    /// every case this does not prove, and debug builds check the result
    /// against the rebuild after each edit.
    pub(crate) fn splice_plain_body(
        &self,
        py: Python<'_>,
        splice: PlainBodySplice,
    ) -> PyResult<Option<SplicedWordInventory>> {
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
        // Every owned direct body slot, by its body story item index.
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
                .map_err(|error| rdocx_to_pyerr(py, error))?
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
        let inserted = usize::from(splice.inserted.is_some());
        let moved = |item: usize| item + inserted - splice.removed;
        let paragraph_location =
            rdocx::ContentLocation::new(story, rdocx::StoryItemKind::Paragraph, vec![first]);
        let fail =
            |_| pyo3::exceptions::PyMemoryError::new_err("Word owner splice allocation failed");
        let mut addresses = Vec::new();
        let mut previous = Vec::new();
        let mut current = Vec::new();
        let added = splice.inserted.as_ref().map_or(0, |(_, runs)| runs.len() + 1);
        addresses
            .try_reserve(self.owners.len() + added)
            .map_err(fail)?;
        previous
            .try_reserve(self.owners.len() + added)
            .map_err(fail)?;
        current.try_reserve(self.owners.len()).map_err(fail)?;
        let mut pending = splice.inserted;
        let mut ordinal = 0;
        let mut place = |addresses: &mut Vec<WordOwnerAddress>,
                         previous: &mut Vec<Option<usize>>,
                         pending: &mut Option<(String, Vec<_>)>,
                         ordinal: usize| {
            let Some((text, runs)) = pending.take() else {
                return;
            };
            let parent = addresses.len();
            addresses.push(WordOwnerAddress {
                location: WordOwnerLocation::Paragraph(paragraph_location.clone()),
                parent: None,
                body_slot: Some(splice.slot),
                projection_ordinal: Some(ordinal),
                read_facts: WordReadFacts::Paragraph(text),
            });
            previous.push(None);
            for path in runs {
                addresses.push(WordOwnerAddress {
                    location: WordOwnerLocation::Run(paragraph_location.clone(), path),
                    parent: Some(parent),
                    body_slot: None,
                    projection_ordinal: None,
                    read_facts: WordReadFacts::None,
                });
                previous.push(None);
            }
        };
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
            if later {
                place(&mut addresses, &mut previous, &mut pending, ordinal);
            }
            if item.is_some_and(|item| removed_items.contains(&item)) {
                if !matches!(
                    owner.location,
                    WordOwnerLocation::Paragraph(_) | WordOwnerLocation::Run(..)
                ) {
                    return Ok(None);
                }
                current.push(None);
                continue;
            }
            let after = item.is_some_and(|item| item >= removed_items.end);
            if body_paragraph && !after {
                ordinal += 1;
            }
            let parent = match owner.parent {
                Some(parent) => match current.get(parent).copied().flatten() {
                    Some(parent) => Some(parent),
                    None => return Ok(None),
                },
                None => None,
            };
            let body_slot = match (&owner.location, owner.body_slot) {
                (WordOwnerLocation::Paragraph(location) | WordOwnerLocation::Table(location), Some(slot))
                    if location.story().kind() == rdocx::StoryKind::Body && slot >= end_slot =>
                {
                    Some(slot + inserted - splice.removed)
                }
                (_, slot) => slot,
            };
            current.push(Some(addresses.len()));
            previous.push(Some(index));
            addresses.push(WordOwnerAddress {
                location: match item {
                    Some(item) if after => shifted_word_owner_location(&owner.location, moved(item)),
                    _ => owner.location.clone(),
                },
                parent,
                body_slot,
                projection_ordinal: match owner.projection_ordinal {
                    Some(ordinal) if body_paragraph && after => {
                        Some(ordinal + inserted - splice.removed)
                    }
                    ordinal => ordinal,
                },
                read_facts: owner.read_facts.clone(),
            });
        }
        place(&mut addresses, &mut previous, &mut pending, ordinal);
        // Facade paragraphs and body tables keep source order, so the splice
        // is one contiguous run of entries.
        let paragraph = body_paragraphs.partition_point(|location| {
            location
                .as_ref()
                .and_then(|location| location.index_path().first())
                .is_some_and(|item| *item < first)
        });
        let mut paragraphs = Vec::new();
        paragraphs
            .try_reserve(body_paragraphs.len() + inserted)
            .map_err(fail)?;
        paragraphs.extend(body_paragraphs[..paragraph].iter().cloned());
        if inserted == 1 {
            paragraphs.push(Some(paragraph_location.clone()));
        }
        for (offset, location) in body_paragraphs[paragraph..].iter().enumerate() {
            let Some(location) = location else {
                return Ok(None);
            };
            let item = location.index_path()[0];
            if offset < splice.removed {
                if location.index_path() != [first + offset] {
                    return Ok(None);
                }
                continue;
            }
            if item < removed_items.end {
                return Ok(None);
            }
            paragraphs.push(Some(shifted_body_location(location, moved(item))));
        }
        let mut tables = Vec::new();
        tables
            .try_reserve(self.indexes.body_tables.len())
            .map_err(fail)?;
        for location in &self.indexes.body_tables {
            tables.push(match location {
                Some(location) if location.story().kind() == rdocx::StoryKind::Body => {
                    let item = location.index_path()[0];
                    if removed_items.contains(&item) {
                        return Ok(None);
                    }
                    Some(if item >= removed_items.end {
                        shifted_body_location(location, moved(item))
                    } else {
                        location.clone()
                    })
                }
                other => other.clone(),
            });
        }
        Ok(Some(SplicedWordInventory {
            addresses,
            previous,
            body_paragraphs: paragraphs,
            body_tables: tables,
            paragraph,
        }))
    }
}
