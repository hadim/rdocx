//! Paragraph properties (`CT_PPr`) and run properties (`CT_RPr`).

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::error::Result;
use crate::namespace::{W_NS, matches_local_name};
use crate::run_properties::language_element_is_explicitly_empty;
use crate::shared::ST_OnOff;

pub use crate::paragraph_properties::{CT_FramePr, CT_PPr};
pub use crate::run_properties::CT_RPr;

/// `CT_Shd` — Shading/background fill.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Shd {
    /// Shading pattern (e.g. "clear", "solid", "horzStripe")
    pub val: String,
    /// Foreground color hex
    pub color: Option<String>,
    /// Background fill color hex
    pub fill: Option<String>,
}

impl CT_Shd {
    pub fn from_xml_attrs(e: &BytesStart) -> Result<Self> {
        let mut val = "clear".to_string();
        let mut color = None;
        let mut fill = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let v = std::str::from_utf8(&attr.value)?;
            if matches_local_name(key, b"val") {
                val = v.to_string();
            } else if matches_local_name(key, b"color") {
                color = Some(v.to_string());
            } else if matches_local_name(key, b"fill") {
                fill = Some(v.to_string());
            }
        }

        Ok(CT_Shd { val, color, fill })
    }

    pub(crate) fn from_xml_attrs_with_prefixes(
        e: &BytesStart,
        word_prefixes: &[String],
    ) -> Result<Self> {
        let mut val = "clear".to_string();
        let mut color = None;
        let mut fill = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let v = std::str::from_utf8(&attr.value)?;
            if is_word_attribute(key, b"val", word_prefixes) {
                val = v.to_string();
            } else if is_word_attribute(key, b"color", word_prefixes) {
                color = Some(v.to_string());
            } else if is_word_attribute(key, b"fill", word_prefixes) {
                fill = Some(v.to_string());
            }
        }

        Ok(CT_Shd { val, color, fill })
    }

    pub fn write_xml<W: std::io::Write>(&self, writer: &mut Writer<W>, tag: &str) -> Result<()> {
        let mut e = BytesStart::new(tag);
        e.push_attribute(("w:val", self.val.as_str()));
        if let Some(ref c) = self.color {
            e.push_attribute(("w:color", c.as_str()));
        }
        if let Some(ref f) = self.fill {
            e.push_attribute(("w:fill", f.as_str()));
        }
        writer.write_event(Event::Empty(e))?;
        Ok(())
    }
}

const RAW_MODELED_ATTRIBUTES_FLAG: usize = 1 << (usize::BITS - 1);
const RAW_MODELED_DUPLICATE_FLAG: usize = 1 << (usize::BITS - 2);
const RAW_MODELED_FLAGS: usize = RAW_MODELED_ATTRIBUTES_FLAG | RAW_MODELED_DUPLICATE_FLAG;

pub(crate) fn raw_occurrence(position: (u8, usize)) -> usize {
    position.1 & !RAW_MODELED_FLAGS
}

pub(crate) fn raw_is_modeled_attribute_carrier(position: (u8, usize)) -> bool {
    position.1 & RAW_MODELED_ATTRIBUTES_FLAG != 0
}

fn modeled_attribute_carrier_occurrence(occurrence: usize) -> usize {
    occurrence | RAW_MODELED_ATTRIBUTES_FLAG
}

fn raw_is_modeled_duplicate(position: (u8, usize)) -> bool {
    position.1 & RAW_MODELED_DUPLICATE_FLAG != 0
}

fn modeled_duplicate_occurrence(occurrence: usize) -> usize {
    occurrence | RAW_MODELED_DUPLICATE_FLAG
}

pub(crate) fn replay_modeled_toggle_raw(
    position: (u8, usize),
    modeled_value_is_present: bool,
) -> bool {
    !raw_is_modeled_attribute_carrier(position)
        && (!raw_is_modeled_duplicate(position) || modeled_value_is_present)
}

