use super::*;

impl Provenance {
    pub(crate) fn components(
        &self,
        block: BlockId,
        source: &Place,
    ) -> BTreeMap<Vec<Projection>, BTreeSet<Place>> {
        let mut result = BTreeMap::<Vec<Projection>, BTreeSet<Place>>::new();
        if let Some(bindings) = &self.incoming[block] {
            for source in resolve(source, bindings, &self.origins).places {
                for (slot, loans) in bindings {
                    if compatible_prefix(&source, slot) {
                        let entry = result
                            .entry(slot.projections[source.projections.len()..].to_vec())
                            .or_default();
                        for loan in loans {
                            entry.extend(self.origins[*loan].places.iter().cloned());
                        }
                    }
                }
            }
        }
        result
    }
}

impl Provenance {
    pub(in super::super) fn reachable(&self, block: BlockId) -> bool {
        self.incoming[block].is_some()
    }
}
