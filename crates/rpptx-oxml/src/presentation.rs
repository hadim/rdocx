use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::units::Emu;
use oxml_core::xml::reject_non_xml_characters;
use oxml_drawing::namespace::A_NS;
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::text::CT_TextListStyle;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::namespace::{
    FIXED_MODEL_PREFIXES, NamespaceBindings, P_NS, R_NS, all_attributes, root_attributes,
};

const MIN_SLIDE_ID: u32 = 256;
const MAX_SLIDE_ID: u32 = 2_147_483_647;
const P14_NS: &str = "http://schemas.microsoft.com/office/powerpoint/2010/main";
const SECTION_EXTENSION_URI: &str = "{521415D9-36F7-43E2-AB2F-B90AF26B5E84}";

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;
type ParsedSlideList = (Vec<CT_SlideId>, RawAttributes, OrderedRawChildren);
type ParsedMasterList = (Vec<CT_SlideMasterId>, RawAttributes, OrderedRawChildren);
type ParsedIdentifier = (Option<u32>, String, RawAttributes, OrderedRawChildren);
type ParsedSectionSlideIds = (
    Vec<u32>,
    Vec<u8>,
    OrderedRawChildren,
    Vec<SectionSlideIdSidecar>,
);
type ParsedSectionList = (Vec<Section>, Option<(usize, usize)>, Vec<(String, String)>);

/// The typed size of a presentation slide or notes page in EMUs.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_SlideSize {
    pub cx: Emu,
    pub cy: Emu,
    pub kind: Option<String>,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_SlideSize {
    pub fn new(cx: Emu, cy: Emu) -> Result<Self> {
        validate_dimension("size", "cx", cx)?;
        validate_dimension("size", "cy", cy)?;
        Ok(Self {
            cx,
            cy,
            kind: None,
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        })
    }
}

/// One ordered slide identifier and its unresolved OPC relationship id.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_SlideId {
    pub id: u32,
    pub relationship_id: String,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_SlideId {
    pub fn new(id: u32, relationship_id: impl Into<String>) -> Result<Self> {
        validate_slide_id(id)?;
        Ok(Self {
            id,
            relationship_id: relationship_id.into(),
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        })
    }
}

/// One slide-master identifier and its unresolved OPC relationship id.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_SlideMasterId {
    /// The producer-assigned master id, when present in the source part.
    pub id: Option<u32>,
    pub relationship_id: String,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

/// One producer section and the stable slide ids assigned to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Section {
    pub id: Option<String>,
    pub name: Option<String>,
    pub slide_ids: Vec<u32>,
    original_slide_ids: Vec<u32>,
    raw_attributes: Vec<u8>,
    slide_id_list_attributes: Vec<u8>,
    slide_id_list_raw_children: OrderedRawChildren,
    slide_id_sidecars: Vec<SectionSlideIdSidecar>,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SectionSlideIdSidecar {
    id: u32,
    raw_attributes: Vec<u8>,
    raw_children: Vec<u8>,
}

impl Section {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        slide_ids: Vec<u32>,
    ) -> Result<Self> {
        let id = id.into();
        validate_guid("section", &id)?;
        let name = name.into();
        reject_non_xml_characters("section name", &name)?;
        Ok(Self {
            id: Some(id),
            name: Some(name),
            original_slide_ids: slide_ids.clone(),
            slide_ids,
            raw_attributes: Vec::new(),
            slide_id_list_attributes: Vec::new(),
            slide_id_list_raw_children: OrderedRawChildren::default(),
            slide_id_sidecars: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        })
    }
}

impl CT_SlideMasterId {
    pub fn new(id: u32, relationship_id: impl Into<String>) -> Self {
        Self {
            id: Some(id),
            relationship_id: relationship_id.into(),
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }
}

/// The face slot one embedded font part fills, in `p:embeddedFont` order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EmbeddedFontKind {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl EmbeddedFontKind {
    /// Every face slot in schema order.
    pub const ALL: [Self; 4] = [Self::Regular, Self::Bold, Self::Italic, Self::BoldItalic];

    /// The PresentationML local name of the face element.
    pub fn element_name(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Bold => "bold",
            Self::Italic => "italic",
            Self::BoldItalic => "boldItalic",
        }
    }

    fn index(self) -> usize {
        self as usize
    }

    fn from_local_name(name: &[u8]) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.element_name().as_bytes() == name)
    }
}

/// One face element of an embedded font and its unresolved relationship id.
#[derive(Clone, Debug, Eq, PartialEq)]
struct EmbeddedFontFace {
    relationship_id: String,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

/// One `p:embeddedFont`: the `p:font` description and its face relationship ids.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_EmbeddedFontListEntry {
    pub typeface: String,
    pub panose: Option<String>,
    pub pitch_family: Option<i8>,
    pub charset: Option<i8>,
    faces: [Option<EmbeddedFontFace>; 4],
    font_raw_attributes: RawAttributes,
    font_raw_children: OrderedRawChildren,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    /// Unmodelled list children that precede this entry in the source list.
    leading_raw: Vec<Vec<u8>>,
}

impl CT_EmbeddedFontListEntry {
    pub fn new(typeface: impl Into<String>) -> Result<Self> {
        let typeface = typeface.into();
        validate_typeface(&typeface)?;
        Ok(Self {
            typeface,
            panose: None,
            pitch_family: None,
            charset: None,
            faces: [None, None, None, None],
            font_raw_attributes: Vec::new(),
            font_raw_children: OrderedRawChildren::default(),
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
            leading_raw: Vec::new(),
        })
    }

    /// Returns the relationship id of one face, when the entry has it.
    pub fn face(&self, kind: EmbeddedFontKind) -> Option<&str> {
        self.faces[kind.index()]
            .as_ref()
            .map(|face| face.relationship_id.as_str())
    }

    /// Returns every present face and its relationship id in schema order.
    pub fn faces(&self) -> impl Iterator<Item = (EmbeddedFontKind, &str)> {
        EmbeddedFontKind::ALL.into_iter().filter_map(|kind| {
            self.face(kind)
                .map(|relationship_id| (kind, relationship_id))
        })
    }

    /// Points one face at a relationship id and returns the id it replaced.
    ///
    /// A replaced face keeps its unmodelled attributes and children.
    pub fn set_face(
        &mut self,
        kind: EmbeddedFontKind,
        relationship_id: impl Into<String>,
    ) -> Option<String> {
        let relationship_id = relationship_id.into();
        match &mut self.faces[kind.index()] {
            Some(face) => Some(std::mem::replace(
                &mut face.relationship_id,
                relationship_id,
            )),
            slot => {
                *slot = Some(EmbeddedFontFace {
                    relationship_id,
                    raw_attributes: Vec::new(),
                    raw_children: OrderedRawChildren::default(),
                });
                None
            }
        }
    }

    /// Removes one face and returns its relationship id.
    pub fn remove_face(&mut self, kind: EmbeddedFontKind) -> Option<String> {
        self.faces[kind.index()]
            .take()
            .map(|face| face.relationship_id)
    }

    /// Whether the entry carries unmodelled children of its own.
    pub fn has_raw_children(&self) -> bool {
        !self.raw_children.is_empty()
    }
}

/// The typed shell for `/ppt/presentation.xml`.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_Presentation {
    pub slide_master_ids: Vec<CT_SlideMasterId>,
    pub slide_ids: Vec<CT_SlideId>,
    pub slide_size: Option<CT_SlideSize>,
    pub notes_size: CT_SlideSize,
    pub default_text_style: Option<CT_TextListStyle>,
    embedded_fonts: Vec<CT_EmbeddedFontListEntry>,
    embedded_font_list: Option<EmbeddedFontListShell>,
    sections: Vec<Section>,
    extension_list_raw: Option<Vec<u8>>,
    section_list_range: Option<(usize, usize)>,
    section_list_namespaces: Vec<(String, String)>,
    sections_dirty: bool,
    raw_attributes: Vec<(String, String)>,
    slide_master_list_present: bool,
    slide_master_list_attributes: Vec<(String, String)>,
    slide_master_list_raw_children: OrderedRawChildren,
    slide_id_list_present: bool,
    slide_id_list_attributes: Vec<(String, String)>,
    slide_id_list_raw_children: OrderedRawChildren,
    original_slide_relationship_ids: Vec<String>,
    raw_children: OrderedRawChildren,
}

/// The attributes and trailing raw children of a present `p:embeddedFontLst`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct EmbeddedFontListShell {
    raw_attributes: RawAttributes,
    trailing_raw: Vec<Vec<u8>>,
}

