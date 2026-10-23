    /// Footer staging only removes owned footer shapes, appends fresh copies,
    /// and preserves every surviving physical identity. Match that explicit
    /// native guarantee and compare complete text owners before publication.
    pub(crate) fn footer_owners(
        &self,
        candidate: &rpptx::Presentation,
    ) -> PyResult<OwnerProofs> {
        use oxml_py_support::ContentPath;
        let next = owner_addresses(candidate)?;
        let mut by_physical = HashMap::<&PhysicalOwner, smallvec::SmallVec<[&OwnerAddress; 1]>>::new();
        by_physical.try_reserve(next.len()).map_err(|_| {
            pyo3::exceptions::PyMemoryError::new_err("footer owner allocation failed")
        })?;
        for address in &next {
            by_physical.entry(&address.physical).or_default().push(address);
        }
        let mut changed_bodies = Vec::<smallvec::SmallVec<[PathSeg; 5]>>::new();
        let every_part = self.prefix_parts(&[]);
        self.part_owner_proofs(every_part, |old, proof| {
            // The native footer provider retains the body but replaces its
            // text children. Earlier nodes do not survive by ordinal.
            if matches!(
                old.address.physical,
                PhysicalOwner::Paragraph { .. } | PhysicalOwner::Run { .. }
            ) && changed_bodies
                .iter()
                .any(|base| old.address.path.starts_with(base))
            {
                return Ok(false);
            }
            let after = match by_physical.get(&old.address.physical).map(|found| found.as_slice()) {
                None | Some([]) => return Ok(false),
                Some([after]) => *after,
                Some(_) => {
                    return Err(PyValueError::new_err("footer owner identity is ambiguous"));
                }
            };
            let before_path = ContentPath::new(old.address.path.clone(), self.revisions.current());
            let after_path = ContentPath::new(after.path.clone(), self.revisions.current());
            let text_changed = match &old.address.physical {
                PhysicalOwner::Shape { .. } => {
                    crate::shape::native_shape_ref_at(&self.inner, &before_path)
                        .and_then(|shape| shape.text_frame())
                        != crate::shape::native_shape_ref_at(candidate, &after_path)
                            .and_then(|shape| shape.text_frame())
                }
                PhysicalOwner::TextBody { cell, .. } => {
                    let frame = |inner, path| {
                        let shape = crate::shape::native_shape_ref_at(inner, path)?;
                        match cell {
                            Some((row, col)) => shape.table()?.cell(*row, *col)?.text_frame(),
                            None => shape.text_frame(),
                        }
                    };
                    frame(&self.inner, &before_path) != frame(candidate, &after_path)
                }
                // Footer publication does not change notes or any table cell.
                PhysicalOwner::Slide(_)
                | PhysicalOwner::Cell { .. }
                | PhysicalOwner::Layout(_)
                | PhysicalOwner::Master(_)
                | PhysicalOwner::Paragraph { .. }
                | PhysicalOwner::Run { .. } => false,
            };
            if text_changed && matches!(old.address.physical, PhysicalOwner::TextBody { .. }) {
                changed_bodies.push(old.address.path.clone());
            }
            proof.destination = after.path.clone();
            proof.layout_index = after.layout_index;
            proof.text_changed = text_changed;
            Ok(true)
        })
    }

    /// The parts holding the owners under a path prefix, every part for an
    /// empty prefix.
    pub(crate) fn prefix_parts(&self, prefix: &[PathSeg]) -> Vec<OwnerPart> {
        if prefix.is_empty() {
            self.owners.parts.iter().map(|(part, _)| part).collect()
        } else {
            vec![OwnerPart::of(prefix)]
        }
    }

    /// Explicit proofs for every owner of `parts`, each starting as the
    /// owner's unchanged proof. `edit` adjusts one proof, or returns false to
    /// retire its owner. Owners of other parts survive unchanged.
    pub(crate) fn part_owner_proofs(
        &self,
        parts: Vec<OwnerPart>,
        mut edit: impl FnMut(&LiveOwner, &mut OwnerSurvival) -> PyResult<bool>,
    ) -> PyResult<OwnerProofs> {
        let mut listed = Vec::new();
        for part in parts {
            let Some(owners) = self.owners.parts.part(part) else {
                continue;
            };
            let mut proofs = Vec::new();
            proofs.try_reserve_exact(owners.owners.len()).map_err(|_| {
                pyo3::exceptions::PyMemoryError::new_err("owner proofs allocation failed")
            })?;
            for owner in &owners.owners {
                let mut proof = PartEdit::Unchanged
                    .proof(owner)
                    .expect("an unchanged owner survives");
                if edit(owner, &mut proof)? {
                    proofs.push(proof);
                }
            }
            listed.push((part, proofs));
        }
        Ok(OwnerProofs {
            edit: PartEdit::Unchanged,
            parts: listed,
        })
    }

    pub(crate) fn unchanged_owners(&self) -> PyResult<OwnerProofs> {
        Ok(OwnerProofs::edit(PartEdit::Unchanged))
    }

    /// Retires one removed layout and its descendants, preserving exact surviving part owners.
    pub(crate) fn removed_layout_owners(&self, removed: usize) -> PyResult<OwnerProofs> {
        Ok(OwnerProofs::edit(PartEdit::RemoveLayout(removed)))
    }

    /// Remap exactly one existing sibling ordinal. The operation must use this
    /// same permutation on its staged native tree before preparing publication.
    pub(crate) fn moved_shape_owners(
        &self,
        parent: &[oxml_py_support::PathSeg],
        from: usize,
        to: usize,
    ) -> PyResult<OwnerProofs> {
        use oxml_py_support::PathSeg;
        self.part_owner_proofs(self.prefix_parts(parent), |_, proof| {
            if proof.layout_index.is_some() || !proof.destination.starts_with(parent) {
                return Ok(true);
            }
            let Some(PathSeg::Shape(index)) = proof.destination.get_mut(parent.len()) else {
                return Ok(true);
            };
            *index = if *index == from {
                to
            } else if from < to && *index > from && *index <= to {
                *index - 1
            } else if to < from && *index >= to && *index < from {
                *index + 1
            } else {
                *index
            };
            Ok(true)
        })
    }

    pub(crate) fn removed_shape_owners(
        &self,
        parent: &[oxml_py_support::PathSeg],
        removed: usize,
    ) -> PyResult<OwnerProofs> {
        use oxml_py_support::PathSeg;
        self.part_owner_proofs(self.prefix_parts(parent), |_, proof| {
            if proof.layout_index.is_some() || !proof.destination.starts_with(parent) {
                return Ok(true);
            }
            let Some(PathSeg::Shape(index)) = proof.destination.get_mut(parent.len()) else {
                return Ok(true);
            };
            if *index == removed {
                return Ok(false);
            }
            if *index > removed {
                *index -= 1;
            }
            Ok(true)
        })
    }

    /// Apply the same explicit ordinal operation to every surviving physical
    /// slide descendant. Removed members never contribute a survival proof.
    pub(crate) fn slide_owner_edit(&self, edit: SlideOwnerEdit) -> PyResult<OwnerProofs> {
        Ok(OwnerProofs::edit(PartEdit::Slides(edit)))
    }

    /// Literal edits preserve bodies but retire the actual old text nodes in
    /// each changed body. Matching ordinals are not survival evidence. A slide
    /// whose native record the candidate still shares has no changed text.
    pub(crate) fn replaced_text_owners(
        &self,
        candidate: &rpptx::Presentation,
    ) -> PyResult<OwnerProofs> {
        use oxml_py_support::ContentPath;
        let origins = candidate.unchanged_slides_since(&self.inner);
        let parts = self
            .owners
            .parts
            .iter()
            .map(|(part, _)| part)
            .filter(|part| {
                !matches!(part, OwnerPart::Slide(index) if origins.get(*index) == Some(&Some(*index)))
            })
            .collect();
        let mut proofs = self.part_owner_proofs(parts, |owner, proof| {
            let path = ContentPath::new(owner.address.path.clone(), self.revisions.current());
            proof.text_changed = match &owner.address.physical {
                PhysicalOwner::Slide(_) => {
                    let index = crate::shape::slide_index(&path)?;
                    match (self.inner.slide(index), candidate.slide(index)) {
                        (Some(before), Some(after)) => !before.notes_text_matches(&after),
                        (None, None) => false,
                        _ => true,
                    }
                }
                PhysicalOwner::TextBody { cell, .. } => {
                    let frame = |inner| {
                        let shape = crate::shape::native_shape_ref_at(inner, &path)?;
                        match cell {
                            Some((row, col)) => shape.table()?.cell(*row, *col)?.text_frame(),
                            None => shape.text_frame(),
                        }
                    };
                    frame(&self.inner) != frame(candidate)
                }
                _ => false,
            };
            Ok(true)
        })?;
        // A changed body's paragraphs and runs share its part.
        for (part, list) in &mut proofs.parts {
            let owners = &self.owners.parts.part(*part).expect("listed live part").owners;
            let changed = owners
                .iter()
                .zip(list.iter())
                .filter(|(owner, proof)| {
                    proof.text_changed
                        && matches!(owner.address.physical, PhysicalOwner::TextBody { .. })
                })
                .map(|(owner, _)| owner.address.path.clone())
                .collect::<Vec<_>>();
            let mut owners = owners.iter();
            list.retain(|_| {
                let owner = owners.next().expect("one proof per owner");
                !matches!(
                    owner.address.physical,
                    PhysicalOwner::Paragraph { .. } | PhysicalOwner::Run { .. }
                ) || !changed
                    .iter()
                    .any(|base| owner.address.path.starts_with(base))
            });
        }
        Ok(proofs)
    }

    pub(crate) fn cell_text_owners(
        &self,
        path: &oxml_py_support::ContentPath,
        first_row: usize,
        last_row: usize,
        first_column: usize,
        last_column: usize,
    ) -> PyResult<OwnerProofs> {
        use oxml_py_support::PathSeg;
        let table_path = path
            .segs
            .iter()
            .filter(|seg| {
                matches!(
                    seg,
                    PathSeg::Slide(_) | PathSeg::Layout(_) | PathSeg::Master(_) | PathSeg::Shape(_)
                )
            })
            .copied()
            .collect::<smallvec::SmallVec<[PathSeg; 5]>>();
        self.part_owner_proofs(self.prefix_parts(&table_path), |owner, proof| {
            if owner.address.layout_index.is_some() || !owner.address.path.starts_with(&table_path)
            {
                return Ok(true);
            }
            let cell = match owner.address.physical {
                PhysicalOwner::Cell { row, column, .. } => Some((row, column)),
                PhysicalOwner::TextBody { cell, .. }
                | PhysicalOwner::Paragraph { cell, .. }
                | PhysicalOwner::Run { cell, .. } => cell,
                _ => None,
            };
            let affected = cell.is_some_and(|(row, column)| {
                row >= first_row
                    && row <= last_row
                    && column >= first_column
                    && column <= last_column
            });
            if affected
                && matches!(
                    owner.address.physical,
                    PhysicalOwner::Paragraph { .. } | PhysicalOwner::Run { .. }
                )
            {
                return Ok(false);
            }
            proof.text_changed = affected;
            Ok(true)
        })
    }

    /// Keep earlier paragraph and run owners. Retire only the appended
    /// owner's cached text view, so a materialised collection is not refreshed.
    pub(crate) fn appended_text_owners(
        &self,
        path: &oxml_py_support::ContentPath,
    ) -> PyResult<OwnerProofs> {
        let generation = path
            .owner_generation()
            .ok_or_else(|| PyValueError::new_err("text append needs a bound owner"))?;
        let (part, _) = *self
            .owners
            .by_generation
            .get(&generation)
            .ok_or_else(|| PyValueError::new_err("text append owner is absent"))?;
        self.part_owner_proofs(vec![part], |_, proof| {
            proof.text_changed |= proof.generation == generation;
            Ok(true)
        })
    }

    /// Text replacement retains its physical paragraph and body ancestors,
    /// while the replaced runs and whole-body paragraph descendants retire.
    pub(crate) fn text_changed_owners(
        &self,
        path: &oxml_py_support::ContentPath,
    ) -> PyResult<OwnerProofs> {
        use oxml_py_support::PathSeg;
        let current = self
            .current_path(path)
            .ok_or_else(|| PyValueError::new_err("text mutation owner is absent"))?;
        let generation = path
            .owner_generation()
            .ok_or_else(|| PyValueError::new_err("text mutation needs a bound owner"))?;
        let paragraph = current
            .segs
            .iter()
            .any(|seg| matches!(seg, PathSeg::Para(_)));
        let base = current
            .segs
            .iter()
            .filter(|seg| !matches!(seg, PathSeg::Para(_) | PathSeg::Run(_)))
            .copied()
            .collect::<smallvec::SmallVec<[PathSeg; 5]>>();
        let body = self
            .body_owner_at(&base)
            .map(|owner| owner.owner.generation());
        let owner = self.owners.by_generation.get(&generation);
        // The slide scalar setter edits only the notes part. Slide body
        // owners are unrelated and cannot be retired by that operation.
        if let Some((part, index)) = owner
            && matches!(
                self.owners.parts.part(*part).map(|owners| &owners.owners[*index].address.physical),
                Some(PhysicalOwner::Slide(_))
            )
        {
            return self.part_owner_proofs(vec![*part], |_, proof| {
                proof.text_changed |= proof.generation == generation;
                Ok(true)
            });
        }
        let mut parts = self.prefix_parts(&base);
        if let Some((part, _)) = owner
            && !parts.contains(part)
        {
            parts.push(*part);
        }
        self.part_owner_proofs(parts, |old, proof| {
            let removed = if paragraph {
                matches!(old.address.physical, PhysicalOwner::Run { .. })
                    && old.address.path.starts_with(&current.segs)
            } else {
                matches!(
                    old.address.physical,
                    PhysicalOwner::Paragraph { .. } | PhysicalOwner::Run { .. }
                ) && old.address.path.starts_with(&base)
            };
            if removed {
                return Ok(false);
            }
            if proof.generation == generation || !paragraph && Some(proof.generation) == body {
                proof.text_changed = true;
            }
            Ok(true)
        })
    }

    pub(crate) fn grid_changed_owners(
        &self,
        path: &oxml_py_support::ContentPath,
    ) -> PyResult<OwnerProofs> {
        use oxml_py_support::PathSeg;
        let current = self
            .current_path(path)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("grid owner is absent"))?;
        let table_path = current
            .segs
            .iter()
            .filter(|seg| {
                matches!(
                    seg,
                    PathSeg::Slide(_) | PathSeg::Layout(_) | PathSeg::Master(_) | PathSeg::Shape(_)
                )
            })
            .copied()
            .collect::<smallvec::SmallVec<[PathSeg; 5]>>();
        let table = self
            .owner_at(&table_path, None)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("table owner is absent"))?
            .owner
            .generation();
        // Every old physical cell retires on grid restructure. The table
        // survives with a fresh grid generation, and new cells get births.
        self.part_owner_proofs(self.prefix_parts(&table_path), |_, proof| {
            if proof.destination.starts_with(&table_path)
                && proof
                    .destination
                    .get(table_path.len())
                    .is_some_and(|seg| matches!(seg, PathSeg::Row(_)))
            {
                return Ok(false);
            }
            proof.grid_changed |= proof.generation == table;
            Ok(true)
        })
    }
}
