//! Run properties (`CT_RPr`).

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::error::Result;
use crate::namespace::matches_local_name;
use crate::properties::{
    CT_Shd, append_modeled_toggle_attributes, get_word_val_attr, is_word_attribute,
    is_word_element, parse_word_toggle, raw_is_modeled_attribute_carrier, raw_occurrence,
    record_modeled_toggle_candidate, remove_redundant_modeled_toggle_candidate,
    replay_modeled_toggle_raw, toggle_element_is_explicitly_empty,
    toggle_has_unsupported_attributes, word_prefixes_at, write_toggle,
};
use crate::raw_xml::{capture_element, capture_empty_element};
use crate::revision::CT_Revision;
use crate::shared::{ST_HighlightColor, ST_Underline};
use crate::units::{HalfPoint, Twips};

const RPR_STYLE_SLOT: u8 = 0;
const RPR_FONTS_SLOT: u8 = 1;
const RPR_BOLD_SLOT: u8 = 2;
const RPR_BOLD_CS_SLOT: u8 = 3;
const RPR_ITALIC_SLOT: u8 = 4;
const RPR_ITALIC_CS_SLOT: u8 = 5;
const RPR_CAPS_SLOT: u8 = 6;
const RPR_SMALL_CAPS_SLOT: u8 = 7;
const RPR_STRIKE_SLOT: u8 = 8;
const RPR_DSTRIKE_SLOT: u8 = 9;
const RPR_VANISH_SLOT: u8 = 16;
const RPR_COLOR_SLOT: u8 = 18;
const RPR_SPACING_SLOT: u8 = 19;
const RPR_WIDTH_SLOT: u8 = 20;
const RPR_POSITION_SLOT: u8 = 22;
const RPR_SIZE_SLOT: u8 = 23;
const RPR_SIZE_CS_SLOT: u8 = 24;
const RPR_HIGHLIGHT_SLOT: u8 = 25;
const RPR_UNDERLINE_SLOT: u8 = 26;
const RPR_SHADING_SLOT: u8 = 29;
const RPR_VERT_ALIGN_SLOT: u8 = 31;
const RPR_RTL_SLOT: u8 = 32;
const RPR_LANG_SLOT: u8 = 35;
const RPR_MARKER_SLOT: u8 = 39;
const RPR_CHANGE_SLOT: u8 = 40;
const RPR_END_SLOT: u8 = 41;

fn record_rpr_modeled(
    rpr: &mut CT_RPr,
    pending_raw: &mut Vec<Vec<u8>>,
    occurrences: &mut [usize],
    slot: u8,
) {
    let occurrence = if slot == RPR_MARKER_SLOT {
        occurrences[slot as usize]
    } else {
        0
    };
    flush_rpr_raw(rpr, pending_raw, slot, occurrence);
    occurrences[slot as usize] += 1;
}

fn record_rpr_raw_at(rpr: &mut CT_RPr, raw: Vec<u8>, slot: u8, occurrence: usize) {
    rpr.revision_xml.push(raw);
    rpr.revision_xml_positions.push((slot, occurrence));
}

fn flush_rpr_raw(rpr: &mut CT_RPr, pending_raw: &mut Vec<Vec<u8>>, slot: u8, occurrence: usize) {
    for raw in pending_raw.drain(..) {
        rpr.revision_xml.push(raw);
        rpr.revision_xml_positions.push((slot, occurrence));
    }
}

/// `CT_RPr` — Run properties.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_RPr {
    /// Character style ID (rStyle)
    pub style_id: Option<String>,
    /// Font name for ASCII range (rFonts/@w:ascii)
    pub font_ascii: Option<String>,
    /// Font name for high-ANSI range (rFonts/@w:hAnsi)
    pub font_hansi: Option<String>,
    /// Font name for East Asian text (rFonts/@w:eastAsia)
    pub font_east_asia: Option<String>,
    /// Font name for complex script (rFonts/@w:cs)
    pub font_cs: Option<String>,
    /// Theme font for ASCII range (rFonts/@w:asciiTheme), e.g. "minorHAnsi", "majorHAnsi"
    pub font_ascii_theme: Option<String>,
    /// Theme font for hAnsi range (rFonts/@w:hAnsiTheme)
    pub font_hansi_theme: Option<String>,
    /// Bold (b)
    pub bold: Option<bool>,
    /// Bold complex script (bCs)
    pub bold_cs: Option<bool>,
    /// Italic (i)
    pub italic: Option<bool>,
    /// Italic complex script (iCs)
    pub italic_cs: Option<bool>,
    /// Underline type (u)
    pub underline: Option<ST_Underline>,
    /// Strikethrough (strike)
    pub strike: Option<bool>,
    /// Double strikethrough (dstrike)
    pub dstrike: Option<bool>,
    /// Font size in half-points (sz)
    pub sz: Option<HalfPoint>,
    /// Complex-script font size in half-points (szCs)
    pub sz_cs: Option<HalfPoint>,
    /// Text color as hex string, e.g. "FF0000" (color/@w:val)
    pub color: Option<String>,
    /// Color theme reference (color/@w:themeColor)
    pub color_theme: Option<String>,
    /// Highlight color (highlight)
    pub highlight: Option<ST_HighlightColor>,
    /// All caps (caps)
    pub caps: Option<bool>,
    /// Small caps (smallCaps)
    pub small_caps: Option<bool>,
    /// Superscript/subscript (vertAlign)
    pub vert_align: Option<String>,
    /// Character spacing in twips (spacing/@w:val)
    pub spacing: Option<Twips>,
    /// Character width scale in percent (w/@w:val)
    pub width_scale: Option<u32>,
    /// Text position (raised/lowered) in half-points (position/@w:val)
    pub position: Option<i32>,
    /// Run shading (shd)
    pub shading: Option<CT_Shd>,
    /// Vanish/hidden text (vanish)
    pub vanish: Option<bool>,
    /// Character-level right-to-left direction (rtl).
    pub rtl: Option<bool>,
    /// Language for Latin and high-ANSI text (`lang/@w:val`).
    pub language: Option<String>,
    /// Language for East Asian text (`lang/@w:eastAsia`).
    pub language_east_asia: Option<String>,
    /// Language for complex-script text (`lang/@w:bidi`).
    pub language_bidi: Option<String>,
    /// Namespace declarations and foreign attributes retained from `w:lang`.
    #[doc(hidden)]
    pub language_extra_attributes: Vec<(String, String)>,
    /// Contextual insertion and deletion markers retained in schema order.
    pub revision_markers: Vec<CT_Revision>,
    /// Prior run properties from the schema-final `w:rPrChange`.
    pub change: Option<CT_Revision>,
    /// Foreign and unmodelled children retained without a typed projection.
    pub revision_xml: Vec<Vec<u8>>,
    /// Schema slots and occurrences for retained raw children.
    #[doc(hidden)]
    pub revision_xml_positions: Vec<(u8, usize)>,
}