#[derive(Default)]
struct PresentationParseState {
    raw_attributes: RawAttributes,
    slide_master_ids: Option<Vec<CT_SlideMasterId>>,
    slide_master_list_attributes: RawAttributes,
    slide_master_list_raw_children: OrderedRawChildren,
    slide_ids: Option<Vec<CT_SlideId>>,
    slide_id_list_attributes: RawAttributes,
    slide_id_list_raw_children: OrderedRawChildren,
    slide_size: Option<CT_SlideSize>,
    notes_size: Option<CT_SlideSize>,
    default_text_style: Option<CT_TextListStyle>,
    embedded_fonts: Vec<CT_EmbeddedFontListEntry>,
    embedded_font_list: Option<EmbeddedFontListShell>,
    sections: Vec<Section>,
    extension_list_raw: Option<Vec<u8>>,
    section_list_range: Option<(usize, usize)>,
    section_list_namespaces: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl PresentationParseState {
    fn capture_child(
        &mut self,
        name: &[u8],
        is_presentationml: bool,
        namespaces: &NamespaceBindings,
        raw: Vec<u8>,
    ) -> Result<()> {
        if !is_presentationml {
            self.raw_children.push(self.boundary, raw);
            return Ok(());
        }

        match name {
            b"sldMasterIdLst" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.slide_master_ids.is_some() {
                    return Err(duplicate("sldMasterIdLst"));
                }
                let (items, attributes, children) = parse_master_list(&raw, namespaces)?;
                self.slide_master_ids = Some(items);
                self.slide_master_list_attributes = attributes;
                self.slide_master_list_raw_children = children;
                self.boundary = self.boundary.max(1);
            }
            b"sldIdLst" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.slide_ids.is_some() {
                    return Err(duplicate("sldIdLst"));
                }
                let (items, attributes, children) = parse_slide_list(&raw, namespaces)?;
                self.slide_ids = Some(items);
                self.slide_id_list_attributes = attributes;
                self.slide_id_list_raw_children = children;
                self.boundary = self.boundary.max(4);
            }
            b"sldSz" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.slide_size.is_some() {
                    return Err(duplicate("sldSz"));
                }
                self.slide_size = Some(parse_size(&raw, "sldSz")?);
                self.boundary = self.boundary.max(5);
            }
            b"notesSz" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.notes_size.is_some() {
                    return Err(duplicate("notesSz"));
                }
                self.notes_size = Some(parse_size(&raw, "notesSz")?);
                self.boundary = self.boundary.max(6);
            }
            b"embeddedFontLst" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.embedded_font_list.is_some() {
                    return Err(duplicate("embeddedFontLst"));
                }
                let (entries, shell) = parse_embedded_font_list(&raw, namespaces)?;
                self.embedded_fonts = entries;
                self.embedded_font_list = Some(shell);
                self.boundary = self.boundary.max(8);
            }
            b"defaultTextStyle" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.default_text_style.is_some() {
                    return Err(duplicate("defaultTextStyle"));
                }
                self.default_text_style = Some(
                    CT_TextListStyle::from_xml(&raw)
                        .map_err(|error| invalid_value(error.to_string()))?,
                );
                self.boundary = self.boundary.max(13);
            }
            b"extLst" => {
                if self.extension_list_raw.is_some() {
                    return Err(duplicate("extLst"));
                }
                let (sections, range, section_namespaces) =
                    parse_sections_from_extension(&raw, namespaces)?;
                self.sections = sections;
                self.section_list_range = range;
                self.section_list_namespaces = section_namespaces;
                self.extension_list_raw = Some(raw);
                self.boundary = self.boundary.max(15);
            }
            _ => {
                let slot = root_schema_boundary(name).unwrap_or(self.boundary);
                self.raw_children.push(slot, raw);
                self.boundary = self.boundary.max(slot);
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<CT_Presentation> {
        let notes_size = self
            .notes_size
            .ok_or_else(|| OxmlError::MissingElement("p:notesSz".to_owned()))?;
        let original_slide_relationship_ids = self
            .slide_ids
            .as_ref()
            .map(|slides| {
                slides
                    .iter()
                    .map(|slide| slide.relationship_id.clone())
                    .collect()
            })
            .unwrap_or_default();
        let presentation = CT_Presentation {
            slide_master_list_present: self.slide_master_ids.is_some(),
            slide_master_ids: self.slide_master_ids.unwrap_or_default(),
            slide_id_list_present: self.slide_ids.is_some(),
            slide_ids: self.slide_ids.unwrap_or_default(),
            slide_size: self.slide_size,
            notes_size,
            default_text_style: self.default_text_style,
            embedded_fonts: self.embedded_fonts,
            embedded_font_list: self.embedded_font_list,
            sections: self.sections,
            extension_list_raw: self.extension_list_raw,
            section_list_range: self.section_list_range,
            section_list_namespaces: self.section_list_namespaces,
            sections_dirty: false,
            raw_attributes: self.raw_attributes,
            slide_master_list_attributes: self.slide_master_list_attributes,
            slide_master_list_raw_children: self.slide_master_list_raw_children,
            slide_id_list_attributes: self.slide_id_list_attributes,
            slide_id_list_raw_children: self.slide_id_list_raw_children,
            original_slide_relationship_ids,
            raw_children: self.raw_children,
        };
        presentation.validate()?;
        Ok(presentation)
    }
}

