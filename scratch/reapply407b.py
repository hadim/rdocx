p='crates/rdocx-py/src/document.rs'
s=open(p).read()
def rep(old, new, count=1):
    global s
    assert s.count(old)==count, old[:90]
    s=s.replace(old,new)
rep('''impl WordOwnerLocation {
    fn route_hash(&self) -> u64 {
        let mut hash = DefaultHasher::new();
        std::mem::discriminant(self).hash(&mut hash);
        let mut content = |location: &rdocx::ContentLocation| {
            let story = location.story();
            story.kind().hash(&mut hash);
            story.part_name().hash(&mut hash);
            story.owner_index().hash(&mut hash);''','''/// A multiply-rotate hasher over machine words, for owner routes and the
/// owner index maps. Their keys are process-local integers and route
/// hashes whose bucket members are still compared with `same_route`, so
/// they need no keyed hash, and a body edit indexes every owner again.
#[derive(Default)]
struct IndexHasher(u64);

/// A map of the owner indexes.
type IndexMap<K, V> = HashMap<K, V, std::hash::BuildHasherDefault<IndexHasher>>;

impl Hasher for IndexHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.write_u64(u64::from_le_bytes(word));
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(0x517c_c1b7_2722_0a95);
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }
}

impl WordOwnerLocation {
    fn route_hash(&self) -> u64 {
        let mut hash = IndexHasher::default();
        std::mem::discriminant(self).hash(&mut hash);
        let mut content = |location: &rdocx::ContentLocation| {
            let story = location.story();
            story.kind().hash(&mut hash);
            // Equal routes share their part, so its length tells parts apart
            // well enough for buckets.
            story.part_name().len().hash(&mut hash);
            story.owner_index().hash(&mut hash);''')
rep('''struct WordOwnerIndexes {
    by_generation: HashMap<u64, usize>,
    by_route: HashMap<u64, Vec<usize>>,
    direct_body_paragraphs: HashMap<usize, usize>,
    body_paragraphs: Option<Vec<Option<rdocx::ContentLocation>>>,
    body_tables: Vec<Option<rdocx::ContentLocation>>,
    cell_paragraphs: HashMap<(usize, usize, usize), Vec<rdocx::ContentLocation>>,
    paragraphs_by_parent: HashMap<usize, Vec<usize>>,
    tables_by_parent: HashMap<usize, Vec<usize>>,
}''','''struct WordOwnerIndexes {
    by_generation: IndexMap<u64, usize>,
    by_route: IndexMap<u64, Vec<usize>>,
    direct_body_paragraphs: IndexMap<usize, usize>,
    body_paragraphs: Option<Vec<Option<rdocx::ContentLocation>>>,
    body_tables: Vec<Option<rdocx::ContentLocation>>,
    cell_paragraphs: HashMap<(usize, usize, usize), Vec<rdocx::ContentLocation>>,
    paragraphs_by_parent: IndexMap<usize, Vec<usize>>,
    tables_by_parent: IndexMap<usize, Vec<usize>>,
}''')
i=s.index('    fn prepare(owners: &[WordOwner], document: &rdocx::Document) -> PyResult<Self> {\n        let mut indexes = Self {')
j=s.index('        };', i)
block=s[i:j]
block2=(block.replace('by_generation: HashMap::new()','by_generation: IndexMap::default()')
        .replace('by_route: HashMap::new()','by_route: IndexMap::default()')
        .replace('direct_body_paragraphs: HashMap::new()','direct_body_paragraphs: IndexMap::default()')
        .replace('paragraphs_by_parent: HashMap::new()','paragraphs_by_parent: IndexMap::default()')
        .replace('tables_by_parent: HashMap::new()','tables_by_parent: IndexMap::default()'))
assert block2.count('IndexMap::default()')==5
s=s[:i]+block2+s[j:]
rep('''    fn index_owners_again(&mut self, owners: &[WordOwner]) {
        self.by_generation.clear();
        self.by_route.clear();
        self.direct_body_paragraphs.clear();
        self.paragraphs_by_parent.clear();
        self.tables_by_parent.clear();
        self.index_owners(owners);
    }''','''    fn index_owners_again(&mut self, owners: &[WordOwner]) {
        self.by_generation.clear();
        // Route buckets keep their allocations, and those left empty go.
        self.by_route.values_mut().for_each(Vec::clear);
        self.direct_body_paragraphs.clear();
        self.paragraphs_by_parent.clear();
        self.tables_by_parent.clear();
        self.index_owners(owners);
        self.by_route.retain(|_, bucket| !bucket.is_empty());
    }''')
rep('''        let after = plan.first + plan.removed;
        let paragraph_delta = isize::from(plan.inserted_paragraph) - plan.removed as isize;
        let moved_ordinal = |ordinal: usize, delta: isize| ordinal.saturating_add_signed(delta);
        let mut cursor = 0;
        for (index, owner) in self.owners.iter_mut().enumerate() {
            note_fx260_word(1);
            if spans.iter().any(|span| span.contains(&index)) {
                cursor += owner.location.scopes().len();
                continue;
            }''','''        let after = plan.first + plan.removed;
        let paragraph_delta = isize::from(plan.inserted_paragraph) - plan.removed as isize;
        let moved_ordinal = |ordinal: usize, delta: isize| ordinal.saturating_add_signed(delta);
        // Kept owners take their next states, in inventory order, by move.
        let mut states = states.into_iter();
        for (index, owner) in self.owners.iter_mut().enumerate() {
            note_fx260_word(1);
            if spans.iter().any(|span| span.contains(&index)) {
                for _ in 0..owner.location.scopes().len() {
                    states.next();
                }
                continue;
            }''')
rep('''            owner.owner = states[cursor].clone();
            cursor += 1;
            for state in [&mut owner.content, &mut owner.properties, &mut owner.grid]
                .into_iter()
                .flatten()
            {
                *state = states[cursor].clone();
                cursor += 1;
            }
        }
        self.indexes.index_owners_again(&self.owners);''','''            let mut next = || states.next().expect("a prepared state for every scope");
            owner.owner = next();
            for state in [&mut owner.content, &mut owner.properties, &mut owner.grid]
                .into_iter()
                .flatten()
            {
                *state = next();
            }
        }
        self.indexes.index_owners_again(&self.owners);''')
open(p,'w').write(s)
print("ok")