#[allow(non_snake_case)]
impl CT_RPr {
    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_prefixes(reader, &["w".to_owned()])
    }

    pub(crate) fn from_xml_with_prefixes(
        reader: &mut Reader<&[u8]>,
        word_prefixes: &[String],
    ) -> Result<Self> {
        Self::from_xml_with_prefixes_and_owner_bindings(reader, word_prefixes, &[])
    }

    pub(crate) fn from_xml_with_prefixes_and_owner_bindings(
        reader: &mut Reader<&[u8]>,
        word_prefixes: &[String],
        owner_bindings: &[(String, String)],
    ) -> Result<Self> {
        let mut rpr = CT_RPr::default();
        let mut change_raw_index = 0usize;
        let mut pending_raw = Vec::new();
        let mut occurrences = [0usize; RPR_END_SLOT as usize + 1];
        let mut rtl_carrier_required = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if is_word_element(name.as_ref(), b"rStyle", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_STYLE_SLOT,
                        );
                        rpr.style_id = get_word_val_attr(e, &prefixes)?;
                    } else if is_word_element(name.as_ref(), b"rFonts", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_FONTS_SLOT,
                        );
                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = attr.key.as_ref();
                            let val = std::str::from_utf8(&attr.value)?.to_string();
                            if is_word_attribute(key, b"ascii", &prefixes) {
                                rpr.font_ascii = Some(val);
                            } else if is_word_attribute(key, b"hAnsi", &prefixes) {
                                rpr.font_hansi = Some(val);
                            } else if is_word_attribute(key, b"eastAsia", &prefixes) {
                                rpr.font_east_asia = Some(val);
                            } else if is_word_attribute(key, b"cs", &prefixes) {
                                rpr.font_cs = Some(val);
                            } else if is_word_attribute(key, b"asciiTheme", &prefixes) {
                                rpr.font_ascii_theme = Some(val);
                            } else if is_word_attribute(key, b"hAnsiTheme", &prefixes) {
                                rpr.font_hansi_theme = Some(val);
                            }
                        }
                    } else if is_word_element(name.as_ref(), b"b", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_BOLD_SLOT,
                        );
                        rpr.bold = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"bCs", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_BOLD_CS_SLOT,
                        );
                        rpr.bold_cs = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"i", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_ITALIC_SLOT,
                        );
                        rpr.italic = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"iCs", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_ITALIC_CS_SLOT,
                        );
                        rpr.italic_cs = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"u", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_UNDERLINE_SLOT,
                        );
                        if let Some(val) = get_word_val_attr(e, &prefixes)? {
                            rpr.underline = ST_Underline::from_str(&val).ok();
                        } else {
                            rpr.underline = Some(ST_Underline::Single);
                        }
                    } else if is_word_element(name.as_ref(), b"strike", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_STRIKE_SLOT,
                        );
                        rpr.strike = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"dstrike", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_DSTRIKE_SLOT,
                        );
                        rpr.dstrike = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"sz", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_SIZE_SLOT,
                        );
                        if let Some(val) = get_word_val_attr(e, &prefixes)? {
                            rpr.sz = Some(HalfPoint(val.parse()?));
                        }
                    } else if is_word_element(name.as_ref(), b"szCs", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_SIZE_CS_SLOT,
                        );
                        if let Some(val) = get_word_val_attr(e, &prefixes)? {
                            rpr.sz_cs = Some(HalfPoint(val.parse()?));
                        }
                    } else if is_word_element(name.as_ref(), b"color", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_COLOR_SLOT,
                        );
                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = attr.key.as_ref();
                            let v = std::str::from_utf8(&attr.value)?.to_string();
                            if is_word_attribute(key, b"val", &prefixes) {
                                rpr.color = Some(v);
                            } else if is_word_attribute(key, b"themeColor", &prefixes) {
                                rpr.color_theme = Some(v);
                            }
                        }
                    } else if is_word_element(name.as_ref(), b"highlight", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_HIGHLIGHT_SLOT,
                        );
                        if let Some(val) = get_word_val_attr(e, &prefixes)? {
                            rpr.highlight = ST_HighlightColor::from_str(&val).ok();
                        }
                    } else if is_word_element(name.as_ref(), b"caps", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_CAPS_SLOT,
                        );
                        rpr.caps = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"smallCaps", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_SMALL_CAPS_SLOT,
                        );
                        rpr.small_caps = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"vertAlign", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_VERT_ALIGN_SLOT,
                        );
                        rpr.vert_align = get_word_val_attr(e, &prefixes)?;
                    } else if is_word_element(name.as_ref(), b"spacing", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_SPACING_SLOT,
                        );
                        if let Some(val) = get_word_val_attr(e, &prefixes)? {
                            rpr.spacing = Some(Twips(val.parse()?));
                        }
                    } else if is_word_element(name.as_ref(), b"w", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_WIDTH_SLOT,
                        );
                        if let Some(val) = get_word_val_attr(e, &prefixes)? {
                            rpr.width_scale = Some(val.parse()?);
                        }
                    } else if is_word_element(name.as_ref(), b"position", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_POSITION_SLOT,
                        );
                        if let Some(val) = get_word_val_attr(e, &prefixes)? {
                            rpr.position = Some(val.parse()?);
                        }
                    } else if is_word_element(name.as_ref(), b"shd", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_SHADING_SLOT,
                        );
                        rpr.shading = Some(CT_Shd::from_xml_attrs(e)?);
                    } else if is_word_element(name.as_ref(), b"vanish", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_VANISH_SLOT,
                        );
                        rpr.vanish = Some(parse_word_toggle(e, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"rtl", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_RTL_SLOT,
                        );
                        rpr.rtl = Some(parse_word_toggle(e, &prefixes)?);
                        rtl_carrier_required = toggle_has_unsupported_attributes(e, &prefixes)?;
                        let raw = crate::text::raw_with_external_bindings(
                            &capture_empty_element(e)?,
                            owner_bindings,
                        )?;
                        record_modeled_toggle_candidate(
                            &mut rpr.revision_xml,
                            &mut rpr.revision_xml_positions,
                            raw,
                            RPR_RTL_SLOT,
                            occurrences[RPR_RTL_SLOT as usize] - 1,
                        );
                    } else if is_word_element(name.as_ref(), b"lang", &prefixes) {
                        record_rpr_modeled(
                            &mut rpr,
                            &mut pending_raw,
                            &mut occurrences,
                            RPR_LANG_SLOT,
                        );
                        parse_language_attributes(&mut rpr, e, &prefixes)?;
                    } else if is_word_element(name.as_ref(), b"ins", &prefixes)
                        || is_word_element(name.as_ref(), b"del", &prefixes)
                    {
                        let raw = crate::text::raw_with_external_bindings(
                            &capture_empty_element(e)?,
                            owner_bindings,
                        )?;
                        if let Some(revision) = CT_Revision::from_raw(raw.clone(), &prefixes) {
                            record_rpr_modeled(
                                &mut rpr,
                                &mut pending_raw,
                                &mut occurrences,
                                RPR_MARKER_SLOT,
                            );
                            rpr.revision_markers.push(revision);
                        } else {
                            pending_raw.push(raw);
                        }
                    } else if is_word_element(name.as_ref(), b"rPrChange", &prefixes) {
                        let raw = crate::text::raw_with_external_bindings(
                            &capture_empty_element(e)?,
                            owner_bindings,
                        )?;
                        if let Some(revision) = CT_Revision::from_raw(raw.clone(), &prefixes) {
                            if let Some(previous) = rpr.change.take() {
                                rpr.revision_xml
                                    .insert(change_raw_index, previous.into_raw_xml());
                                rpr.revision_xml_positions
                                    .insert(change_raw_index, (RPR_CHANGE_SLOT, 0));
                            }
                            record_rpr_modeled(
                                &mut rpr,
                                &mut pending_raw,
                                &mut occurrences,
                                RPR_CHANGE_SLOT,
                            );
                            rpr.change = Some(revision);
                            change_raw_index = rpr.revision_xml.len();
                        } else {
                            flush_rpr_raw(&mut rpr, &mut pending_raw, RPR_CHANGE_SLOT, 0);
                            record_rpr_raw_at(&mut rpr, raw, RPR_CHANGE_SLOT, 0);
                        }
                    } else {
                        let raw = crate::text::raw_with_external_bindings(
                            &capture_empty_element(e)?,
                            owner_bindings,
                        )?;
                        if let Some(slot) = rpr_schema_slot(name.as_ref(), &prefixes) {
                            record_rpr_raw_at(&mut rpr, raw, slot, 0);
                        } else {
                            pending_raw.push(raw);
                        }
                    }
                }
                Ok(Event::Start(ref e)) => {
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if is_word_element(e.name().as_ref(), b"rtl", &prefixes) {
                        let captured = capture_element(reader, e)?;
                        let raw =
                            crate::text::raw_with_external_bindings(&captured, owner_bindings)?;
                        if toggle_element_is_explicitly_empty(&captured)? {
                            record_rpr_modeled(
                                &mut rpr,
                                &mut pending_raw,
                                &mut occurrences,
                                RPR_RTL_SLOT,
                            );
                            rpr.rtl = Some(parse_word_toggle(e, &prefixes)?);
                            rtl_carrier_required = toggle_has_unsupported_attributes(e, &prefixes)?;
                            record_modeled_toggle_candidate(
                                &mut rpr.revision_xml,
                                &mut rpr.revision_xml_positions,
                                raw,
                                RPR_RTL_SLOT,
                                occurrences[RPR_RTL_SLOT as usize] - 1,
                            );
                        } else {
                            let occurrence = occurrences[RPR_RTL_SLOT as usize];
                            occurrences[RPR_RTL_SLOT as usize] += 1;
                            record_rpr_raw_at(&mut rpr, raw, RPR_RTL_SLOT, occurrence);
                        }
                    } else if is_word_element(e.name().as_ref(), b"lang", &prefixes) {
                        let captured = capture_element(reader, e)?;
                        let raw =
                            crate::text::raw_with_external_bindings(&captured, owner_bindings)?;
                        if language_element_is_explicitly_empty(&captured)? {
                            record_rpr_modeled(
                                &mut rpr,
                                &mut pending_raw,
                                &mut occurrences,
                                RPR_LANG_SLOT,
                            );
                            parse_language_attributes(&mut rpr, e, &prefixes)?;
                        } else {
                            record_rpr_raw_at(
                                &mut rpr,
                                raw,
                                RPR_LANG_SLOT,
                                occurrences[RPR_LANG_SLOT as usize],
                            );
                        }
                    } else if is_word_element(e.name().as_ref(), b"ins", &prefixes)
                        || is_word_element(e.name().as_ref(), b"del", &prefixes)
                    {
                        let raw = crate::text::raw_with_external_bindings(
                            &capture_element(reader, e)?,
                            owner_bindings,
                        )?;
                        if let Some(revision) = CT_Revision::from_raw(raw.clone(), &prefixes) {
                            record_rpr_modeled(
                                &mut rpr,
                                &mut pending_raw,
                                &mut occurrences,
                                RPR_MARKER_SLOT,
                            );
                            rpr.revision_markers.push(revision);
                        } else {
                            pending_raw.push(raw);
                        }
                    } else if is_word_element(e.name().as_ref(), b"rPrChange", &prefixes) {
                        let raw = crate::text::raw_with_external_bindings(
                            &capture_element(reader, e)?,
                            owner_bindings,
                        )?;
                        if let Some(revision) = CT_Revision::from_raw(raw.clone(), &prefixes) {
                            if let Some(previous) = rpr.change.take() {
                                rpr.revision_xml
                                    .insert(change_raw_index, previous.into_raw_xml());
                                rpr.revision_xml_positions
                                    .insert(change_raw_index, (RPR_CHANGE_SLOT, 0));
                            }
                            record_rpr_modeled(
                                &mut rpr,
                                &mut pending_raw,
                                &mut occurrences,
                                RPR_CHANGE_SLOT,
                            );
                            rpr.change = Some(revision);
                            change_raw_index = rpr.revision_xml.len();
                        } else {
                            flush_rpr_raw(&mut rpr, &mut pending_raw, RPR_CHANGE_SLOT, 0);
                            record_rpr_raw_at(&mut rpr, raw, RPR_CHANGE_SLOT, 0);
                        }
                    } else {
                        let raw = crate::text::raw_with_external_bindings(
                            &capture_element(reader, e)?,
                            owner_bindings,
                        )?;
                        if let Some(slot) = rpr_schema_slot(e.name().as_ref(), &prefixes) {
                            record_rpr_raw_at(&mut rpr, raw, slot, 0);
                        } else {
                            pending_raw.push(raw);
                        }
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"rPr") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        let final_slot = if rpr.change.is_some() {
            RPR_CHANGE_SLOT
        } else {
            RPR_END_SLOT
        };
        flush_rpr_raw(&mut rpr, &mut pending_raw, final_slot, 0);
        remove_redundant_modeled_toggle_candidate(
            &mut rpr.revision_xml,
            &mut rpr.revision_xml_positions,
            RPR_RTL_SLOT,
            rtl_carrier_required,
        );

        Ok(rpr)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.to_xml_with_word_override(writer, None)
    }

    pub(crate) fn to_xml_with_word_override<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        foreign_word_namespace: Option<&str>,
    ) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }

        if !self.revision_xml.is_empty()
            && self.revision_xml_positions.len() == self.revision_xml.len()
        {
            let mut modeled = self.clone();
            modeled.revision_xml.clear();
            modeled.revision_xml_positions.clear();
            let mut generated = Writer::new(Vec::new());
            if modeled.is_empty() {
                generated.write_event(Event::Start(BytesStart::new("w:rPr")))?;
                generated.write_event(Event::End(BytesEnd::new("w:rPr")))?;
            } else {
                modeled.to_xml_with_word_override(&mut generated, foreign_word_namespace)?;
            }
            return write_rpr_with_positioned_raw(
                writer,
                &generated.into_inner(),
                self,
                foreign_word_namespace,
            );
        }

        let mut buf = itoa::Buffer::new();
        writer.write_event(Event::Start(BytesStart::new("w:rPr")))?;

        if let Some(ref style_id) = self.style_id {
            let mut e = BytesStart::new("w:rStyle");
            e.push_attribute(("w:val", style_id.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        // rFonts
        if self.font_ascii.is_some()
            || self.font_hansi.is_some()
            || self.font_east_asia.is_some()
            || self.font_cs.is_some()
            || self.font_ascii_theme.is_some()
            || self.font_hansi_theme.is_some()
        {
            let mut e = BytesStart::new("w:rFonts");
            if let Some(ref f) = self.font_ascii {
                e.push_attribute(("w:ascii", f.as_str()));
            }
            if let Some(ref f) = self.font_hansi {
                e.push_attribute(("w:hAnsi", f.as_str()));
            }
            if let Some(ref f) = self.font_east_asia {
                e.push_attribute(("w:eastAsia", f.as_str()));
            }
            if let Some(ref f) = self.font_cs {
                e.push_attribute(("w:cs", f.as_str()));
            }
            if let Some(ref f) = self.font_ascii_theme {
                e.push_attribute(("w:asciiTheme", f.as_str()));
            }
            if let Some(ref f) = self.font_hansi_theme {
                e.push_attribute(("w:hAnsiTheme", f.as_str()));
            }
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(bold) = self.bold {
            write_toggle(writer, "w:b", bold)?;
        }
        if let Some(bold_cs) = self.bold_cs {
            write_toggle(writer, "w:bCs", bold_cs)?;
        }
        if let Some(italic) = self.italic {
            write_toggle(writer, "w:i", italic)?;
        }
        if let Some(italic_cs) = self.italic_cs {
            write_toggle(writer, "w:iCs", italic_cs)?;
        }
        if let Some(caps) = self.caps {
            write_toggle(writer, "w:caps", caps)?;
        }
        if let Some(small_caps) = self.small_caps {
            write_toggle(writer, "w:smallCaps", small_caps)?;
        }
        if let Some(strike) = self.strike {
            write_toggle(writer, "w:strike", strike)?;
        }
        if let Some(dstrike) = self.dstrike {
            write_toggle(writer, "w:dstrike", dstrike)?;
        }
        if let Some(vanish) = self.vanish {
            write_toggle(writer, "w:vanish", vanish)?;
        }

        if let Some(ref color) = self.color {
            let mut e = BytesStart::new("w:color");
            e.push_attribute(("w:val", color.as_str()));
            if let Some(ref tc) = self.color_theme {
                e.push_attribute(("w:themeColor", tc.as_str()));
            }
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref spacing) = self.spacing {
            let mut e = BytesStart::new("w:spacing");
            e.push_attribute(("w:val", buf.format(spacing.0)));
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ws) = self.width_scale {
            let mut e = BytesStart::new("w:w");
            e.push_attribute(("w:val", buf.format(ws)));
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(pos) = self.position {
            let mut e = BytesStart::new("w:position");
            e.push_attribute(("w:val", buf.format(pos)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref sz) = self.sz {
            let mut e = BytesStart::new("w:sz");
            e.push_attribute(("w:val", buf.format(sz.0)));
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ref sz_cs) = self.sz_cs {
            let mut e = BytesStart::new("w:szCs");
            e.push_attribute(("w:val", buf.format(sz_cs.0)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref highlight) = self.highlight {
            let mut e = BytesStart::new("w:highlight");
            e.push_attribute(("w:val", highlight.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(underline) = self.underline {
            let mut e = BytesStart::new("w:u");
            e.push_attribute(("w:val", underline.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref shd) = self.shading {
            shd.write_xml(writer, "w:shd")?;
        }

        if let Some(ref vert_align) = self.vert_align {
            let mut e = BytesStart::new("w:vertAlign");
            e.push_attribute(("w:val", vert_align.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(rtl) = self.rtl {
            write_toggle(writer, "w:rtl", rtl)?;
        }

        if self.language.is_some()
            || self.language_east_asia.is_some()
            || self.language_bidi.is_some()
            || !self.language_extra_attributes.is_empty()
        {
            let mut e = BytesStart::new("w:lang");
            if let Some(language) = &self.language {
                e.push_attribute(("w:val", language.as_str()));
            }
            if let Some(language) = &self.language_east_asia {
                e.push_attribute(("w:eastAsia", language.as_str()));
            }
            if let Some(language) = &self.language_bidi {
                e.push_attribute(("w:bidi", language.as_str()));
            }
            for (name, value) in &self.language_extra_attributes {
                e.push_attribute((name.as_str(), value.as_str()));
            }
            writer.write_event(Event::Empty(e))?;
        }

        for revision in &self.revision_markers {
            revision.write_xml_with_word_override(writer, foreign_word_namespace)?;
        }
        for raw in &self.revision_xml {
            crate::text::write_raw_with_word_override(writer, raw, foreign_word_namespace)?;
        }
        if let Some(change) = &self.change {
            change.write_xml_with_word_override(writer, foreign_word_namespace)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:rPr")))?;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.style_id.is_none()
            && self.font_ascii.is_none()
            && self.font_hansi.is_none()
            && self.font_east_asia.is_none()
            && self.font_cs.is_none()
            && self.font_ascii_theme.is_none()
            && self.font_hansi_theme.is_none()
            && self.bold.is_none()
            && self.bold_cs.is_none()
            && self.italic.is_none()
            && self.italic_cs.is_none()
            && self.underline.is_none()
            && self.strike.is_none()
            && self.dstrike.is_none()
            && self.sz.is_none()
            && self.sz_cs.is_none()
            && self.color.is_none()
            && self.color_theme.is_none()
            && self.highlight.is_none()
            && self.caps.is_none()
            && self.small_caps.is_none()
            && self.vert_align.is_none()
            && self.spacing.is_none()
            && self.width_scale.is_none()
            && self.position.is_none()
            && self.shading.is_none()
            && self.vanish.is_none()
            && self.rtl.is_none()
            && self.language.is_none()
            && self.language_east_asia.is_none()
            && self.language_bidi.is_none()
            && self.language_extra_attributes.is_empty()
            && self.revision_markers.is_empty()
            && self.change.is_none()
            && self.revision_xml.is_empty()
    }

    /// Merge another CT_RPr into this one (non-None fields override).
    /// Used for style inheritance.
    pub fn merge_from(&mut self, other: &CT_RPr) {
        if other.style_id.is_some() {
            self.style_id = other.style_id.clone();
        }
        if other.font_ascii.is_some() {
            self.font_ascii = other.font_ascii.clone();
        }
        if other.font_hansi.is_some() {
            self.font_hansi = other.font_hansi.clone();
        }
        if other.font_east_asia.is_some() {
            self.font_east_asia = other.font_east_asia.clone();
        }
        if other.font_cs.is_some() {
            self.font_cs = other.font_cs.clone();
        }
        if other.font_ascii_theme.is_some() {
            self.font_ascii_theme = other.font_ascii_theme.clone();
        }
        if other.font_hansi_theme.is_some() {
            self.font_hansi_theme = other.font_hansi_theme.clone();
        }
        if other.bold.is_some() {
            self.bold = other.bold;
        }
        if other.bold_cs.is_some() {
            self.bold_cs = other.bold_cs;
        }
        if other.italic.is_some() {
            self.italic = other.italic;
        }
        if other.italic_cs.is_some() {
            self.italic_cs = other.italic_cs;
        }
        if other.underline.is_some() {
            self.underline = other.underline;
        }
        if other.strike.is_some() {
            self.strike = other.strike;
        }
        if other.dstrike.is_some() {
            self.dstrike = other.dstrike;
        }
        if other.sz.is_some() {
            self.sz = other.sz;
        }
        if other.sz_cs.is_some() {
            self.sz_cs = other.sz_cs;
        }
        if other.color.is_some() {
            self.color = other.color.clone();
        }
        if other.color_theme.is_some() {
            self.color_theme = other.color_theme.clone();
        }
        if other.highlight.is_some() {
            self.highlight = other.highlight;
        }
        if other.caps.is_some() {
            self.caps = other.caps;
        }
        if other.small_caps.is_some() {
            self.small_caps = other.small_caps;
        }
        if other.vert_align.is_some() {
            self.vert_align = other.vert_align.clone();
        }
        if other.spacing.is_some() {
            self.spacing = other.spacing;
        }
        if other.width_scale.is_some() {
            self.width_scale = other.width_scale;
        }
        if other.position.is_some() {
            self.position = other.position;
        }
        if other.shading.is_some() {
            self.shading = other.shading.clone();
        }
        if other.vanish.is_some() {
            self.vanish = other.vanish;
        }
        if other.rtl.is_some() {
            self.rtl = other.rtl;
        }
        if other.language.is_some() {
            self.language = other.language.clone();
        }
        if other.language_east_asia.is_some() {
            self.language_east_asia = other.language_east_asia.clone();
        }
        if other.language_bidi.is_some() {
            self.language_bidi = other.language_bidi.clone();
        }
        if !other.language_extra_attributes.is_empty() {
            self.language_extra_attributes = other.language_extra_attributes.clone();
        }
    }
}

fn parse_language_attributes(
    rpr: &mut CT_RPr,
    element: &BytesStart<'_>,
    prefixes: &[String],
) -> Result<()> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
            .into_owned();
        if is_word_attribute(key, b"val", prefixes) {
            rpr.language = Some(value);
        } else if is_word_attribute(key, b"eastAsia", prefixes) {
            rpr.language_east_asia = Some(value);
        } else if is_word_attribute(key, b"bidi", prefixes) {
            rpr.language_bidi = Some(value);
        } else {
            rpr.language_extra_attributes
                .push((std::str::from_utf8(key)?.to_owned(), value));
        }
    }
    Ok(())
}

pub(crate) fn language_element_is_explicitly_empty(xml: &[u8]) -> Result<bool> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    if !matches!(reader.read_event_into(&mut buffer)?, Event::Start(_)) {
        return Ok(false);
    }
    buffer.clear();
    Ok(matches!(
        reader.read_event_into(&mut buffer)?,
        Event::End(_)
    ))
}

fn write_rpr_with_positioned_raw<W: std::io::Write>(
    writer: &mut Writer<W>,
    generated: &[u8],
    rpr: &CT_RPr,
    foreign_word_namespace: Option<&str>,
) -> Result<()> {
    let mut raw_order = (0..rpr.revision_xml.len())
        .filter(|index| {
            replay_modeled_toggle_raw(
                rpr.revision_xml_positions[*index],
                rpr.revision_xml_positions[*index].0 != RPR_RTL_SLOT || rpr.rtl.is_some(),
            )
        })
        .collect::<Vec<_>>();
    raw_order.sort_by_key(|index| {
        let (slot, occurrence) = effective_rpr_raw_position(rpr, *index);
        (slot, occurrence, *index)
    });
    let mut raw_index = 0usize;
    let mut occurrences = [0usize; RPR_END_SLOT as usize + 1];
    let mut reader = Reader::from_reader(generated);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut inside = false;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if !inside => {
                inside = true;
                writer.write_event(Event::Start(element.into_owned()))?;
            }
            Event::Start(element) => {
                let slot = rpr_slot_for_name(element.local_name().as_ref());
                write_rpr_raw_before(
                    writer,
                    rpr,
                    &raw_order,
                    &mut raw_index,
                    slot,
                    occurrences[slot as usize],
                    foreign_word_namespace,
                )?;
                let raw = capture_element(&mut reader, &element)?;
                writer.get_mut().write_all(&raw)?;
                occurrences[slot as usize] += 1;
            }
            Event::Empty(mut element) => {
                let slot = rpr_slot_for_name(element.local_name().as_ref());
                write_rpr_raw_before(
                    writer,
                    rpr,
                    &raw_order,
                    &mut raw_index,
                    slot,
                    occurrences[slot as usize],
                    foreign_word_namespace,
                )?;
                append_modeled_toggle_attributes(
                    &mut element,
                    &rpr.revision_xml,
                    &rpr.revision_xml_positions,
                    slot,
                    occurrences[slot as usize],
                )?;
                writer.write_event(Event::Empty(element.into_owned()))?;
                occurrences[slot as usize] += 1;
            }
            Event::End(element) if inside => {
                write_rpr_raw_before(
                    writer,
                    rpr,
                    &raw_order,
                    &mut raw_index,
                    RPR_END_SLOT,
                    0,
                    foreign_word_namespace,
                )?;
                while let Some(index) = raw_order.get(raw_index).copied() {
                    crate::text::write_raw_with_word_override(
                        writer,
                        &rpr.revision_xml[index],
                        foreign_word_namespace,
                    )?;
                    raw_index += 1;
                }
                writer.write_event(Event::End(element.into_owned()))?;
                return Ok(());
            }
            Event::Eof => return Ok(()),
            event => writer.write_event(event.into_owned())?,
        }
        buffer.clear();
    }
}

fn write_rpr_raw_before<W: std::io::Write>(
    writer: &mut Writer<W>,
    rpr: &CT_RPr,
    raw_order: &[usize],
    raw_index: &mut usize,
    slot: u8,
    occurrence: usize,
    foreign_word_namespace: Option<&str>,
) -> Result<()> {
    while let Some(index) = raw_order.get(*raw_index).copied() {
        let position = effective_rpr_raw_position(rpr, index);
        if position.0 > slot || (position.0 == slot && position.1 > occurrence) {
            break;
        }
        crate::text::write_raw_with_word_override(
            writer,
            &rpr.revision_xml[index],
            foreign_word_namespace,
        )?;
        *raw_index += 1;
    }
    Ok(())
}

fn effective_rpr_raw_position(rpr: &CT_RPr, index: usize) -> (u8, usize) {
    let mut position = rpr.revision_xml_positions[index];
    position.1 = raw_occurrence(position);
    if position.0 == RPR_RTL_SLOT
        && let Some((carrier_index, carrier_occurrence)) = rpr
            .revision_xml_positions
            .iter()
            .enumerate()
            .find(|candidate| {
                candidate.1.0 == RPR_RTL_SLOT && raw_is_modeled_attribute_carrier(*candidate.1)
            })
            .map(|(carrier_index, candidate)| (carrier_index, raw_occurrence(*candidate)))
    {
        position.1 = usize::from(
            position.1 > carrier_occurrence
                || (position.1 == carrier_occurrence && index > carrier_index),
        );
    }
    if rpr.change.is_some() && position.0 >= RPR_CHANGE_SLOT {
        (RPR_CHANGE_SLOT, 0)
    } else {
        position
    }
}

fn rpr_slot_for_name(local: &[u8]) -> u8 {
    match local {
        b"rStyle" => RPR_STYLE_SLOT,
        b"rFonts" => RPR_FONTS_SLOT,
        b"b" => RPR_BOLD_SLOT,
        b"bCs" => RPR_BOLD_CS_SLOT,
        b"i" => RPR_ITALIC_SLOT,
        b"iCs" => RPR_ITALIC_CS_SLOT,
        b"caps" => RPR_CAPS_SLOT,
        b"smallCaps" => RPR_SMALL_CAPS_SLOT,
        b"strike" => RPR_STRIKE_SLOT,
        b"dstrike" => RPR_DSTRIKE_SLOT,
        b"outline" => 10,
        b"shadow" => 11,
        b"emboss" => 12,
        b"imprint" => 13,
        b"noProof" => 14,
        b"snapToGrid" => 15,
        b"vanish" => RPR_VANISH_SLOT,
        b"webHidden" => 17,
        b"color" => RPR_COLOR_SLOT,
        b"spacing" => RPR_SPACING_SLOT,
        b"w" => RPR_WIDTH_SLOT,
        b"kern" => 21,
        b"position" => RPR_POSITION_SLOT,
        b"sz" => RPR_SIZE_SLOT,
        b"szCs" => RPR_SIZE_CS_SLOT,
        b"highlight" => RPR_HIGHLIGHT_SLOT,
        b"u" => RPR_UNDERLINE_SLOT,
        b"effect" => 27,
        b"bdr" => 28,
        b"shd" => RPR_SHADING_SLOT,
        b"fitText" => 30,
        b"vertAlign" => RPR_VERT_ALIGN_SLOT,
        b"rtl" => 32,
        b"cs" => 33,
        b"em" => 34,
        b"lang" => RPR_LANG_SLOT,
        b"eastAsianLayout" => 36,
        b"specVanish" => 37,
        b"oMath" => 38,
        b"ins" | b"del" => RPR_MARKER_SLOT,
        b"rPrChange" => RPR_CHANGE_SLOT,
        _ => RPR_END_SLOT,
    }
}

fn rpr_schema_slot(name: &[u8], word_prefixes: &[String]) -> Option<u8> {
    let local = name.rsplit(|byte| *byte == b':').next().unwrap_or(name);
    let slot = rpr_slot_for_name(local);
    (slot != RPR_END_SLOT && is_word_element(name, local, word_prefixes)).then_some(slot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::W_NS;

    fn parse_rpr(xml: &str) -> CT_RPr {
        let full = format!("<w:rPr>{xml}</w:rPr>");
        let mut reader = Reader::from_str(&full);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"rPr") => break,
                _ => {}
            }
            buf.clear();
        }
        CT_RPr::from_xml(&mut reader).unwrap()
    }

    #[test]
    fn parse_basic_rpr() {
        let rpr = parse_rpr(r#"<w:b/><w:i/><w:sz w:val="24"/><w:color w:val="FF0000"/>"#);
        assert_eq!(rpr.bold, Some(true));
        assert_eq!(rpr.italic, Some(true));
        assert_eq!(rpr.sz, Some(HalfPoint(24)));
        assert_eq!(rpr.color, Some("FF0000".to_string()));
    }

    #[test]
    fn parse_rpr_spacing() {
        let rpr = parse_rpr(r#"<w:spacing w:val="20"/><w:w w:val="150"/><w:position w:val="-4"/>"#);
        assert_eq!(rpr.spacing, Some(Twips(20)));
        assert_eq!(rpr.width_scale, Some(150));
        assert_eq!(rpr.position, Some(-4));
    }

    #[test]
    fn round_trip_rpr() {
        let original = CT_RPr {
            bold: Some(true),
            italic: Some(true),
            sz: Some(HalfPoint(24)),
            color: Some("FF0000".to_string()),
            underline: Some(ST_Underline::Single),
            spacing: Some(Twips(20)),
            ..Default::default()
        };

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        original.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let inner = xml
            .strip_prefix("<w:rPr>")
            .unwrap()
            .strip_suffix("</w:rPr>")
            .unwrap();
        let parsed = parse_rpr(inner);
        assert_eq!(parsed.bold, original.bold);
        assert_eq!(parsed.italic, original.italic);
        assert_eq!(parsed.sz, original.sz);
        assert_eq!(parsed.color, original.color);
        assert_eq!(parsed.underline, original.underline);
        assert_eq!(parsed.spacing, original.spacing);
    }

    #[test]
    fn merge_rpr() {
        let mut base = CT_RPr {
            bold: Some(true),
            sz: Some(HalfPoint(24)),
            ..Default::default()
        };
        let override_rpr = CT_RPr {
            sz: Some(HalfPoint(28)),
            italic: Some(true),
            ..Default::default()
        };
        base.merge_from(&override_rpr);
        assert_eq!(base.bold, Some(true)); // kept
        assert_eq!(base.sz, Some(HalfPoint(28))); // overridden
        assert_eq!(base.italic, Some(true)); // added
    }

    #[test]
    fn run_language_parses_aliases_and_rejects_foreign_same_local_attributes() {
        let rpr = parse_rpr(&format!(
            r#"<q:lang xmlns:q="{}" xmlns:x="urn:foreign" q:val="en-US" q:eastAsia="zh-CN" q:bidi="ar-SA" x:val="ignored" x:kept="raw"/>"#,
            W_NS
        ));
        assert_eq!(rpr.language.as_deref(), Some("en-US"));
        assert_eq!(rpr.language_east_asia.as_deref(), Some("zh-CN"));
        assert_eq!(rpr.language_bidi.as_deref(), Some("ar-SA"));

        let mut output = Vec::new();
        rpr.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains(r#"<w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA""#));
        assert!(output.contains(r#"x:kept="raw""#));
        assert!(!output.contains(r#"w:val="ignored""#));

        let explicit = parse_rpr(&format!(
            r#"<q:lang xmlns:q="{}" q:val="en-GB"></q:lang>"#,
            W_NS
        ));
        assert_eq!(explicit.language.as_deref(), Some("en-GB"));
    }

    #[test]
    fn run_language_round_trips_at_its_schema_slot_and_cascades() {
        let mut base = CT_RPr {
            language: Some("fr-FR".to_owned()),
            language_east_asia: Some("ja-JP".to_owned()),
            ..Default::default()
        };
        base.merge_from(&CT_RPr {
            language: Some("de-DE".to_owned()),
            language_bidi: Some("ar-SA".to_owned()),
            ..Default::default()
        });
        assert_eq!(base.language.as_deref(), Some("de-DE"));
        assert_eq!(base.language_east_asia.as_deref(), Some("ja-JP"));
        assert_eq!(base.language_bidi.as_deref(), Some("ar-SA"));

        let mut output = Vec::new();
        base.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.find("w:vertAlign").is_none());
        assert!(output.find("w:lang").unwrap() < output.rfind("</w:rPr>").unwrap());
    }

    #[test]
    fn malformed_language_after_the_modeled_occurrence_keeps_its_relative_position() {
        let rpr = parse_rpr(r#"<w:lang w:val="en-US"/><w:lang><w:producerExtension/></w:lang>"#);

        let mut output = Vec::new();
        rpr.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output_text = std::str::from_utf8(&output).unwrap();
        assert!(
            output_text.find(r#"w:val="en-US""#).unwrap()
                < output_text.find("w:producerExtension").unwrap()
        );

        let mut reader = Reader::from_reader(output.as_slice());
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer).unwrap() {
                Event::Start(element) if element.local_name().as_ref() == b"rPr" => break,
                Event::Eof => panic!("serialized run properties have no root"),
                _ => {}
            }
            buffer.clear();
        }
        let reparsed = CT_RPr::from_xml(&mut reader).unwrap();
        let mut repeated = Vec::new();
        reparsed.to_xml(&mut Writer::new(&mut repeated)).unwrap();
        assert_eq!(repeated, output);
    }
}