impl CT_Presentation {
    pub fn new(notes_size: CT_SlideSize) -> Self {
        Self {
            slide_master_ids: Vec::new(),
            slide_ids: Vec::new(),
            slide_size: None,
            notes_size,
            default_text_style: None,
            embedded_fonts: Vec::new(),
            embedded_font_list: None,
            sections: Vec::new(),
            extension_list_raw: None,
            section_list_range: None,
            section_list_namespaces: Vec::new(),
            sections_dirty: false,
            raw_attributes: Vec::new(),
            slide_master_list_present: false,
            slide_master_list_attributes: Vec::new(),
            slide_master_list_raw_children: OrderedRawChildren::default(),
            slide_id_list_present: false,
            slide_id_list_attributes: Vec::new(),
            slide_id_list_raw_children: OrderedRawChildren::default(),
            original_slide_relationship_ids: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }

    /// Replaces only the slide dimensions, preserving size kind and raw data.
    pub fn set_slide_size(&mut self, cx: Emu, cy: Emu) -> Result<()> {
        validate_dimension("sldSz", "cx", cx)?;
        validate_dimension("sldSz", "cy", cy)?;
        if let Some(size) = &mut self.slide_size {
            size.cx = cx;
            size.cy = cy;
        } else {
            self.slide_size = Some(CT_SlideSize::new(cx, cy)?);
        }
        Ok(())
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub fn set_sections(&mut self, sections: Vec<Section>) -> Result<()> {
        validate_sections(&sections, &self.slide_ids)?;
        if sections.is_empty()
            && let (Some(extension_list), Some((start, end))) =
                (&self.extension_list_raw, self.section_list_range)
            && section_list_has_raw_content(
                &extension_list[start..end],
                &self.section_list_namespaces,
            )?
        {
            return Err(invalid_value(
                "cannot clear sections while preserving direct section-list raw payload".to_owned(),
            ));
        }
        self.sections = sections;
        self.sections_dirty = true;
        Ok(())
    }

    pub fn remove_slide_from_sections(&mut self, slide_id: u32) {
        for section in &mut self.sections {
            let before = section.slide_ids.len();
            section.slide_ids.retain(|id| *id != slide_id);
            self.sections_dirty |= section.slide_ids.len() != before;
        }
    }

    /// Returns the modelled `p:embeddedFont` entries in list order.
    pub fn embedded_fonts(&self) -> &[CT_EmbeddedFontListEntry] {
        &self.embedded_fonts
    }

    /// Returns the entry whose `p:font/@typeface` equals `typeface`.
    pub fn embedded_font_mut(&mut self, typeface: &str) -> Option<&mut CT_EmbeddedFontListEntry> {
        self.embedded_fonts
            .iter_mut()
            .find(|entry| entry.typeface == typeface)
    }

    /// Appends one entry, materializing `p:embeddedFontLst` when absent.
    pub fn push_embedded_font(&mut self, entry: CT_EmbeddedFontListEntry) -> Result<()> {
        validate_typeface(&entry.typeface)?;
        if self
            .embedded_fonts
            .iter()
            .any(|existing| existing.typeface == entry.typeface)
        {
            return Err(invalid_value(format!(
                "embedded font {} already exists",
                entry.typeface
            )));
        }
        self.embedded_font_list.get_or_insert_with(Default::default);
        self.embedded_fonts.push(entry);
        Ok(())
    }

    /// Removes one entry by typeface.
    ///
    /// Unmodelled list children that preceded it move to the next entry, so
    /// they keep their place. A list left with no entry, attribute, or raw
    /// child is dropped.
    pub fn remove_embedded_font(&mut self, typeface: &str) -> Option<CT_EmbeddedFontListEntry> {
        let index = self
            .embedded_fonts
            .iter()
            .position(|entry| entry.typeface == typeface)?;
        let mut removed = self.embedded_fonts.remove(index);
        let leading_raw = std::mem::take(&mut removed.leading_raw);
        let shell = self
            .embedded_font_list
            .as_mut()
            .expect("a modelled entry has a list");
        match self.embedded_fonts.get_mut(index) {
            Some(next) => {
                let mut moved = leading_raw;
                moved.append(&mut next.leading_raw);
                next.leading_raw = moved;
            }
            None => {
                let mut moved = leading_raw;
                moved.append(&mut shell.trailing_raw);
                shell.trailing_raw = moved;
            }
        }
        if self.embedded_fonts.is_empty()
            && shell.trailing_raw.is_empty()
            && shell.raw_attributes.is_empty()
        {
            self.embedded_font_list = None;
        }
        Some(removed)
    }

    /// Counts the modelled faces that reference one relationship id.
    pub fn embedded_font_relationship_reference_count(&self, relationship_id: &str) -> usize {
        self.embedded_fonts
            .iter()
            .flat_map(CT_EmbeddedFontListEntry::faces)
            .filter(|(_, id)| *id == relationship_id)
            .count()
    }

    /// Returns `p:presentation/@embedTrueTypeFonts`, or `None` when absent or
    /// not an `xsd:boolean`.
    pub fn embed_true_type_fonts(&self) -> Option<bool> {
        self.root_boolean("embedTrueTypeFonts")
    }

    /// Writes `embedTrueTypeFonts` as `1` or `0` in place, or removes it.
    pub fn set_embed_true_type_fonts(&mut self, value: Option<bool>) {
        self.set_root_boolean("embedTrueTypeFonts", value);
    }

    /// Returns `p:presentation/@saveSubsetFonts`, or `None` when absent or
    /// not an `xsd:boolean`.
    pub fn save_subset_fonts(&self) -> Option<bool> {
        self.root_boolean("saveSubsetFonts")
    }

    /// Writes `saveSubsetFonts` as `1` or `0` in place, or removes it.
    pub fn set_save_subset_fonts(&mut self, value: Option<bool>) {
        self.set_root_boolean("saveSubsetFonts", value);
    }

    fn root_boolean(&self, name: &str) -> Option<bool> {
        self.raw_attributes
            .iter()
            .find(|(attribute, _)| attribute == name)
            .and_then(|(_, value)| match value.trim() {
                "1" | "true" => Some(true),
                "0" | "false" => Some(false),
                _ => None,
            })
    }

    fn set_root_boolean(&mut self, name: &str, value: Option<bool>) {
        if self.root_boolean(name) == value {
            return;
        }
        let position = self
            .raw_attributes
            .iter()
            .position(|(attribute, _)| attribute == name);
        match (value, position) {
            (Some(value), Some(index)) => {
                self.raw_attributes[index].1 = if value { "1" } else { "0" }.to_owned();
            }
            (Some(value), None) => self
                .raw_attributes
                .push((name.to_owned(), if value { "1" } else { "0" }.to_owned())),
            (None, Some(index)) => {
                self.raw_attributes.remove(index);
            }
            (None, None) => {}
        }
    }

    /// Parses a complete PresentationML presentation root with any prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    let namespaces = NamespaceBindings::default().with_start(&element)?;
                    if local_name(element.name().as_ref()) == b"presentation"
                        && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                    {
                        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                        return Self::from_element(&mut reader, &element, &namespaces);
                    }
                    return Err(OxmlError::UnexpectedElement(element_name(&element)));
                }
                Event::Empty(element) => {
                    let namespaces = NamespaceBindings::default().with_start(&element)?;
                    if local_name(element.name().as_ref()) == b"presentation"
                        && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                    {
                        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                        return Err(OxmlError::MissingElement("p:notesSz".to_owned()));
                    }
                    return Err(OxmlError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => {
                    return Err(OxmlError::MissingElement(
                        "PresentationML presentation".to_owned(),
                    ));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        root_namespaces: &NamespaceBindings,
    ) -> Result<Self> {
        let mut state = PresentationParseState {
            raw_attributes: root_attributes(start, FIXED_MODEL_PREFIXES)?,
            ..PresentationParseState::default()
        };
        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    let name = local_name(element.name().as_ref()).to_vec();
                    let namespaces = root_namespaces.with_start(&element)?;
                    let is_presentationml =
                        namespaces.element_uri(element.name().as_ref()) == Some(P_NS);
                    let raw = capture_element(reader, &element)?;
                    state.capture_child(&name, is_presentationml, &namespaces, raw)?;
                }
                Event::Empty(element) => {
                    let name = local_name(element.name().as_ref()).to_vec();
                    let namespaces = root_namespaces.with_start(&element)?;
                    let is_presentationml =
                        namespaces.element_uri(element.name().as_ref()) == Some(P_NS);
                    let raw = capture_empty_element(&element)?;
                    state.capture_child(&name, is_presentationml, &namespaces, raw)?;
                }
                Event::End(element) if local_name(element.name().as_ref()) == b"presentation" => {
                    return state.finish();
                }
                Event::Eof => {
                    return Err(OxmlError::MissingElement(
                        "closing p:presentation".to_owned(),
                    ));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn validate(&self) -> Result<()> {
        if let Some(size) = &self.slide_size {
            validate_size("sldSz", size)?;
        }
        validate_size("notesSz", &self.notes_size)
    }

    /// Writes fixed `p`, `a`, and `r` prefixes in presentation schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut writer = Writer::new(Vec::new());
        let mut root = BytesStart::new("p:presentation");
        root.push_attribute(("xmlns:p", P_NS));
        root.push_attribute(("xmlns:a", A_NS));
        root.push_attribute(("xmlns:r", R_NS));
        push_attributes(&mut root, &self.raw_attributes);
        writer.write_event(Event::Start(root))?;

        emit_raw(&mut writer, self.raw_children.at(0))?;
        if self.slide_master_list_present || !self.slide_master_ids.is_empty() {
            write_master_list(self, &mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(1))?;
        emit_raw(&mut writer, self.raw_children.at(2))?;
        emit_raw(&mut writer, self.raw_children.at(3))?;
        if self.slide_id_list_present || !self.slide_ids.is_empty() {
            write_slide_list(self, &mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(4))?;
        if let Some(size) = &self.slide_size {
            write_size(size, "p:sldSz", &mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(5))?;
        write_size(&self.notes_size, "p:notesSz", &mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(6))?;
        emit_raw(&mut writer, self.raw_children.at(7))?;
        if let Some(shell) = &self.embedded_font_list {
            write_embedded_font_list(shell, &self.embedded_fonts, &mut writer)?;
        }
        for boundary in 8..=12 {
            emit_raw(&mut writer, self.raw_children.at(boundary))?;
        }
        if let Some(style) = &self.default_text_style {
            style
                .write_xml_as(&mut writer, "p:defaultTextStyle")
                .map_err(|error| invalid_value(error.to_string()))?;
        }
        emit_raw(&mut writer, self.raw_children.at(13))?;
        emit_raw(&mut writer, self.raw_children.at(14))?;
        if let Some(extension_list) = write_extension_list(
            self.extension_list_raw.as_deref(),
            self.section_list_range,
            &self.sections,
            self.sections_dirty,
            &self.section_list_namespaces,
        )? {
            writer.get_mut().write_all(&extension_list)?;
        }
        emit_raw(&mut writer, self.raw_children.at(15))?;
        writer.write_event(Event::End(BytesEnd::new("p:presentation")))?;
        Ok(writer.into_inner())
    }

    /// Removes references to one presentation relationship from custom shows.
    pub fn remove_custom_show_slide(&mut self, relationship_id: &str) -> Result<()> {
        let xml = self.to_xml()?;
        let rewritten = remove_custom_show_slide_references(&xml, relationship_id)?;
        *self = Self::from_xml(&rewritten)?;
        Ok(())
    }
}

/// Returns relationship ids referenced by slides inside preserved custom shows.
pub fn custom_show_relationship_ids(xml: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut scopes = vec![NamespaceBindings::default()];
    let mut depth = 0usize;
    let mut custom_show_depth = None;
    let mut ids = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let parent = scopes.last().ok_or_else(|| {
                    invalid_value("missing custom-show namespace scope".to_owned())
                })?;
                let scope = parent.with_start(&element)?;
                let is_custom_show = scope.element_uri(element.name().as_ref()) == Some(P_NS)
                    && local_name(element.name().as_ref()) == b"custShowLst";
                if custom_show_depth.is_some() || is_custom_show {
                    collect_custom_show_slide_id(&element, &scope, &mut ids)?;
                }
                scopes.push(scope);
                depth += 1;
                if is_custom_show {
                    custom_show_depth = Some(depth);
                }
            }
            Event::Empty(element) => {
                let parent = scopes.last().ok_or_else(|| {
                    invalid_value("missing custom-show namespace scope".to_owned())
                })?;
                let scope = parent.with_start(&element)?;
                if custom_show_depth.is_some() {
                    collect_custom_show_slide_id(&element, &scope, &mut ids)?;
                }
            }
            Event::End(_) => {
                if custom_show_depth == Some(depth) {
                    custom_show_depth = None;
                }
                if depth == 0 || scopes.len() == 1 {
                    return Err(invalid_value(
                        "custom-show XML has an unmatched closing tag".to_owned(),
                    ));
                }
                depth -= 1;
                scopes.pop();
            }
            Event::Eof => {
                if depth != 0 || scopes.len() != 1 {
                    return Err(invalid_value(
                        "custom-show XML ended before its root closed".to_owned(),
                    ));
                }
                return Ok(ids);
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn collect_custom_show_slide_id(
    element: &BytesStart<'_>,
    scope: &NamespaceBindings,
    ids: &mut Vec<String>,
) -> Result<()> {
    if scope.element_uri(element.name().as_ref()) != Some(P_NS)
        || local_name(element.name().as_ref()) != b"sld"
    {
        return Ok(());
    }
    for attribute in element.attributes() {
        let attribute = attribute?;
        if scope.attribute_uri(attribute.key.as_ref()) == Some(R_NS)
            && local_name(attribute.key.as_ref()) == b"id"
        {
            ids.push(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                    .into_owned(),
            );
        }
    }
    Ok(())
}

fn remove_custom_show_slide_references(xml: &[u8], relationship_id: &str) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut scopes = vec![NamespaceBindings::default()];
    let mut depth = 0usize;
    let mut custom_show_depth = None;
    let mut removals = Vec::new();
    loop {
        let event_start = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let parent = scopes.last().ok_or_else(|| {
                    invalid_value("missing custom-show namespace scope".to_owned())
                })?;
                let scope = parent.with_start(&element)?;
                let is_custom_show = scope.element_uri(element.name().as_ref()) == Some(P_NS)
                    && local_name(element.name().as_ref()) == b"custShowLst";
                if custom_show_depth.is_some()
                    && custom_show_slide_relationship_id(&element, &scope)?.as_deref()
                        == Some(relationship_id)
                {
                    reader.read_to_end(element.name())?;
                    removals.push(event_start..reader.buffer_position() as usize);
                } else {
                    scopes.push(scope);
                    depth += 1;
                    if is_custom_show {
                        custom_show_depth = Some(depth);
                    }
                }
            }
            Event::Empty(element) => {
                let parent = scopes.last().ok_or_else(|| {
                    invalid_value("missing custom-show namespace scope".to_owned())
                })?;
                let scope = parent.with_start(&element)?;
                if custom_show_depth.is_some()
                    && custom_show_slide_relationship_id(&element, &scope)?.as_deref()
                        == Some(relationship_id)
                {
                    removals.push(event_start..reader.buffer_position() as usize);
                }
            }
            Event::End(_) => {
                if custom_show_depth == Some(depth) {
                    custom_show_depth = None;
                }
                if depth == 0 || scopes.len() == 1 {
                    return Err(invalid_value(
                        "custom-show XML has an unmatched closing tag".to_owned(),
                    ));
                }
                depth -= 1;
                scopes.pop();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if depth != 0 || scopes.len() != 1 {
        return Err(invalid_value(
            "custom-show XML ended before its root closed".to_owned(),
        ));
    }
    let mut rewritten = xml.to_vec();
    for range in removals.into_iter().rev() {
        rewritten.drain(range);
    }
    Ok(rewritten)
}

fn custom_show_slide_relationship_id(
    element: &BytesStart<'_>,
    scope: &NamespaceBindings,
) -> Result<Option<String>> {
    if scope.element_uri(element.name().as_ref()) != Some(P_NS)
        || local_name(element.name().as_ref()) != b"sld"
    {
        return Ok(None);
    }
    for attribute in element.attributes() {
        let attribute = attribute?;
        if scope.attribute_uri(attribute.key.as_ref()) == Some(R_NS)
            && local_name(attribute.key.as_ref()) == b"id"
        {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn parse_sections_from_extension(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<ParsedSectionList> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut scopes = vec![inherited.clone()];
    let mut depth = 0usize;
    let mut section_extension_depth = None;
    loop {
        let start_position = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let parent = scopes
                    .last()
                    .ok_or_else(|| invalid_value("missing section namespace scope".to_owned()))?;
                let scope = parent.with_start(&element)?;
                if depth == 1
                    && local_name(element.name().as_ref()) == b"ext"
                    && scope.element_uri(element.name().as_ref()) == Some(P_NS)
                    && all_attributes(&element)?
                        .into_iter()
                        .any(|(name, value)| name == "uri" && value == SECTION_EXTENSION_URI)
                {
                    section_extension_depth = Some(depth + 1);
                }
                if section_extension_depth == Some(depth)
                    && local_name(element.name().as_ref()) == b"sectionLst"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                {
                    let raw = capture_element(&mut reader, &element)?;
                    let section_namespaces = scope
                        .entries()
                        .into_iter()
                        .filter(|(prefix, _)| raw_uses_namespace_prefix(&raw, prefix))
                        .collect();
                    let end_position = reader.buffer_position() as usize;
                    return Ok((
                        parse_section_list(&raw, &scope)?,
                        Some((start_position, end_position)),
                        section_namespaces,
                    ));
                }
                scopes.push(scope);
                depth += 1;
            }
            Event::Empty(element) => {
                let parent = scopes
                    .last()
                    .ok_or_else(|| invalid_value("missing section namespace scope".to_owned()))?;
                let scope = parent.with_start(&element)?;
                if section_extension_depth == Some(depth)
                    && local_name(element.name().as_ref()) == b"sectionLst"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                {
                    return Err(OxmlError::MissingElement(
                        "p14:sectionLst requires p14:section".to_owned(),
                    ));
                }
            }
            Event::End(_) => {
                if section_extension_depth == Some(depth) {
                    section_extension_depth = None;
                }
                if scopes.len() > 1 {
                    scopes.pop();
                    depth -= 1;
                }
            }
            Event::Eof => return Ok((Vec::new(), None, Vec::new())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_section_list(xml: &[u8], inherited: &NamespaceBindings) -> Result<Vec<Section>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut root_scope = inherited.clone();
    let mut sections = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"sectionLst" => {
                root_scope = inherited.with_start(&element)?;
            }
            Event::Start(element) => {
                let scope = root_scope.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                if local_name(element.name().as_ref()) == b"section"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                {
                    sections.push(parse_section(&raw, &scope)?);
                }
            }
            Event::Empty(element) => {
                let scope = root_scope.with_start(&element)?;
                if local_name(element.name().as_ref()) == b"section"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                {
                    return Err(OxmlError::MissingElement(
                        "p14:section requires p14:sldIdLst".to_owned(),
                    ));
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sectionLst" => break,
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "closing p14:sectionLst".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
    if sections.is_empty() {
        return Err(OxmlError::MissingElement(
            "p14:sectionLst requires p14:section".to_owned(),
        ));
    }
    Ok(sections)
}

fn parse_section(xml: &[u8], inherited: &NamespaceBindings) -> Result<Section> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut id = None;
    let mut name = None;
    let mut raw_attributes = Vec::new();
    let mut slide_ids = None;
    let mut slide_id_list_attributes = Vec::new();
    let mut slide_id_list_raw_children = OrderedRawChildren::default();
    let mut slide_id_sidecars = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut section_scope = inherited.clone();
    let mut boundary = 0;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"section" => {
                section_scope = inherited.with_start(&element)?;
                for (attribute, value) in all_attributes(&element)? {
                    match attribute.as_str() {
                        "id" if id.is_none() => id = Some(value),
                        "name" if name.is_none() => name = Some(value),
                        "id" | "name" => return Err(duplicate("section attribute")),
                        _ => {}
                    }
                }
                raw_attributes = lexical_attributes(&element, &["id", "name"])?;
                raw_attributes.extend_from_slice(&dependent_p14_shadow_attributes(&element, xml)?);
                raw_attributes.extend_from_slice(&inherited_p14_shadow_attributes(
                    inherited,
                    &element,
                    xml,
                    &raw_attributes,
                ));
            }
            Event::Start(element) => {
                let scope = section_scope.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                if local_name(element.name().as_ref()) == b"sldIdLst"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                    && slide_ids.is_none()
                {
                    let parsed = parse_section_slide_ids(&raw, &scope)?;
                    slide_ids = Some(parsed.0);
                    slide_id_list_attributes = parsed.1;
                    slide_id_list_raw_children = parsed.2;
                    slide_id_sidecars = parsed.3;
                    boundary = 1;
                } else {
                    raw_children.push(boundary, raw);
                }
            }
            Event::Empty(element) => {
                let scope = section_scope.with_start(&element)?;
                let raw = capture_empty_element(&element)?;
                if local_name(element.name().as_ref()) == b"sldIdLst"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                    && slide_ids.is_none()
                {
                    slide_ids = Some(Vec::new());
                    slide_id_list_attributes = lexical_attributes(&element, &[])?;
                    slide_id_list_attributes
                        .extend_from_slice(&dependent_p14_shadow_attributes(&element, &raw)?);
                    slide_id_list_attributes.extend_from_slice(&inherited_p14_shadow_attributes(
                        &section_scope,
                        &element,
                        &raw,
                        &slide_id_list_attributes,
                    ));
                    boundary = 1;
                } else {
                    raw_children.push(boundary, raw);
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"section" => break,
            Event::Eof => return Err(OxmlError::MissingElement("closing p14:section".to_owned())),
            event @ (Event::Text(_)
            | Event::CData(_)
            | Event::Comment(_)
            | Event::PI(_)
            | Event::DocType(_)) => raw_children.push(boundary, capture_misc_event(event)?),
            _ => {}
        }
        buffer.clear();
    }
    if let Some(value) = &id {
        validate_guid("section", value)?;
    }
    let slide_ids = slide_ids
        .ok_or_else(|| OxmlError::MissingElement("p14:section requires p14:sldIdLst".to_owned()))?;
    Ok(Section {
        id,
        name,
        original_slide_ids: slide_ids.clone(),
        slide_ids,
        raw_attributes,
        slide_id_list_attributes,
        slide_id_list_raw_children,
        slide_id_sidecars,
        raw_children,
    })
}

fn parse_section_slide_ids(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<ParsedSectionSlideIds> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut root_scope = inherited.clone();
    let mut attributes = Vec::new();
    let mut ids = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut sidecars = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"sldIdLst" => {
                root_scope = inherited.with_start(&element)?;
                attributes = lexical_attributes(&element, &[])?;
                attributes.extend_from_slice(&dependent_p14_shadow_attributes(&element, xml)?);
                attributes.extend_from_slice(&inherited_p14_shadow_attributes(
                    inherited,
                    &element,
                    xml,
                    &attributes,
                ));
            }
            Event::Start(element) => {
                let scope = root_scope.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                if local_name(element.name().as_ref()) == b"sldId"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                {
                    let sidecar = parse_section_slide_id_sidecar(&raw, &scope)?;
                    ids.push(sidecar.id);
                    sidecars.push(sidecar);
                } else {
                    raw_children.push(ids.len(), raw);
                }
            }
            Event::Empty(element) => {
                let scope = root_scope.with_start(&element)?;
                let raw = capture_empty_element(&element)?;
                if local_name(element.name().as_ref()) == b"sldId"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                {
                    let id = parse_section_slide_id(&element)?;
                    ids.push(id);
                    let mut raw_attributes = lexical_attributes(&element, &["id"])?;
                    raw_attributes
                        .extend_from_slice(&dependent_p14_shadow_attributes(&element, &raw)?);
                    raw_attributes.extend_from_slice(&inherited_p14_shadow_attributes(
                        &root_scope,
                        &element,
                        &raw,
                        &raw_attributes,
                    ));
                    sidecars.push(SectionSlideIdSidecar {
                        id,
                        raw_attributes,
                        raw_children: Vec::new(),
                    });
                } else {
                    raw_children.push(ids.len(), raw);
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sldIdLst" => break,
            Event::Eof => return Err(OxmlError::MissingElement("closing p14:sldIdLst".to_owned())),
            event @ (Event::Text(_)
            | Event::CData(_)
            | Event::Comment(_)
            | Event::PI(_)
            | Event::DocType(_)) => raw_children.push(ids.len(), capture_misc_event(event)?),
            _ => {}
        }
        buffer.clear();
    }
    Ok((ids, attributes, raw_children, sidecars))
}

fn parse_section_slide_id(element: &BytesStart<'_>) -> Result<u32> {
    all_attributes(element)?
        .into_iter()
        .find(|(name, _)| name == "id")
        .ok_or_else(|| missing_attribute("p14:sldId", "id"))?
        .1
        .parse()
        .map_err(|_| invalid_value("p14:sldId has malformed @id".to_owned()))
}

fn parse_section_slide_id_sidecar(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<SectionSlideIdSidecar> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut id = None;
    let mut raw_attributes = Vec::new();
    let mut raw_children = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if id.is_none() => {
                id = Some(parse_section_slide_id(&element)?);
                raw_attributes = lexical_attributes(&element, &["id"])?;
                raw_attributes.extend_from_slice(&dependent_p14_shadow_attributes(&element, xml)?);
                raw_attributes.extend_from_slice(&inherited_p14_shadow_attributes(
                    inherited,
                    &element,
                    xml,
                    &raw_attributes,
                ));
            }
            Event::Start(element) => {
                raw_children.extend_from_slice(&capture_element(&mut reader, &element)?);
            }
            Event::Empty(element) => {
                raw_children.extend_from_slice(&capture_empty_element(&element)?);
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sldId" => break,
            Event::Eof => return Err(OxmlError::MissingElement("closing p14:sldId".to_owned())),
            event @ (Event::Text(_)
            | Event::CData(_)
            | Event::Comment(_)
            | Event::PI(_)
            | Event::DocType(_)) => raw_children.extend_from_slice(&capture_misc_event(event)?),
            _ => {}
        }
        buffer.clear();
    }
    Ok(SectionSlideIdSidecar {
        id: id.ok_or_else(|| missing_attribute("p14:sldId", "id"))?,
        raw_attributes,
        raw_children,
    })
}

fn write_extension_list(
    original: Option<&[u8]>,
    section_range: Option<(usize, usize)>,
    sections: &[Section],
    sections_dirty: bool,
    section_list_namespaces: &[(String, String)],
) -> Result<Option<Vec<u8>>> {
    if original.is_none() && sections.is_empty() {
        return Ok(None);
    }
    if let Some(original) = original {
        if let Some((start, end)) = section_range {
            if !sections_dirty {
                return Ok(Some(original.to_vec()));
            }
            if sections.is_empty() {
                if section_list_has_raw_content(&original[start..end], section_list_namespaces)? {
                    return Err(invalid_value(
                        "cannot clear sections while preserving direct section-list raw payload"
                            .to_owned(),
                    ));
                }
                let mut output = Vec::with_capacity(original.len() - (end - start));
                output.extend_from_slice(&original[..start]);
                output.extend_from_slice(&original[end..]);
                return Ok(Some(output));
            }
            let replacement = write_section_list_preserving_raw(
                &original[start..end],
                sections,
                section_list_namespaces,
            )?;
            let mut output = Vec::with_capacity(original.len() - (end - start) + replacement.len());
            output.extend_from_slice(&original[..start]);
            output.extend_from_slice(&replacement);
            output.extend_from_slice(&original[end..]);
            return Ok(Some(output));
        }
        if sections.is_empty() {
            return Ok(Some(original.to_vec()));
        }
        let section_xml = write_section_list(sections)?;
        if original.trim_ascii_end().ends_with(b"/>") {
            let slash = original
                .windows(2)
                .rposition(|window| window == b"/>")
                .expect("trimmed empty element ends with its slash");
            let name_end = original[1..]
                .iter()
                .position(|byte| byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>'))
                .map(|index| index + 1)
                .ok_or_else(|| invalid_value("malformed empty p:extLst".to_owned()))?;
            let mut output = Vec::with_capacity(original.len() + section_xml.len() + 80);
            output.extend_from_slice(&original[..slash]);
            output.extend_from_slice(b">");
            output.extend_from_slice(format!("<p:ext uri=\"{SECTION_EXTENSION_URI}\">").as_bytes());
            output.extend_from_slice(&section_xml);
            output.extend_from_slice(b"</p:ext></");
            output.extend_from_slice(&original[1..name_end]);
            output.extend_from_slice(b">");
            return Ok(Some(output));
        }
        let closing = original
            .windows(b"</".len())
            .rposition(|window| window == b"</")
            .ok_or_else(|| invalid_value("p:extLst has no closing tag".to_owned()))?;
        let mut output = Vec::with_capacity(original.len() + section_xml.len() + 80);
        output.extend_from_slice(&original[..closing]);
        output.extend_from_slice(format!("<p:ext uri=\"{SECTION_EXTENSION_URI}\">").as_bytes());
        output.extend_from_slice(&section_xml);
        output.extend_from_slice(b"</p:ext>");
        output.extend_from_slice(&original[closing..]);
        return Ok(Some(output));
    }
    let section_xml = write_section_list(sections)?;
    let mut output = format!("<p:extLst><p:ext uri=\"{SECTION_EXTENSION_URI}\">").into_bytes();
    output.extend_from_slice(&section_xml);
    output.extend_from_slice(b"</p:ext></p:extLst>");
    Ok(Some(output))
}

fn section_list_has_raw_content(
    original: &[u8],
    inherited_namespaces: &[(String, String)],
) -> Result<bool> {
    let mut reader = Reader::from_reader(original);
    let mut buffer = Vec::new();
    let mut root_scope = NamespaceBindings::from_entries(inherited_namespaces);
    let mut saw_root = false;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if !saw_root => {
                saw_root = true;
                root_scope = root_scope.with_start(&element)?;
                if all_attributes(&element)?.into_iter().any(|(name, value)| {
                    !((name == "xmlns" || name.starts_with("xmlns:")) && value == P14_NS)
                }) {
                    return Ok(true);
                }
            }
            Event::Start(element) => {
                let scope = root_scope.with_start(&element)?;
                if local_name(element.name().as_ref()) != b"section"
                    || scope.element_uri(element.name().as_ref()) != Some(P14_NS)
                {
                    return Ok(true);
                }
                capture_element(&mut reader, &element)?;
            }
            Event::Empty(element) => {
                let scope = root_scope.with_start(&element)?;
                if local_name(element.name().as_ref()) != b"section"
                    || scope.element_uri(element.name().as_ref()) != Some(P14_NS)
                {
                    return Ok(true);
                }
            }
            Event::Text(text) => {
                let bytes: &[u8] = text.as_ref();
                if bytes.iter().any(|byte| !byte.is_ascii_whitespace()) {
                    return Ok(true);
                }
            }
            Event::Comment(_) | Event::PI(_) | Event::CData(_) | Event::DocType(_) => {
                return Ok(true);
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sectionLst" => {
                return Ok(false);
            }
            Event::Eof => return Ok(false),
            _ => {}
        }
        buffer.clear();
    }
}

fn write_section_list(sections: &[Section]) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    let mut root = BytesStart::new("p14:sectionLst");
    root.push_attribute(("xmlns:p14", P14_NS));
    writer.write_event(Event::Start(root))?;
    for section in sections {
        write_section(&mut writer, section)?;
    }
    writer.write_event(Event::End(BytesEnd::new("p14:sectionLst")))?;
    Ok(writer.into_inner())
}

fn write_section_list_preserving_raw(
    original: &[u8],
    sections: &[Section],
    inherited_namespaces: &[(String, String)],
) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(original);
    let mut buffer = Vec::new();
    let mut root_scope = NamespaceBindings::from_entries(inherited_namespaces);
    let mut opening = Vec::new();
    let mut closing = Vec::new();
    let mut originals = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    loop {
        let event_start = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if opening.is_empty() => {
                root_scope = root_scope.with_start(&element)?;
                opening.extend_from_slice(&original[..reader.buffer_position() as usize]);
            }
            Event::Start(element) => {
                let scope = root_scope.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                if local_name(element.name().as_ref()) == b"section"
                    && scope.element_uri(element.name().as_ref()) == Some(P14_NS)
                {
                    originals.push(parse_section(&raw, &scope)?);
                } else {
                    raw_children.push(originals.len(), raw);
                }
            }
            Event::Empty(element) => {
                raw_children.push(originals.len(), capture_empty_element(&element)?);
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sectionLst" => {
                closing
                    .extend_from_slice(&original[event_start..reader.buffer_position() as usize]);
                break;
            }
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "closing p14:sectionLst".to_owned(),
                ));
            }
            event @ (Event::Text(_)
            | Event::CData(_)
            | Event::Comment(_)
            | Event::PI(_)
            | Event::DocType(_)) => {
                let mut event_writer = Writer::new(Vec::new());
                event_writer.write_event(event.into_owned())?;
                raw_children.push(originals.len(), event_writer.into_inner());
            }
            _ => {}
        }
        buffer.clear();
    }
    let original_to_current = originals
        .iter()
        .map(|original| {
            sections
                .iter()
                .position(|section| section.id == original.id)
        })
        .collect::<Vec<_>>();
    let mut writer = Writer::new(Vec::new());
    writer.get_mut().write_all(&opening)?;
    for (index, section) in sections.iter().enumerate() {
        emit_raw(
            &mut writer,
            raw_children.at_reconciled(index, 0, &original_to_current, sections.len()),
        )?;
        write_section(&mut writer, section)?;
    }
    emit_raw(
        &mut writer,
        raw_children.at_reconciled(sections.len(), 0, &original_to_current, sections.len()),
    )?;
    writer.get_mut().write_all(&closing)?;
    Ok(writer.into_inner())
}

fn write_section<W: Write>(writer: &mut Writer<W>, section: &Section) -> Result<()> {
    let prefix = section_model_prefix(&section.raw_attributes, "p14")?;
    let tag = format!("{prefix}:section");
    let mut start = BytesStart::new(&tag);
    push_section_namespace(&mut start, prefix);
    if let Some(name) = &section.name {
        start.push_attribute(("name", name.as_str()));
    }
    if let Some(id) = &section.id {
        start.push_attribute(("id", id.as_str()));
    }
    write_start_with_raw(writer, &start, &section.raw_attributes, false)?;
    emit_raw(writer, section.raw_children.at(0))?;
    let list_prefix = section_model_prefix(&section.slide_id_list_attributes, prefix)?;
    let list_tag = format!("{list_prefix}:sldIdLst");
    let mut list = BytesStart::new(&list_tag);
    if list_prefix != prefix {
        push_section_namespace(&mut list, list_prefix);
    }
    write_start_with_raw(writer, &list, &section.slide_id_list_attributes, false)?;
    let original_to_current = section
        .original_slide_ids
        .iter()
        .map(|id| section.slide_ids.iter().position(|current| current == id))
        .collect::<Vec<_>>();
    for (index, id) in section.slide_ids.iter().enumerate() {
        emit_raw(
            writer,
            section.slide_id_list_raw_children.at_reconciled(
                index,
                0,
                &original_to_current,
                section.slide_ids.len(),
            ),
        )?;
        let value = id.to_string();
        let sidecar = section.slide_id_sidecars.iter().find(|item| item.id == *id);
        let slide_prefix = if let Some(sidecar) = sidecar {
            section_model_prefix(&sidecar.raw_attributes, list_prefix)?
        } else {
            list_prefix
        };
        let slide_tag = format!("{slide_prefix}:sldId");
        let mut slide = BytesStart::new(&slide_tag);
        if slide_prefix != list_prefix {
            push_section_namespace(&mut slide, slide_prefix);
        }
        slide.push_attribute(("id", value.as_str()));
        if let Some(sidecar) = sidecar {
            if sidecar.raw_children.is_empty() {
                write_start_with_raw(writer, &slide, &sidecar.raw_attributes, true)?;
            } else {
                write_start_with_raw(writer, &slide, &sidecar.raw_attributes, false)?;
                writer.get_mut().write_all(&sidecar.raw_children)?;
                writer.write_event(Event::End(BytesEnd::new(&slide_tag)))?;
            }
        } else {
            writer.write_event(Event::Empty(slide))?;
        }
    }
    emit_raw(
        writer,
        section.slide_id_list_raw_children.at_reconciled(
            section.slide_ids.len(),
            0,
            &original_to_current,
            section.slide_ids.len(),
        ),
    )?;
    writer.write_event(Event::End(BytesEnd::new(&list_tag)))?;
    emit_raw(writer, section.raw_children.at(1))?;
    writer.write_event(Event::End(BytesEnd::new(&tag)))?;
    Ok(())
}

fn lexical_attributes(start: &BytesStart<'_>, modeled: &[&str]) -> Result<Vec<u8>> {
    let _ = all_attributes(start)?;
    let raw = start.attributes_raw();
    let mut output = Vec::new();
    let mut cursor = 0usize;
    while cursor < raw.len() {
        let span_start = cursor;
        while cursor < raw.len() && raw[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor == raw.len() {
            output.extend_from_slice(&raw[span_start..]);
            break;
        }
        let name_start = cursor;
        while cursor < raw.len() && !raw[cursor].is_ascii_whitespace() && raw[cursor] != b'=' {
            cursor += 1;
        }
        let name = &raw[name_start..cursor];
        while cursor < raw.len() && raw[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        cursor += 1;
        while cursor < raw.len() && raw[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let quote = raw[cursor];
        cursor += 1;
        while cursor < raw.len() && raw[cursor] != quote {
            cursor += 1;
        }
        cursor += 1;
        let is_modeled =
            !name.contains(&b':') && modeled.iter().any(|candidate| name == candidate.as_bytes());
        let fixed_namespace = matches!(name, b"xmlns:p14" | b"xmlns:p14m" | b"xmlns:p14model");
        if !is_modeled && !fixed_namespace {
            output.extend_from_slice(&raw[span_start..cursor]);
        }
    }
    Ok(output)
}

fn section_model_prefix<'a>(raw_attributes: &[u8], inherited: &'a str) -> Result<&'a str> {
    if !has_raw_namespace_declaration(raw_attributes, inherited) {
        return Ok(inherited);
    }
    ["p14", "p14m", "p14model"]
        .into_iter()
        .find(|prefix| !has_raw_namespace_declaration(raw_attributes, prefix))
        .ok_or_else(|| invalid_value("no unshadowed section model prefix is available".to_owned()))
}

fn push_section_namespace(start: &mut BytesStart<'_>, prefix: &str) {
    match prefix {
        "p14" => start.push_attribute(("xmlns:p14", P14_NS)),
        "p14m" => start.push_attribute(("xmlns:p14m", P14_NS)),
        "p14model" => start.push_attribute(("xmlns:p14model", P14_NS)),
        _ => unreachable!("section model prefix is selected from fixed candidates"),
    }
}

fn has_raw_namespace_declaration(raw_attributes: &[u8], prefix: &str) -> bool {
    let declaration = format!("xmlns:{prefix}");
    raw_attributes
        .windows(declaration.len())
        .enumerate()
        .any(|(index, window)| {
            window == declaration.as_bytes()
                && raw_attributes
                    .get(index + declaration.len())
                    .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'=')
        })
}

fn dependent_p14_shadow_attributes(start: &BytesStart<'_>, xml: &[u8]) -> Result<Vec<u8>> {
    let decoded = all_attributes(start)?;
    let descendants = xml
        .iter()
        .position(|byte| *byte == b'>')
        .map_or(&[][..], |index| &xml[index + 1..]);
    let raw = start.attributes_raw();
    let mut preserved = Vec::new();
    let mut cursor = 0usize;
    while cursor < raw.len() {
        let span_start = cursor;
        while cursor < raw.len() && raw[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < raw.len() && !raw[cursor].is_ascii_whitespace() && raw[cursor] != b'=' {
            cursor += 1;
        }
        let name = &raw[name_start..cursor];
        while cursor < raw.len() && raw[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor == raw.len() {
            break;
        }
        cursor += 1;
        while cursor < raw.len() && raw[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let quote = *raw
            .get(cursor)
            .ok_or_else(|| invalid_value("malformed p14 namespace shadow".to_owned()))?;
        cursor += 1;
        while cursor < raw.len() && raw[cursor] != quote {
            cursor += 1;
        }
        cursor += 1;
        let prefix = match name {
            b"xmlns:p14" => b"p14".as_slice(),
            b"xmlns:p14m" => b"p14m".as_slice(),
            b"xmlns:p14model" => b"p14model".as_slice(),
            _ => continue,
        };
        let value = decoded
            .iter()
            .find(|(candidate, _)| candidate.as_bytes() == name)
            .map(|(_, value)| value.as_str());
        if value != Some(P14_NS)
            && (shell_attribute_depends_on_prefix(start, prefix)
                || descendant_depends_on_prefix(descendants, prefix))
        {
            preserved.extend_from_slice(&raw[span_start..cursor]);
        }
    }
    Ok(preserved)
}

fn inherited_p14_shadow_attributes(
    inherited: &NamespaceBindings,
    start: &BytesStart<'_>,
    xml: &[u8],
    local_attributes: &[u8],
) -> Vec<u8> {
    let descendants = xml
        .iter()
        .position(|byte| *byte == b'>')
        .map_or(&[][..], |index| &xml[index + 1..]);
    let mut preserved = Vec::new();
    for (prefix, uri) in inherited.entries() {
        if !matches!(prefix.as_str(), "p14" | "p14m" | "p14model")
            || uri == P14_NS
            || has_raw_namespace_declaration(local_attributes, &prefix)
            || !(shell_attribute_depends_on_prefix(start, prefix.as_bytes())
                || descendant_depends_on_prefix(descendants, prefix.as_bytes()))
        {
            continue;
        }
        let escaped = quick_xml::escape::escape(&uri);
        preserved.extend_from_slice(format!(" xmlns:{prefix}=\"{escaped}\"").as_bytes());
    }
    preserved
}

fn shell_attribute_depends_on_prefix(start: &BytesStart<'_>, prefix: &[u8]) -> bool {
    let qualified = [prefix, b":"].concat();
    start
        .attributes()
        .with_checks(false)
        .filter_map(std::result::Result::ok)
        .any(|attribute| attribute.key.as_ref().starts_with(&qualified))
}

fn descendant_depends_on_prefix(xml: &[u8], prefix: &[u8]) -> bool {
    let qualified = [prefix, b":"].concat();
    let declaration = [b"xmlns:".as_slice(), prefix].concat();
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut shadowed = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                let current = shadowed.last().copied().unwrap_or(false)
                    || element.attributes().with_checks(false).any(|attribute| {
                        attribute.is_ok_and(|attribute| attribute.key.as_ref() == declaration)
                    });
                if !current && element_uses_prefix(&element, &qualified) {
                    return true;
                }
                shadowed.push(current);
            }
            Ok(Event::Empty(element)) => {
                let current = shadowed.last().copied().unwrap_or(false)
                    || element.attributes().with_checks(false).any(|attribute| {
                        attribute.is_ok_and(|attribute| attribute.key.as_ref() == declaration)
                    });
                if !current && element_uses_prefix(&element, &qualified) {
                    return true;
                }
            }
            Ok(Event::End(_)) => {
                shadowed.pop();
            }
            Ok(Event::Eof) | Err(_) => return false,
            _ => {}
        }
        buffer.clear();
    }
}

fn element_uses_prefix(element: &BytesStart<'_>, qualified: &[u8]) -> bool {
    element.name().as_ref().starts_with(qualified)
        || element
            .attributes()
            .with_checks(false)
            .filter_map(std::result::Result::ok)
            .any(|attribute| attribute.key.as_ref().starts_with(qualified))
}

fn raw_uses_namespace_prefix(xml: &[u8], prefix: &str) -> bool {
    if prefix.is_empty() {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(element) | Event::Empty(element)) => {
                    if !element.name().as_ref().contains(&b':') {
                        return true;
                    }
                }
                Ok(Event::Eof) | Err(_) => return false,
                _ => {}
            }
            buffer.clear();
        }
    }
    let qualified = format!("{prefix}:");
    xml.windows(qualified.len())
        .any(|window| window == qualified.as_bytes())
}

fn write_start_with_raw<W: Write>(
    writer: &mut Writer<W>,
    start: &BytesStart<'_>,
    raw_attributes: &[u8],
    empty: bool,
) -> Result<()> {
    writer.get_mut().write_all(b"<")?;
    writer.get_mut().write_all(start.as_ref())?;
    writer.get_mut().write_all(raw_attributes)?;
    writer
        .get_mut()
        .write_all(if empty { b"/>" } else { b">" })?;
    Ok(())
}

fn capture_misc_event(event: Event<'_>) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    writer.write_event(event.into_owned())?;
    Ok(writer.into_inner())
}

fn validate_sections(sections: &[Section], slides: &[CT_SlideId]) -> Result<()> {
    let known = slides
        .iter()
        .map(|slide| slide.id)
        .collect::<std::collections::HashSet<_>>();
    let mut section_ids = std::collections::HashSet::new();
    let mut assigned_slides = std::collections::HashSet::new();
    for section in sections {
        let id = section
            .id
            .as_ref()
            .ok_or_else(|| invalid_value("section mutation requires an id".to_owned()))?;
        validate_guid("section", id)?;
        if !section_ids.insert(id) {
            return Err(invalid_value(format!("duplicate section id {id}")));
        }
        for slide_id in &section.slide_ids {
            if !known.contains(slide_id) {
                return Err(invalid_value(format!(
                    "unknown section slide id {slide_id}"
                )));
            }
            if !assigned_slides.insert(*slide_id) {
                return Err(invalid_value(format!(
                    "slide id {slide_id} belongs to more than one section"
                )));
            }
        }
    }
    Ok(())
}

fn validate_guid(kind: &str, value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 38
        && bytes[0] == b'{'
        && bytes[37] == b'}'
        && [9, 14, 19, 24]
            .into_iter()
            .all(|index| bytes[index] == b'-')
        && bytes[1..37]
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(invalid_value(format!(
            "{kind} id is not a braced GUID: {value}"
        )))
    }
}