pub(crate) fn record_modeled_toggle_candidate(
    raw_xml: &mut Vec<Vec<u8>>,
    positions: &mut Vec<(u8, usize)>,
    raw: Vec<u8>,
    slot: u8,
    occurrence: usize,
) {
    let replacing_carrier = positions
        .iter()
        .any(|position| position.0 == slot && raw_is_modeled_attribute_carrier(*position));
    if replacing_carrier {
        for position in positions
            .iter_mut()
            .filter(|position| position.0 == slot && raw_is_modeled_attribute_carrier(**position))
        {
            position.1 = modeled_duplicate_occurrence(raw_occurrence(*position));
        }
    }
    raw_xml.push(raw);
    positions.push((slot, modeled_attribute_carrier_occurrence(occurrence)));
}

pub(crate) fn remove_redundant_modeled_toggle_candidate(
    raw_xml: &mut Vec<Vec<u8>>,
    positions: &mut Vec<(u8, usize)>,
    slot: u8,
    carrier_required: bool,
) {
    if carrier_required {
        return;
    }
    if let Some(index) = positions
        .iter()
        .rposition(|position| position.0 == slot && raw_is_modeled_attribute_carrier(*position))
    {
        let carrier_occurrence = raw_occurrence(positions[index]);
        for (candidate_index, position) in positions.iter_mut().enumerate() {
            if candidate_index == index || position.0 != slot {
                continue;
            }
            let flags = position.1 & RAW_MODELED_FLAGS;
            let occurrence = raw_occurrence(*position);
            position.1 = flags
                | usize::from(
                    occurrence > carrier_occurrence
                        || (occurrence == carrier_occurrence && candidate_index > index),
                );
        }
        raw_xml.remove(index);
        positions.remove(index);
    }
}

pub(crate) fn toggle_element_is_explicitly_empty(xml: &[u8]) -> Result<bool> {
    language_element_is_explicitly_empty(xml)
}

pub(crate) fn toggle_has_unsupported_attributes(
    element: &BytesStart<'_>,
    word_prefixes: &[String],
) -> Result<bool> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        if !is_word_attribute(attribute.key.as_ref(), b"val", word_prefixes) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn append_modeled_toggle_attributes(
    element: &mut BytesStart<'_>,
    raw_xml: &[Vec<u8>],
    positions: &[(u8, usize)],
    slot: u8,
    _occurrence: usize,
) -> Result<()> {
    for (raw, position) in raw_xml.iter().zip(positions) {
        if position.0 != slot || !raw_is_modeled_attribute_carrier(*position) {
            continue;
        }
        let mut reader = Reader::from_reader(raw.as_slice());
        let mut buffer = Vec::new();
        let source = match reader.read_event_into(&mut buffer)? {
            Event::Start(source) | Event::Empty(source) => source.into_owned(),
            _ => continue,
        };
        let prefixes = word_prefixes_at(&source, &["w".to_owned()])?;
        let mut preserved = Vec::new();
        for attribute in source.attributes() {
            let attribute = attribute?;
            if is_word_attribute(attribute.key.as_ref(), b"val", &prefixes) {
                continue;
            }
            let name = std::str::from_utf8(attribute.key.as_ref())?.to_owned();
            let value = attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, source.decoder())?
                .into_owned();
            preserved.push((name, value));
        }
        for (name, value) in &preserved {
            element.push_attribute((name.as_str(), value.as_str()));
        }
    }
    Ok(())
}

pub(crate) fn word_prefixes_at(
    start: &BytesStart<'_>,
    inherited: &[String],
) -> Result<Vec<String>> {
    let mut prefixes = inherited.to_vec();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let name = attribute.key.as_ref();
        let prefix = if name == b"xmlns" {
            b"".as_slice()
        } else if let Some(prefix) = name.strip_prefix(b"xmlns:") {
            prefix
        } else {
            continue;
        };
        let prefix = std::str::from_utf8(prefix)?.to_string();
        prefixes.retain(|candidate| candidate != &prefix);
        let value =
            attribute.decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?;
        if value.as_bytes() == W_NS.as_bytes() {
            prefixes.push(prefix);
        }
    }
    Ok(prefixes)
}

fn is_word_name(name: &[u8], word_prefixes: &[String]) -> bool {
    let Some(separator) = name.iter().position(|byte| *byte == b':') else {
        return word_prefixes.iter().any(String::is_empty);
    };
    word_prefixes
        .iter()
        .any(|prefix| prefix.as_bytes() == &name[..separator])
}

