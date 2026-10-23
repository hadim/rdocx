    pub(crate) fn current_layout_index(
        &self,
        path: &oxml_py_support::ContentPath,
    ) -> Option<usize> {
        self.owners.owner(path.owner_generation()?)?.address.layout_index
    }

    /// Build the candidate inventory, checking all survival proofs.
    /// It never infers that an ID still names the same incarnation.
    ///
    /// A slide whose native record the candidate still shares, and whose
    /// owners all survive unchanged, keeps its inventory part. Every other
    /// part is walked again, so the work follows the parts an edit touches.
    pub(crate) fn prepare_owners(
        &self,
        candidate: &rpptx::Presentation,
        operation: &'static str,
        preserved: &OwnerProofs,
        slides_changed: bool,
        layouts_changed: bool,
        masters_changed: bool,
    ) -> PyResult<PreparedOwners> {
        let changed = [slides_changed, layouts_changed, masters_changed];
        let carried = self.carried_slides(candidate, preserved);
        let prepared = self.prepare_parts(candidate, operation, preserved, &carried, changed);
        // Debug builds, which run the binding tests, check that carrying
        // parts over prepares exactly what walking every part prepares.
        #[cfg(debug_assertions)]
        if carried.iter().any(Option::is_some) {
            let walked = without_work_count(|| {
                let none = vec![None; carried.len()];
                self.prepare_parts(candidate, operation, preserved, &none, changed)
            });
            assert_same_preparation(&prepared, &walked);
        }
        prepared
    }

    /// For each candidate slide, the live slide part it carries over: one
    /// whose native record it shares, at the ordinal the proofs move that
    /// slide to, without an explicit proof. Any doubt walks every part.
    fn carried_slides(
        &self,
        candidate: &rpptx::Presentation,
        proofs: &OwnerProofs,
    ) -> Vec<Option<usize>> {
        let walk_all = vec![None; candidate.len()];
        // Shape ID counts stay local to a part while slide IDs are unique.
        let unique = |inner: &rpptx::Presentation| {
            let mut ids = HashSet::new();
            inner.slides().all(|slide| ids.insert(slide.id()))
        };
        if self.owners.parts.slides.len() != self.inner.len()
            || !unique(&self.inner)
            || !unique(candidate)
        {
            return walk_all;
        }
        let listed = proofs
            .parts
            .iter()
            .map(|(part, _)| *part)
            .collect::<HashSet<_>>();
        let carried = candidate
            .unchanged_slides_since(&self.inner)
            .into_iter()
            .enumerate()
            .map(|(at, origin)| {
                origin.filter(|from| {
                    proofs.edit.slide_index(*from) == Some(at)
                        && !listed.contains(&OwnerPart::Slide(*from))
                })
            })
            .collect::<Vec<_>>();
        // Explicit proofs must prove owners of their own part and land in a
        // part walked again, as every proof of a full walk does.
        for (part, list) in &proofs.parts {
            for proof in list {
                let own = self
                    .owners
                    .by_generation
                    .get(&proof.generation)
                    .is_some_and(|(at, _)| at == part);
                let into_carried = matches!(OwnerPart::of(&proof.destination),
                    OwnerPart::Slide(at) if carried.get(at).is_some_and(Option::is_some));
                if !own || into_carried {
                    return walk_all;
                }
            }
        }
        carried
    }

    fn prepare_parts(
        &self,
        candidate: &rpptx::Presentation,
        operation: &'static str,
        proofs: &OwnerProofs,
        carried: &[Option<usize>],
        [slides_changed, layouts_changed, masters_changed]: [bool; 3],
    ) -> PyResult<PreparedOwners> {
        use oxml_py_support::RevisionScope as S;
        let fail = |message: &str| pyo3::exceptions::PyValueError::new_err(message.to_owned());
        let live = &self.owners.parts;
        let mut kept = vec![false; live.slides.len()];
        for from in carried.iter().flatten() {
            kept[*from] = true;
        }
        // Live parts nothing carries over, with the proofs of their owners.
        let mut old_parts = Vec::new();
        for (part, owners) in live.iter() {
            if !matches!(part, OwnerPart::Slide(index) if kept[index]) {
                old_parts.push((part, owners, proofs.for_part(part, owners)?));
            }
        }
        // Candidate parts walked again, in inventory order.
        let mut new_parts = Vec::new();
        for (at, origin) in carried.iter().enumerate() {
            if origin.is_none() {
                let mut addresses = Vec::new();
                slide_owner_addresses(candidate, at, &mut addresses)?;
                new_parts.push((OwnerPart::Slide(at), addresses));
            }
        }
        let mut addresses = Vec::new();
        master_owner_addresses(candidate, &mut addresses)?;
        new_parts.push((OwnerPart::Masters, addresses));
        let mut destinations = HashMap::new();
        for (part_at, (_, addresses)) in new_parts.iter().enumerate() {
            destinations.try_reserve(addresses.len()).map_err(|_| {
                pyo3::exceptions::PyMemoryError::new_err("survival destinations allocation failed")
            })?;
            for (index, address) in addresses.iter().enumerate() {
                let key = (
                    address.path.clone(),
                    address.layout_index,
                    std::mem::discriminant(&address.physical),
                );
                destinations.entry(key).or_insert((part_at, index));
            }
        }
        let mut claims = new_parts
            .iter()
            .map(|(_, addresses)| vec![None; addresses.len()])
            .collect::<Vec<_>>();
        let mut seen = HashSet::new();
        let mut retained = Vec::new();
        let mut retained_ends = Vec::new();
        for (_, _, list) in &old_parts {
            for proof in list.iter() {
                note_fx260_ppt(1);
                if !seen.insert(proof.generation) {
                    return Err(fail("duplicate owner survival proof"));
                }
                let old = self
                    .owners
                    .owner(proof.generation)
                    .ok_or_else(|| fail("survival proof has no current owner"))?;
                let key = (
                    proof.destination.clone(),
                    proof.layout_index,
                    std::mem::discriminant(&old.address.physical),
                );
                let &(part_at, index) = destinations
                    .get(&key)
                    .ok_or_else(|| fail("survival destination is absent"))?;
                if claims[part_at][index].is_some() {
                    return Err(fail("two owners claim one destination"));
                }
                let next = &new_parts[part_at].1[index];
                if old.address.physical != next.physical || old.address.table != next.table {
                    return Err(fail("survival proof changes physical owner identity"));
                }
                claims[part_at][index] = Some(retained.len());
                retained.push((old, proof));
            }
            retained_ends.push(retained.len());
        }
        let next_len = carried
            .iter()
            .flatten()
            .map(|from| live.slides[*from].owners.len())
            .chain(new_parts.iter().map(|(_, addresses)| addresses.len()))
            .sum::<usize>();
        let structural = next_len != live.len()
            || carried
                .iter()
                .enumerate()
                .any(|(at, from)| from.is_some_and(|from| from != at))
            || retained.iter().any(|(old, proof)| {
                old.address.path != proof.destination
                    || old.address.layout_index != proof.layout_index
            });
        if structural {
            // A mutation cannot disambiguate imported missing/duplicate IDs by
            // merely finding the same number again in a candidate tree. The
            // candidate counts are the live ones with the walked parts swapped.
            let mut walked_ids = HashMap::<(PartIdentity, u32), (usize, usize)>::new();
            for (_, owners, _) in &old_parts {
                for (key, count) in &owners.shape_ids {
                    walked_ids.entry(key.clone()).or_default().0 += count;
                }
            }
            for (_, addresses) in &new_parts {
                for address in addresses {
                    if let PhysicalOwner::Shape {
                        slide,
                        id: Some(id),
                        ..
                    } = &address.physical
                    {
                        walked_ids.entry((slide.clone(), *id)).or_default().1 += 1;
                    }
                }
            }
            let mut walked = 0;
            for (part, owners) in live.iter() {
                if matches!(part, OwnerPart::Slide(index) if kept[index]) {
                    if let Some(refusal) = owners.refusal {
                        return Err(fail(refusal));
                    }
                    continue;
                }
                let start = walked.checked_sub(1).map_or(0, |at| retained_ends[at]);
                for (old, _) in &retained[start..retained_ends[walked]] {
                    if let PhysicalOwner::Shape { slide, id, kind } = &old.address.physical {
                        if *kind == rpptx::ShapeKind::AlternateContent {
                            continue;
                        }
                        let Some(id) = id else {
                            return Err(fail("structural owner proof requires a shape id"));
                        };
                        let key = (slide.clone(), *id);
                        let original = self.owners.shape_ids.get(&key).copied().unwrap_or(0);
                        let (before, after) = walked_ids.get(&key).copied().unwrap_or_default();
                        if original != 1 || original - before + after != 1 {
                            return Err(fail("structural owner proof has an ambiguous shape id"));
                        }
                    }
                }
                walked += 1;
            }
        }
        let mut affected = Vec::new();
        let walked_owners = old_parts
            .iter()
            .map(|(_, owners, _)| owners.owners.len())
            .sum::<usize>();
        let affected_capacity = walked_owners
            .checked_mul(4)
            .and_then(|count| count.checked_add(3))
            .ok_or_else(|| {
                pyo3::exceptions::PyOverflowError::new_err("owner retirement count overflow")
            })?;
        affected.try_reserve_exact(affected_capacity).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner retirement allocation failed")
        })?;
        for (_, owners, _) in &old_parts {
            for old in &owners.owners {
                if !seen.contains(&old.owner.generation()) {
                    affected.push(old.owner.clone());
                    affected.push(old.text.clone());
                    affected.push(old.xml_children.clone());
                    if let Some(grid) = &old.grid {
                        affected.push(grid.clone());
                    }
                }
            }
        }
        let mut successors = Vec::new();
        successors.try_reserve_exact(retained.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("owner replacements allocation failed")
        })?;
        for (old, proof) in &retained {
            let mut push = |state: &oxml_py_support::RevisionState| {
                affected.push(state.clone());
                affected.len() - 1
            };
            let xml_children = proof
                .xml_children_changed
                .then(|| push(&old.xml_children));
            let text = proof.text_changed.then(|| push(&old.text));
            let grid = if proof.grid_changed {
                Some(push(
                    old.grid
                        .as_ref()
                        .ok_or_else(|| fail("grid change on non-table"))?,
                ))
            } else {
                None
            };
            successors.push((xml_children, text, grid));
        }
        let slides_at = slides_changed.then(|| {
            let index = affected.len();
            affected.push(self.slides.clone());
            index
        });
        let layouts_at = layouts_changed.then(|| {
            let index = affected.len();
            affected.push(self.layouts.clone());
            index
        });
        let masters_at = masters_changed.then(|| {
            let index = affected.len();
            affected.push(self.masters.clone());
            index
        });
        let mut births = Vec::new();
        let walked_addresses = new_parts
            .iter()
            .map(|(_, addresses)| addresses.len())
            .sum::<usize>();
        births
            .try_reserve_exact(walked_addresses.checked_mul(4).ok_or_else(|| {
                pyo3::exceptions::PyOverflowError::new_err("owner birth count overflow")
            })?)
            .map_err(|_| {
                pyo3::exceptions::PyMemoryError::new_err("owner births allocation failed")
            })?;
        for (part_at, (_, addresses)) in new_parts.iter().enumerate() {
            for (index, address) in addresses.iter().enumerate() {
                if claims[part_at][index].is_none() {
                    births.extend([S::Owner, S::Text, S::Text]);
                    if address.table {
                        births.push(S::Grid);
                    }
                }
            }
        }
        let revision = self
            .revisions
            .prepare_scopes(operation, &affected, &births)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        let mut born = 0;
        let mut rebuilt = Vec::new();
        for (part_at, (part, addresses)) in new_parts.into_iter().enumerate() {
            let mut owners = Vec::new();
            owners.try_reserve_exact(addresses.len()).map_err(|_| {
                pyo3::exceptions::PyMemoryError::new_err(
                    "candidate owner inventory allocation failed",
                )
            })?;
            for (index, address) in addresses.into_iter().enumerate() {
                if let Some(at) = claims[part_at][index] {
                    let (old, _) = retained[at];
                    let (xml_children, text, grid) = successors[at];
                    let successor =
                        |at: usize| revision.successor(at).expect("affected scope successor exists");
                    let mut owner = old.clone();
                    owner.address = address;
                    if let Some(at) = text {
                        owner.text = successor(at);
                    }
                    if let Some(at) = grid {
                        owner.grid = Some(successor(at));
                    }
                    if let Some(at) = xml_children {
                        owner.xml_children = successor(at);
                    }
                    owners.push(owner);
                } else {
                    let start = born;
                    born += 3 + usize::from(address.table);
                    let grid = address
                        .table
                        .then(|| revision.born(start + 3).expect("table birth exists"));
                    owners.push(LiveOwner {
                        address,
                        owner: revision.born(start).expect("owner birth exists"),
                        text: revision.born(start + 1).expect("text birth exists"),
                        xml_children: revision.born(start + 2).expect("XML child birth exists"),
                        grid,
                    });
                }
            }
            rebuilt.push((part, Arc::new(PartOwners::new(owners)?)));
        }
        let mut parts = OwnerParts::default();
        let mut moved = Vec::new();
        let mut walked = rebuilt.iter();
        for (at, from) in carried.iter().enumerate() {
            parts.slides.push(match from {
                Some(from) if *from == at => Arc::clone(&live.slides[*from]),
                Some(from) => {
                    moved.push(OwnerPart::Slide(at));
                    Arc::new(live.slides[*from].moved_to(at)?)
                }
                None => Arc::clone(&walked.next().expect("walked slide part").1),
            });
        }
        parts.masters = Arc::clone(&walked.next().expect("walked master part").1);
        let slides = slides_at.map_or_else(
            || self.slides.clone(),
            |at| revision.successor(at).expect("inventory successor exists"),
        );
        let layouts = layouts_at.map_or_else(
            || self.layouts.clone(),
            |at| revision.successor(at).expect("inventory successor exists"),
        );
        let masters = masters_at.map_or_else(
            || self.masters.clone(),
            |at| revision.successor(at).expect("inventory successor exists"),
        );
        Ok(PreparedOwners {
            revision,
            owners: OwnerChanges::Replace(InventoryChange {
                parts,
                dropped: old_parts.iter().map(|(part, _, _)| *part).collect(),
                rebuilt: rebuilt.into_iter().map(|(part, _)| part).collect(),
                moved,
            }),
            slides,
            layouts,
            masters,
            #[cfg(debug_assertions)]
            retired: affected.iter().map(|state| state.generation()).collect(),
        })
    }

    /// No fallible action or Python allocation follows successful token commit.
    pub(crate) fn publish_owners(
        &mut self,
        candidate: rpptx::Presentation,
        prepared: PreparedOwners,
    ) -> PyResult<()> {
        self.commit_prepared(prepared)?;
        self.inner = candidate;
        Ok(())
    }

    pub(crate) fn commit_prepared(&mut self, prepared: PreparedOwners) -> PyResult<()> {
        match &prepared.owners {
            OwnerChanges::Replace(change) => self.owners.reserve(change)?,
            OwnerChanges::Append(owners) => {
                self.owners
                    .by_generation
                    .try_reserve(owners.len())
                    .map_err(|_| {
                        pyo3::exceptions::PyMemoryError::new_err(
                            "PowerPoint owner index allocation failed",
                        )
                    })?;
                self.owners.shape_ids.try_reserve(1).map_err(|_| {
                    pyo3::exceptions::PyMemoryError::new_err(
                        "PowerPoint owner index allocation failed",
                    )
                })?;
            }
        }
        self.revisions
            .commit_scopes(prepared.revision)
            .map_err(oxml_py_support::revision_to_pyerr)?;
        match prepared.owners {
            OwnerChanges::Replace(change) => self.owners.apply(change),
            OwnerChanges::Append(owners) => {
                let Some(&PathSeg::Slide(slide)) = owners[0].address.path.first() else {
                    unreachable!("appended owners belong to a slide");
                };
                let part = OwnerPart::Slide(slide);
                let owners_part = self
                    .owners
                    .parts
                    .part_mut(part)
                    .expect("appended slide part");
                for owner in owners {
                    let index = owners_part.owners.len();
                    if let PhysicalOwner::Shape {
                        slide,
                        id: Some(id),
                        ..
                    } = &owner.address.physical
                    {
                        *self
                            .owners
                            .shape_ids
                            .entry((slide.clone(), *id))
                            .or_default() += 1;
                    }
                    self.owners
                        .by_generation
                        .insert(owner.owner.generation(), (part, index));
                    owners_part.owners.push(owner);
                    owners_part.index(index);
                }
                owners_part.refusal = owners_part.structural_refusal();
            }
        }
        self.slides = prepared.slides;
        self.layouts = prepared.layouts;
        self.masters = prepared.masters;
        Ok(())
    }