fn parse_embedded_font_list(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<(Vec<CT_EmbeddedFontListEntry>, EmbeddedFontListShell)> {
    let mut reader = Reader::from_reader(xml);
    let mut list_namespaces = inherited.clone();
    let mut entries = Vec::new();
    let mut shell = EmbeddedFontListShell::default();
    let mut pending_raw = Vec::new();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"embeddedFontLst" => {
                shell.raw_attributes = all_attributes(&element)?;
                list_namespaces = inherited.with_start(&element)?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == b"embeddedFontLst" => {
                shell.raw_attributes = all_attributes(&element)?;
                break;
            }
            Event::Start(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                push_embedded_font_child(
                    &element,
                    raw,
                    &namespaces,
                    &mut entries,
                    &mut pending_raw,
                )?;
            }
            Event::Empty(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_empty_element(&element)?;
                push_embedded_font_child(
                    &element,
                    raw,
                    &namespaces,
                    &mut entries,
                    &mut pending_raw,
                )?;
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"embeddedFontLst" => {
                break;
            }
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "closing p:embeddedFontLst".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
    shell.trailing_raw = pending_raw;
    Ok((entries, shell))
}

/// Models one list child, or keeps it raw in place when it is not an
/// embedded font entry this model can represent without loss.
fn push_embedded_font_child(
    element: &BytesStart<'_>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    entries: &mut Vec<CT_EmbeddedFontListEntry>,
    pending_raw: &mut Vec<Vec<u8>>,
) -> Result<()> {
    let modelled = if local_name(element.name().as_ref()) == b"embeddedFont"
        && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
    {
        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
        parse_embedded_font_entry(&raw, namespaces)
            .ok()
            .filter(|entry| {
                entries
                    .iter()
                    .all(|existing| existing.typeface != entry.typeface)
            })
    } else {
        None
    };
    match modelled {
        Some(mut entry) => {
            entry.leading_raw = std::mem::take(pending_raw);
            entries.push(entry);
        }
        None => pending_raw.push(raw),
    }
    Ok(())
}

fn parse_embedded_font_entry(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<CT_EmbeddedFontListEntry> {
    let mut reader = Reader::from_reader(xml);
    let mut entry_namespaces = inherited.clone();
    let mut entry = None::<CT_EmbeddedFontListEntry>;
    let mut raw_attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut faces: [Option<EmbeddedFontFace>; 4] = [None, None, None, None];
    // 0 before p:font, 1 after it, and 2 + face index after each face.
    let mut boundary = 0;
    let mut buffer = Vec::new();
    let mut depth = 0_usize;
    loop {
        let event = reader.read_event_into(&mut buffer)?;
        let (element, is_empty) = match event {
            Event::Start(element) => (element, false),
            Event::Empty(element) => (element, true),
            Event::End(_) => break,
            Event::Eof => break,
            _ => {
                buffer.clear();
                continue;
            }
        };
        if depth == 0 {
            raw_attributes = all_attributes(&element)?;
            entry_namespaces = inherited.with_start(&element)?;
            depth = 1;
            if is_empty {
                break;
            }
            buffer.clear();
            continue;
        }
        let element = element.into_owned();
        let namespaces = entry_namespaces.with_start(&element)?;
        let raw = if is_empty {
            capture_empty_element(&element)?
        } else {
            capture_element(&mut reader, &element)?
        };
        let name = local_name(element.name().as_ref()).to_vec();
        let is_presentationml = namespaces.element_uri(element.name().as_ref()) == Some(P_NS);
        if is_presentationml && name == b"font" {
            if entry.is_some() || boundary > 0 {
                return Err(duplicate("embeddedFont p:font"));
            }
            entry = Some(parse_embedded_font_description(&raw)?);
            boundary = 1;
        } else if let Some(kind) = EmbeddedFontKind::from_local_name(&name)
            .filter(|_| is_presentationml && entry.is_some())
        {
            if boundary > kind.index() + 1 {
                return Err(invalid_value(format!(
                    "embeddedFont p:{} is out of schema order",
                    kind.element_name()
                )));
            }
            let (id, relationship_id, face_attributes, face_children) =
                parse_identifier(&raw, kind.element_name(), &namespaces)?;
            if id.is_some() {
                return Err(invalid_value(format!(
                    "embeddedFont p:{} carries an unmodelled id",
                    kind.element_name()
                )));
            }
            faces[kind.index()] = Some(EmbeddedFontFace {
                relationship_id,
                raw_attributes: face_attributes,
                raw_children: face_children,
            });
            boundary = kind.index() + 2;
        } else {
            raw_children.push(boundary, raw);
        }
        buffer.clear();
    }
    let mut entry = entry.ok_or_else(|| OxmlError::MissingElement("embeddedFont p:font".into()))?;
    entry.faces = faces;
    entry.raw_attributes = raw_attributes;
    entry.raw_children = raw_children;
    Ok(entry)
}

fn parse_embedded_font_description(xml: &[u8]) -> Result<CT_EmbeddedFontListEntry> {
    let mut reader = Reader::from_reader(xml);
    let mut attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    let mut seen_root = false;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if !seen_root => {
                attributes = all_attributes(&element)?;
                seen_root = true;
            }
            Event::Empty(element) if !seen_root => {
                attributes = all_attributes(&element)?;
                break;
            }
            Event::Start(element) => raw_children.push(0, capture_element(&mut reader, &element)?),
            Event::Empty(element) => raw_children.push(0, capture_empty_element(&element)?),
            Event::End(_) | Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    let mut typeface = None;
    let mut panose = None;
    let mut pitch_family = None;
    let mut charset = None;
    let mut font_raw_attributes = Vec::new();
    for (name, value) in attributes {
        match name.as_str() {
            "typeface" if typeface.is_none() => typeface = Some(value),
            "panose" if panose.is_none() => panose = Some(value),
            "pitchFamily" if pitch_family.is_none() => {
                pitch_family = Some(parse_byte("font", "pitchFamily", &value)?);
            }
            "charset" if charset.is_none() => {
                charset = Some(parse_byte("font", "charset", &value)?);
            }
            "typeface" | "panose" | "pitchFamily" | "charset" => {
                return Err(duplicate(&format!("font @{name}")));
            }
            _ => font_raw_attributes.push((name, value)),
        }
    }
    let mut entry = CT_EmbeddedFontListEntry::new(
        typeface.ok_or_else(|| missing_attribute("font", "typeface"))?,
    )?;
    entry.panose = panose;
    entry.pitch_family = pitch_family;
    entry.charset = charset;
    entry.font_raw_attributes = font_raw_attributes;
    entry.font_raw_children = raw_children;
    Ok(entry)
}

fn write_embedded_font_list<W: Write>(
    shell: &EmbeddedFontListShell,
    entries: &[CT_EmbeddedFontListEntry],
    writer: &mut Writer<W>,
) -> Result<()> {
    let mut list = BytesStart::new("p:embeddedFontLst");
    push_attributes(&mut list, &shell.raw_attributes);
    if entries.is_empty() && shell.trailing_raw.is_empty() {
        writer.write_event(Event::Empty(list))?;
        return Ok(());
    }
    writer.write_event(Event::Start(list))?;
    for entry in entries {
        emit_raw(writer, entry.leading_raw.iter().map(Vec::as_slice))?;
        write_embedded_font_entry(entry, writer)?;
    }
    emit_raw(writer, shell.trailing_raw.iter().map(Vec::as_slice))?;
    writer.write_event(Event::End(BytesEnd::new("p:embeddedFontLst")))?;
    Ok(())
}

fn write_embedded_font_entry<W: Write>(
    entry: &CT_EmbeddedFontListEntry,
    writer: &mut Writer<W>,
) -> Result<()> {
    validate_typeface(&entry.typeface)?;
    let mut start = BytesStart::new("p:embeddedFont");
    push_attributes(&mut start, &entry.raw_attributes);
    writer.write_event(Event::Start(start))?;
    emit_raw(writer, entry.raw_children.at(0))?;

    let mut font = BytesStart::new("p:font");
    font.push_attribute(("typeface", entry.typeface.as_str()));
    if let Some(panose) = &entry.panose {
        font.push_attribute(("panose", panose.as_str()));
    }
    let pitch_family = entry.pitch_family.map(|value| value.to_string());
    if let Some(pitch_family) = &pitch_family {
        font.push_attribute(("pitchFamily", pitch_family.as_str()));
    }
    let charset = entry.charset.map(|value| value.to_string());
    if let Some(charset) = &charset {
        font.push_attribute(("charset", charset.as_str()));
    }
    push_attributes(&mut font, &entry.font_raw_attributes);
    if entry.font_raw_children.is_empty() {
        writer.write_event(Event::Empty(font))?;
    } else {
        writer.write_event(Event::Start(font))?;
        emit_raw(writer, entry.font_raw_children.at(0))?;
        writer.write_event(Event::End(BytesEnd::new("p:font")))?;
    }
    emit_raw(writer, entry.raw_children.at(1))?;

    for kind in EmbeddedFontKind::ALL {
        if let Some(face) = &entry.faces[kind.index()] {
            write_identifier(
                &format!("p:{}", kind.element_name()),
                None,
                &face.relationship_id,
                &face.raw_attributes,
                &face.raw_children,
                writer,
            )?;
        }
        emit_raw(writer, entry.raw_children.at(kind.index() + 2))?;
    }
    writer.write_event(Event::End(BytesEnd::new("p:embeddedFont")))?;
    Ok(())
}

fn validate_typeface(typeface: &str) -> Result<()> {
    if typeface.trim().is_empty() {
        return Err(invalid_value(
            "embedded font typeface must not be empty".to_owned(),
        ));
    }
    reject_non_xml_characters("embedded font typeface", typeface)
}

fn parse_byte(element: &str, attribute: &str, value: &str) -> Result<i8> {
    value
        .parse::<i8>()
        .map_err(|_| invalid_value(format!("{element} has malformed @{attribute}: {value}")))
}

fn parse_slide_list(xml: &[u8], inherited: &NamespaceBindings) -> Result<ParsedSlideList> {
    let mut reader = Reader::from_reader(xml);
    let mut list_namespaces = inherited.clone();
    let mut items = Vec::new();
    let mut attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"sldIdLst" => {
                attributes = all_attributes(&element)?;
                list_namespaces = inherited.with_start(&element)?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == b"sldIdLst" => {
                attributes = all_attributes(&element)?;
                return Ok((items, attributes, raw_children));
            }
            Event::Start(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                if local_name(element.name().as_ref()) == b"sldId"
                    && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                {
                    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                    items.push(parse_slide_id(&raw, &namespaces)?);
                } else {
                    raw_children.push(items.len(), raw);
                }
            }
            Event::Empty(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_empty_element(&element)?;
                if local_name(element.name().as_ref()) == b"sldId"
                    && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                {
                    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                    items.push(parse_slide_id(&raw, &namespaces)?);
                } else {
                    raw_children.push(items.len(), raw);
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sldIdLst" => {
                return Ok((items, attributes, raw_children));
            }
            Event::Eof => return Err(OxmlError::MissingElement("closing p:sldIdLst".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_master_list(xml: &[u8], inherited: &NamespaceBindings) -> Result<ParsedMasterList> {
    let mut reader = Reader::from_reader(xml);
    let mut list_namespaces = inherited.clone();
    let mut items = Vec::new();
    let mut attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"sldMasterIdLst" => {
                attributes = all_attributes(&element)?;
                list_namespaces = inherited.with_start(&element)?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == b"sldMasterIdLst" => {
                attributes = all_attributes(&element)?;
                return Ok((items, attributes, raw_children));
            }
            Event::Start(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                if local_name(element.name().as_ref()) == b"sldMasterId"
                    && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                {
                    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                    items.push(parse_master_id(&raw, &namespaces)?);
                } else {
                    raw_children.push(items.len(), raw);
                }
            }
            Event::Empty(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_empty_element(&element)?;
                if local_name(element.name().as_ref()) == b"sldMasterId"
                    && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                {
                    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                    items.push(parse_master_id(&raw, &namespaces)?);
                } else {
                    raw_children.push(items.len(), raw);
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sldMasterIdLst" => {
                return Ok((items, attributes, raw_children));
            }
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "closing p:sldMasterIdLst".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_slide_id(xml: &[u8], namespaces: &NamespaceBindings) -> Result<CT_SlideId> {
    let (id, relationship_id, raw_attributes, raw_children) =
        parse_identifier(xml, "sldId", namespaces)?;
    let id = id.ok_or_else(|| missing_attribute("sldId", "id"))?;
    Ok(CT_SlideId {
        id,
        relationship_id,
        raw_attributes,
        raw_children,
    })
}

fn parse_master_id(xml: &[u8], namespaces: &NamespaceBindings) -> Result<CT_SlideMasterId> {
    let (id, relationship_id, raw_attributes, raw_children) =
        parse_identifier(xml, "sldMasterId", namespaces)?;
    Ok(CT_SlideMasterId {
        id,
        relationship_id,
        raw_attributes,
        raw_children,
    })
}

fn parse_identifier(
    xml: &[u8],
    expected: &str,
    inherited: &NamespaceBindings,
) -> Result<ParsedIdentifier> {
    let mut reader = Reader::from_reader(xml);
    let mut id = None;
    let mut relationship_id = None;
    let mut raw_attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                let namespaces = inherited.with_start(&element)?;
                parse_identifier_attributes(
                    &element,
                    expected,
                    &namespaces,
                    &mut id,
                    &mut relationship_id,
                    &mut raw_attributes,
                )?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                let namespaces = inherited.with_start(&element)?;
                parse_identifier_attributes(
                    &element,
                    expected,
                    &namespaces,
                    &mut id,
                    &mut relationship_id,
                    &mut raw_attributes,
                )?;
                break;
            }
            Event::Start(element) => raw_children.push(0, capture_element(&mut reader, &element)?),
            Event::Empty(element) => raw_children.push(0, capture_empty_element(&element)?),
            Event::End(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                break;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok((
        id,
        relationship_id.ok_or_else(|| missing_attribute(expected, "r:id"))?,
        raw_attributes,
        raw_children,
    ))
}

fn parse_identifier_attributes(
    element: &BytesStart<'_>,
    expected: &str,
    namespaces: &NamespaceBindings,
    id: &mut Option<u32>,
    relationship_id: &mut Option<String>,
    raw_attributes: &mut Vec<(String, String)>,
) -> Result<()> {
    for (name, value) in all_attributes(element)? {
        if name == "id" {
            if id.is_some() {
                return Err(duplicate("id attribute"));
            }
            *id = Some(parse_u32(expected, "id", &value)?);
        } else if local_name(name.as_bytes()) == b"id"
            && namespaces.attribute_uri(name.as_bytes()) == Some(R_NS)
        {
            if relationship_id.replace(value).is_some() {
                return Err(duplicate("relationship id attribute"));
            }
        } else {
            raw_attributes.push((name, value));
        }
    }
    Ok(())
}

fn parse_size(xml: &[u8], expected: &str) -> Result<CT_SlideSize> {
    let mut reader = Reader::from_reader(xml);
    let mut cx = None;
    let mut cy = None;
    let mut kind = None;
    let mut raw_attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                parse_size_attributes(
                    &element,
                    expected,
                    &mut cx,
                    &mut cy,
                    &mut kind,
                    &mut raw_attributes,
                )?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                parse_size_attributes(
                    &element,
                    expected,
                    &mut cx,
                    &mut cy,
                    &mut kind,
                    &mut raw_attributes,
                )?;
                break;
            }
            Event::Start(element) => raw_children.push(0, capture_element(&mut reader, &element)?),
            Event::Empty(element) => raw_children.push(0, capture_empty_element(&element)?),
            Event::End(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                break;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    let size = CT_SlideSize {
        cx: cx.ok_or_else(|| missing_attribute(expected, "cx"))?,
        cy: cy.ok_or_else(|| missing_attribute(expected, "cy"))?,
        kind,
        raw_attributes,
        raw_children,
    };
    validate_size(expected, &size)?;
    Ok(size)
}

fn parse_size_attributes(
    element: &BytesStart<'_>,
    expected: &str,
    cx: &mut Option<Emu>,
    cy: &mut Option<Emu>,
    kind: &mut Option<String>,
    raw_attributes: &mut Vec<(String, String)>,
) -> Result<()> {
    for (name, value) in all_attributes(element)? {
        match name.as_str() {
            "cx" if cx.is_none() => *cx = Some(parse_emu(expected, "cx", &value)?),
            "cy" if cy.is_none() => *cy = Some(parse_emu(expected, "cy", &value)?),
            "type" if kind.is_none() => *kind = Some(value),
            "cx" | "cy" | "type" => return Err(duplicate(&format!("{expected} @{name}"))),
            _ => raw_attributes.push((name, value)),
        }
    }
    Ok(())
}

fn write_master_list<W: Write>(
    presentation: &CT_Presentation,
    writer: &mut Writer<W>,
) -> Result<()> {
    let mut list = BytesStart::new("p:sldMasterIdLst");
    push_attributes(&mut list, &presentation.slide_master_list_attributes);
    if presentation.slide_master_ids.is_empty()
        && presentation.slide_master_list_raw_children.is_empty()
    {
        writer.write_event(Event::Empty(list))?;
        return Ok(());
    }
    writer.write_event(Event::Start(list))?;
    emit_raw(writer, presentation.slide_master_list_raw_children.at(0))?;
    for (index, item) in presentation.slide_master_ids.iter().enumerate() {
        write_identifier(
            "p:sldMasterId",
            item.id,
            &item.relationship_id,
            &item.raw_attributes,
            &item.raw_children,
            writer,
        )?;
        emit_raw(
            writer,
            presentation.slide_master_list_raw_children.at(index + 1),
        )?;
    }
    writer.write_event(Event::End(BytesEnd::new("p:sldMasterIdLst")))?;
    Ok(())
}

fn write_slide_list<W: Write>(
    presentation: &CT_Presentation,
    writer: &mut Writer<W>,
) -> Result<()> {
    let mut list = BytesStart::new("p:sldIdLst");
    push_attributes(&mut list, &presentation.slide_id_list_attributes);
    if presentation.slide_ids.is_empty() && presentation.slide_id_list_raw_children.is_empty() {
        writer.write_event(Event::Empty(list))?;
        return Ok(());
    }
    writer.write_event(Event::Start(list))?;
    let original_to_current = presentation
        .original_slide_relationship_ids
        .iter()
        .map(|relationship_id| {
            presentation
                .slide_ids
                .iter()
                .position(|slide| slide.relationship_id == *relationship_id)
        })
        .collect::<Vec<_>>();
    for (index, item) in presentation.slide_ids.iter().enumerate() {
        emit_raw(
            writer,
            presentation.slide_id_list_raw_children.at_reconciled(
                index,
                0,
                &original_to_current,
                presentation.slide_ids.len(),
            ),
        )?;
        write_identifier(
            "p:sldId",
            Some(item.id),
            &item.relationship_id,
            &item.raw_attributes,
            &item.raw_children,
            writer,
        )?;
    }
    emit_raw(
        writer,
        presentation.slide_id_list_raw_children.at_reconciled(
            presentation.slide_ids.len(),
            0,
            &original_to_current,
            presentation.slide_ids.len(),
        ),
    )?;
    writer.write_event(Event::End(BytesEnd::new("p:sldIdLst")))?;
    Ok(())
}

fn write_identifier<W: Write>(
    tag: &str,
    id: Option<u32>,
    relationship_id: &str,
    raw_attributes: &[(String, String)],
    raw_children: &OrderedRawChildren,
    writer: &mut Writer<W>,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    let id_value = id.map(|value| value.to_string());
    if let Some(id_value) = &id_value {
        start.push_attribute(("id", id_value.as_str()));
    }
    start.push_attribute(("r:id", relationship_id));
    push_attributes(&mut start, raw_attributes);
    if raw_children.is_empty() {
        writer.write_event(Event::Empty(start))?;
    } else {
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, raw_children.at(0))?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
    }
    Ok(())
}

fn write_size<W: Write>(size: &CT_SlideSize, tag: &str, writer: &mut Writer<W>) -> Result<()> {
    validate_size(tag, size)?;
    let cx = size.cx.0.to_string();
    let cy = size.cy.0.to_string();
    let mut start = BytesStart::new(tag);
    start.push_attribute(("cx", cx.as_str()));
    start.push_attribute(("cy", cy.as_str()));
    if let Some(kind) = &size.kind {
        start.push_attribute(("type", kind.as_str()));
    }
    push_attributes(&mut start, &size.raw_attributes);
    if size.raw_children.is_empty() {
        writer.write_event(Event::Empty(start))?;
    } else {
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, size.raw_children.at(0))?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
    }
    Ok(())
}

fn validate_slide_id(id: u32) -> Result<()> {
    if (MIN_SLIDE_ID..=MAX_SLIDE_ID).contains(&id) {
        Ok(())
    } else {
        Err(invalid_value(format!(
            "PowerPoint slide id {id} is outside {MIN_SLIDE_ID}..={MAX_SLIDE_ID}"
        )))
    }
}

fn validate_size(element: &str, size: &CT_SlideSize) -> Result<()> {
    validate_dimension(element, "cx", size.cx)?;
    validate_dimension(element, "cy", size.cy)
}

fn validate_dimension(element: &str, attribute: &str, value: Emu) -> Result<()> {
    if value.0 > 0 {
        Ok(())
    } else {
        Err(invalid_value(format!(
            "{element} @{attribute} must be a positive EMU value"
        )))
    }
}

fn parse_emu(element: &str, attribute: &str, value: &str) -> Result<Emu> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| invalid_value(format!("{element} has malformed @{attribute}: {value}")))?;
    let emu = Emu(parsed);
    validate_dimension(element, attribute, emu)?;
    Ok(emu)
}

fn parse_u32(element: &str, attribute: &str, value: &str) -> Result<u32> {
    value
        .parse::<u32>()
        .map_err(|_| invalid_value(format!("{element} has malformed @{attribute}: {value}")))
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &[(String, String)]) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn emit_raw<'a, W: Write>(
    writer: &mut Writer<W>,
    children: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    for child in children {
        writer.get_mut().write_all(child)?;
    }
    Ok(())
}

fn root_schema_boundary(name: &[u8]) -> Option<usize> {
    match name {
        b"notesMasterIdLst" => Some(2),
        b"handoutMasterIdLst" => Some(3),
        b"smartTags" => Some(7),
        b"custShowLst" => Some(9),
        b"photoAlbum" => Some(10),
        b"custDataLst" => Some(11),
        b"kiosk" => Some(12),
        b"modifyVerifier" => Some(14),
        b"extLst" => Some(15),
        _ => None,
    }
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn element_name(element: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.name().as_ref()).into_owned()
}

fn missing_attribute(element: &str, attribute: &str) -> OxmlError {
    OxmlError::MissingElement(format!("{element} requires @{attribute}"))
}

fn duplicate(element: &str) -> OxmlError {
    invalid_value(format!("duplicate PresentationML {element}"))
}

fn invalid_value(message: String) -> OxmlError {
    OxmlError::InvalidValue(message)
}