pub(crate) fn is_word_element(name: &[u8], local: &[u8], word_prefixes: &[String]) -> bool {
    matches_local_name(name, local) && is_word_name(name, word_prefixes)
}

pub(crate) fn is_word_attribute(key: &[u8], local: &[u8], word_prefixes: &[String]) -> bool {
    let Some(separator) = key.iter().position(|byte| *byte == b':') else {
        return false;
    };
    key.get(separator + 1..) == Some(local)
        && word_prefixes
            .iter()
            .any(|prefix| prefix.as_bytes() == &key[..separator])
}

pub(crate) fn get_word_val_attr(
    e: &BytesStart,
    word_prefixes: &[String],
) -> Result<Option<String>> {
    for attr in e.attributes() {
        let attr = attr?;
        if is_word_attribute(attr.key.as_ref(), b"val", word_prefixes) {
            return Ok(Some(std::str::from_utf8(&attr.value)?.to_string()));
        }
    }
    Ok(None)
}

pub(crate) fn parse_word_toggle(e: &BytesStart, word_prefixes: &[String]) -> Result<bool> {
    let val = get_word_val_attr(e, word_prefixes)?;
    Ok(ST_OnOff::from_str_or_default(val.as_deref()).is_on())
}

/// Write a toggle element.
pub(crate) fn write_toggle<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    value: bool,
) -> Result<()> {
    if value {
        writer.write_event(Event::Empty(BytesStart::new(tag)))?;
    } else {
        let mut e = BytesStart::new(tag);
        e.push_attribute(("w:val", "false"));
        writer.write_event(Event::Empty(e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ST_Jc;

    fn try_parse_ppr(xml: &str) -> Result<CT_PPr> {
        let full = format!("<w:pPr>{xml}</w:pPr>");
        let mut reader = Reader::from_str(&full);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"pPr") => break,
                _ => {}
            }
            buf.clear();
        }
        CT_PPr::from_xml(&mut reader)
    }

    fn parse_ppr(xml: &str) -> CT_PPr {
        try_parse_ppr(xml).unwrap()
    }

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
    fn word_bidi_and_run_rtl_parse_write_parse_without_raw_loss() {
        let ppr = parse_ppr(
            r#"<x:before xmlns:x="urn:producer"/><w:bidi/><x:after xmlns:x="urn:producer"/>"#,
        );
        assert_eq!(ppr.bidi, Some(true));

        let mut paragraph_output = Vec::new();
        ppr.to_xml(&mut Writer::new(&mut paragraph_output)).unwrap();
        let paragraph_output = String::from_utf8(paragraph_output).unwrap();
        assert!(
            paragraph_output.find("x:before").unwrap() < paragraph_output.find("w:bidi").unwrap()
        );
        assert!(
            paragraph_output.find("w:bidi").unwrap() < paragraph_output.find("x:after").unwrap()
        );

        let rpr = parse_rpr(
            r#"<x:before xmlns:x="urn:producer"/><w:rtl/><x:rtl xmlns:x="urn:foreign"/><x:after xmlns:x="urn:producer"/>"#,
        );
        assert_eq!(rpr.rtl, Some(true));
        let mut run_output = Vec::new();
        rpr.to_xml(&mut Writer::new(&mut run_output)).unwrap();
        let run_output = String::from_utf8(run_output).unwrap();
        assert!(run_output.contains("<w:rtl/>"));
        assert!(run_output.contains("<x:rtl xmlns:x=\"urn:foreign\"/>"));

        let reparsed_ppr = parse_ppr(
            paragraph_output
                .strip_prefix("<w:pPr>")
                .unwrap()
                .strip_suffix("</w:pPr>")
                .unwrap(),
        );
        let reparsed_rpr = parse_rpr(
            run_output
                .strip_prefix("<w:rPr>")
                .unwrap()
                .strip_suffix("</w:rPr>")
                .unwrap(),
        );
        assert_eq!(reparsed_ppr.bidi, Some(true));
        assert_eq!(reparsed_rpr.rtl, Some(true));
    }

    #[test]
    fn word_direction_toggles_preserve_unsupported_attributes() {
        let ppr = parse_ppr(r#"<w:bidi xmlns:x="urn:producer" w:val="0" x:flag="paragraph"/>"#);
        let rpr = parse_rpr(r#"<w:rtl xmlns:x="urn:producer" w:val="1" x:flag="run"></w:rtl>"#);
        assert_eq!(ppr.bidi, Some(false));
        assert_eq!(rpr.rtl, Some(true));

        let mut paragraph_output = Vec::new();
        ppr.to_xml(&mut Writer::new(&mut paragraph_output)).unwrap();
        let paragraph_output = String::from_utf8(paragraph_output).unwrap();
        assert!(paragraph_output.contains(r#"xmlns:x="urn:producer""#));
        assert!(paragraph_output.contains(r#"x:flag="paragraph""#));

        let mut run_output = Vec::new();
        rpr.to_xml(&mut Writer::new(&mut run_output)).unwrap();
        let run_output = String::from_utf8(run_output).unwrap();
        assert!(run_output.contains(r#"xmlns:x="urn:producer""#));
        assert!(run_output.contains(r#"x:flag="run""#));
    }

    #[test]
    fn malformed_direction_duplicates_keep_their_relative_occurrence() {
        let ppr =
            parse_ppr(r#"<w:bidi/><w:bidi><w:producerExtension w:val="paragraph"/></w:bidi>"#);
        let rpr = parse_rpr(r#"<w:rtl/><w:rtl><w:producerExtension w:val="run"/></w:rtl>"#);

        let mut paragraph_output = Vec::new();
        ppr.to_xml(&mut Writer::new(&mut paragraph_output)).unwrap();
        let paragraph_output = String::from_utf8(paragraph_output).unwrap();
        assert!(
            paragraph_output.find("<w:bidi/>").unwrap()
                < paragraph_output.find("producerExtension").unwrap()
        );

        let mut run_output = Vec::new();
        rpr.to_xml(&mut Writer::new(&mut run_output)).unwrap();
        let run_output = String::from_utf8(run_output).unwrap();
        assert!(
            run_output.find("<w:rtl/>").unwrap() < run_output.find("producerExtension").unwrap()
        );

        let reparsed_ppr = parse_ppr(
            paragraph_output
                .strip_prefix("<w:pPr>")
                .unwrap()
                .strip_suffix("</w:pPr>")
                .unwrap(),
        );
        let reparsed_rpr = parse_rpr(
            run_output
                .strip_prefix("<w:rPr>")
                .unwrap()
                .strip_suffix("</w:rPr>")
                .unwrap(),
        );
        let mut repeated_ppr = Vec::new();
        reparsed_ppr
            .to_xml(&mut Writer::new(&mut repeated_ppr))
            .unwrap();
        let mut repeated_rpr = Vec::new();
        reparsed_rpr
            .to_xml(&mut Writer::new(&mut repeated_rpr))
            .unwrap();
        assert_eq!(repeated_ppr, paragraph_output.as_bytes());
        assert_eq!(repeated_rpr, run_output.as_bytes());
    }

    #[test]
    fn interleaved_direction_duplicates_keep_their_relative_occurrence() {
        let ppr = parse_ppr(
            r#"<w:bidi xmlns:x="urn:first" x:first="paragraph"/><w:bidi><w:producerExtension w:val="paragraph"/></w:bidi><w:bidi xmlns:y="urn:last" y:last="paragraph"/>"#,
        );
        let rpr = parse_rpr(
            r#"<w:rtl xmlns:x="urn:first" x:first="run"/><w:rtl><w:producerExtension w:val="run"/></w:rtl><w:rtl xmlns:y="urn:last" y:last="run"/>"#,
        );

        let mut paragraph_output = Vec::new();
        ppr.to_xml(&mut Writer::new(&mut paragraph_output)).unwrap();
        let paragraph_output = String::from_utf8(paragraph_output).unwrap();
        assert!(
            paragraph_output.find("x:first").unwrap()
                < paragraph_output.find("producerExtension").unwrap(),
            "{paragraph_output}"
        );
        assert!(
            paragraph_output.find("producerExtension").unwrap()
                < paragraph_output.find("y:last").unwrap(),
            "{paragraph_output}"
        );

        let mut run_output = Vec::new();
        rpr.to_xml(&mut Writer::new(&mut run_output)).unwrap();
        let run_output = String::from_utf8(run_output).unwrap();
        assert!(
            run_output.find("x:first").unwrap() < run_output.find("producerExtension").unwrap(),
            "{run_output}"
        );
        assert!(
            run_output.find("producerExtension").unwrap() < run_output.find("y:last").unwrap(),
            "{run_output}"
        );

        let reparsed_ppr = parse_ppr(
            paragraph_output
                .strip_prefix("<w:pPr>")
                .unwrap()
                .strip_suffix("</w:pPr>")
                .unwrap(),
        );
        let reparsed_rpr = parse_rpr(
            run_output
                .strip_prefix("<w:rPr>")
                .unwrap()
                .strip_suffix("</w:rPr>")
                .unwrap(),
        );
        let mut repeated_ppr = Vec::new();
        reparsed_ppr
            .to_xml(&mut Writer::new(&mut repeated_ppr))
            .unwrap();
        let mut repeated_rpr = Vec::new();
        reparsed_rpr
            .to_xml(&mut Writer::new(&mut repeated_rpr))
            .unwrap();
        assert_eq!(repeated_ppr, paragraph_output.as_bytes());
        assert_eq!(repeated_rpr, run_output.as_bytes());
    }

    #[test]
    fn canonical_final_direction_toggle_stays_after_an_interleaved_malformed_occurrence() {
        let ppr = parse_ppr(
            r#"<w:bidi xmlns:x="urn:first" x:first="paragraph"/><w:bidi><w:producerExtension w:val="paragraph"/></w:bidi><w:bidi/>"#,
        );
        let rpr = parse_rpr(
            r#"<w:rtl xmlns:x="urn:first" x:first="run"/><w:rtl><w:producerExtension w:val="run"/></w:rtl><w:rtl/>"#,
        );

        let mut paragraph_output = Vec::new();
        ppr.to_xml(&mut Writer::new(&mut paragraph_output)).unwrap();
        let paragraph_output = String::from_utf8(paragraph_output).unwrap();
        assert_eq!(paragraph_output.matches("<w:bidi").count(), 3);
        assert!(
            paragraph_output.find("x:first").unwrap()
                < paragraph_output.find("producerExtension").unwrap(),
            "{paragraph_output}"
        );
        assert!(
            paragraph_output.find("producerExtension").unwrap()
                < paragraph_output.rfind("<w:bidi").unwrap(),
            "{paragraph_output}"
        );

        let mut run_output = Vec::new();
        rpr.to_xml(&mut Writer::new(&mut run_output)).unwrap();
        let run_output = String::from_utf8(run_output).unwrap();
        assert_eq!(run_output.matches("<w:rtl").count(), 3);
        assert!(
            run_output.find("x:first").unwrap() < run_output.find("producerExtension").unwrap(),
            "{run_output}"
        );
        assert!(
            run_output.find("producerExtension").unwrap() < run_output.rfind("<w:rtl").unwrap(),
            "{run_output}"
        );

        let reparsed_ppr = parse_ppr(
            paragraph_output
                .strip_prefix("<w:pPr>")
                .unwrap()
                .strip_suffix("</w:pPr>")
                .unwrap(),
        );
        let reparsed_rpr = parse_rpr(
            run_output
                .strip_prefix("<w:rPr>")
                .unwrap()
                .strip_suffix("</w:rPr>")
                .unwrap(),
        );
        let mut repeated_ppr = Vec::new();
        reparsed_ppr
            .to_xml(&mut Writer::new(&mut repeated_ppr))
            .unwrap();
        let mut repeated_rpr = Vec::new();
        reparsed_rpr
            .to_xml(&mut Writer::new(&mut repeated_rpr))
            .unwrap();
        assert_eq!(repeated_ppr, paragraph_output.as_bytes());
        assert_eq!(repeated_rpr, run_output.as_bytes());
    }

    #[test]
    fn duplicate_valid_direction_toggles_keep_each_occurrences_attributes_and_order() {
        let ppr = parse_ppr(
            r#"<w:bidi xmlns:x="urn:first" w:val="0" x:first="paragraph"/><w:bidi xmlns:y="urn:second" w:val="1" y:second="paragraph"/>"#,
        );
        let rpr = parse_rpr(
            r#"<w:rtl xmlns:x="urn:first" w:val="0" x:first="run"/><w:rtl xmlns:y="urn:second" w:val="1" y:second="run"/>"#,
        );

        let mut paragraph_output = Vec::new();
        ppr.to_xml(&mut Writer::new(&mut paragraph_output)).unwrap();
        let paragraph_output = String::from_utf8(paragraph_output).unwrap();
        assert_eq!(paragraph_output.matches("<w:bidi").count(), 2);
        assert!(
            paragraph_output.find("x:first").unwrap() < paragraph_output.find("y:second").unwrap(),
            "{paragraph_output}"
        );
        assert!(paragraph_output.contains(r#"x:first="paragraph""#));
        assert!(paragraph_output.contains(r#"y:second="paragraph""#));

        let mut run_output = Vec::new();
        rpr.to_xml(&mut Writer::new(&mut run_output)).unwrap();
        let run_output = String::from_utf8(run_output).unwrap();
        assert_eq!(run_output.matches("<w:rtl").count(), 2);
        assert!(
            run_output.find("x:first").unwrap() < run_output.find("y:second").unwrap(),
            "{run_output}"
        );
        assert!(run_output.contains(r#"x:first="run""#));
        assert!(run_output.contains(r#"y:second="run""#));

        let reparsed_ppr = parse_ppr(
            paragraph_output
                .strip_prefix("<w:pPr>")
                .unwrap()
                .strip_suffix("</w:pPr>")
                .unwrap(),
        );
        let reparsed_rpr = parse_rpr(
            run_output
                .strip_prefix("<w:rPr>")
                .unwrap()
                .strip_suffix("</w:rPr>")
                .unwrap(),
        );
        assert_eq!(reparsed_ppr.bidi, Some(true));
        assert_eq!(reparsed_rpr.rtl, Some(true));

        let mut repeated_ppr = Vec::new();
        reparsed_ppr
            .to_xml(&mut Writer::new(&mut repeated_ppr))
            .unwrap();
        let mut repeated_rpr = Vec::new();
        reparsed_rpr
            .to_xml(&mut Writer::new(&mut repeated_rpr))
            .unwrap();
        assert_eq!(repeated_ppr, paragraph_output.as_bytes());
        assert_eq!(repeated_rpr, run_output.as_bytes());

        let mut cleared_ppr = reparsed_ppr;
        cleared_ppr.bidi = None;
        cleared_ppr.jc = Some(ST_Jc::Left);
        let mut cleared_ppr_output = Vec::new();
        cleared_ppr
            .to_xml(&mut Writer::new(&mut cleared_ppr_output))
            .unwrap();
        let cleared_ppr_output = String::from_utf8(cleared_ppr_output).unwrap();
        assert!(
            !cleared_ppr_output.contains("<w:bidi"),
            "{cleared_ppr_output}"
        );
        assert_eq!(
            parse_ppr(
                cleared_ppr_output
                    .strip_prefix("<w:pPr>")
                    .unwrap()
                    .strip_suffix("</w:pPr>")
                    .unwrap(),
            )
            .bidi,
            None
        );

        let mut cleared_rpr = reparsed_rpr;
        cleared_rpr.rtl = None;
        cleared_rpr.bold = Some(true);
        let mut cleared_rpr_output = Vec::new();
        cleared_rpr
            .to_xml(&mut Writer::new(&mut cleared_rpr_output))
            .unwrap();
        let cleared_rpr_output = String::from_utf8(cleared_rpr_output).unwrap();
        assert!(
            !cleared_rpr_output.contains("<w:rtl"),
            "{cleared_rpr_output}"
        );
        assert_eq!(
            parse_rpr(
                cleared_rpr_output
                    .strip_prefix("<w:rPr>")
                    .unwrap()
                    .strip_suffix("</w:rPr>")
                    .unwrap(),
            )
            .rtl,
            None
        );
    }
}
