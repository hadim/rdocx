//! Ruby phonetic guides (`w:ruby`).
//!
//! `w:ruby` is a sibling of `w:r` inside `w:p`, not a run child. It carries a
//! phonetic line in `w:rt` and a base line in `w:rubyBase`, and both hold
//! ordinary runs, so [`CT_R`] is reused rather than a second run grammar
//! being introduced.
//!
//! The base runs live in `CT_P::runs` and the annotation records the
//! half-open span they occupy, exactly as `w:hyperlink` already does. That is
//! what makes text extraction, search, redaction and layout see the base text
//! as ordinary paragraph content, while the phonetic line, held here in
//! [`CT_Ruby::ruby_text`], stays an annotation that no text extraction
//! returns.
//!
//! The typed model admits only the shapes it can write back exactly. A
//! `w:ruby` carrying an attribute, a `w:rt` or `w:rubyBase` child that is not
//! a run, non-whitespace character data between children, or an empty base
//! line is left as preserved raw XML by the paragraph parser instead.
//! Whitespace between children is layout indentation that this crate's own
//! writer produces, so it is read and dropped rather than rejected.
//!
//! The one exception is an unmodelled `w:rubyPr` element child, which is
//! retained and written after the modelled ones, because `w:rubyPr` is a
//! closed sequence and such a child is already off-schema wherever it sat.

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::Result;
use crate::properties::{
    get_word_val_attr, is_word_element, parse_word_toggle, word_prefixes_at, write_toggle,
};
use crate::raw_xml::{capture_element, capture_empty_element};
use crate::text::{CT_R, parse_run_raw};
use crate::units::HalfPoint;

/// `ST_RubyAlign` — the horizontal distribution of the phonetic line over the
/// base line, on `w:rubyAlign/@w:val`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ST_RubyAlign {
    Center,
    DistributeLetter,
    DistributeSpace,
    Left,
    RightVertical,
    Right,
    /// A token outside the ECMA-376 inventory, retained verbatim.
    Other(String),
}

impl ST_RubyAlign {
    pub fn from_str(value: &str) -> Self {
        match value {
            "center" => ST_RubyAlign::Center,
            "distributeLetter" => ST_RubyAlign::DistributeLetter,
            "distributeSpace" => ST_RubyAlign::DistributeSpace,
            "left" => ST_RubyAlign::Left,
            "rightVertical" => ST_RubyAlign::RightVertical,
            "right" => ST_RubyAlign::Right,
            other => ST_RubyAlign::Other(other.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ST_RubyAlign::Center => "center",
            ST_RubyAlign::DistributeLetter => "distributeLetter",
            ST_RubyAlign::DistributeSpace => "distributeSpace",
            ST_RubyAlign::Left => "left",
            ST_RubyAlign::RightVertical => "rightVertical",
            ST_RubyAlign::Right => "right",
            ST_RubyAlign::Other(value) => value.as_str(),
        }
    }
}

/// `CT_RubyPr` — the geometry of one phonetic guide.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct CT_RubyPr {
    /// Distribution of the phonetic line over the base (`w:rubyAlign`).
    pub align: Option<ST_RubyAlign>,
    /// Phonetic line size in half-points (`w:hps`).
    pub hps: Option<HalfPoint>,
    /// Phonetic baseline raise above the base baseline in half-points
    /// (`w:hpsRaise`).
    pub hps_raise: Option<HalfPoint>,
    /// Base line size in half-points (`w:hpsBaseText`).
    pub hps_base_text: Option<HalfPoint>,
    /// Language identifier for the phonetic line (`w:lid/@w:val`).
    pub language: Option<String>,
    /// Producer-set dirty flag (`w:dirty`).
    pub dirty: Option<bool>,
    /// Unmodelled `w:rubyPr` children, written back after the modelled ones.
    #[doc(hidden)]
    pub raw_xml: Vec<Vec<u8>>,
}

impl CT_RubyPr {
    /// Parse `w:rubyPr` from its captured source.
    ///
    /// `None` reports non-whitespace character data this model cannot write
    /// back exactly, so the whole annotation stays raw.
    fn from_raw(raw: &[u8], word_prefixes: &[String]) -> Result<Option<Self>> {
        let mut reader = Reader::from_reader(raw);
        reader.config_mut().trim_text(false);
        let mut properties = CT_RubyPr::default();
        let mut word_prefixes = word_prefixes.to_vec();
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        loop {
            let mut child = None;
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    if depth == 0 {
                        word_prefixes = word_prefixes_at(&element, &word_prefixes)?;
                        depth = 1;
                    } else {
                        let prefixes = word_prefixes_at(&element, &word_prefixes)?;
                        let raw = capture_element(&mut reader, &element)?;
                        child = Some((element.into_owned(), prefixes, Some(raw)));
                    }
                }
                Event::Empty(element) => {
                    if depth == 0 {
                        return Ok(Some(properties));
                    }
                    let prefixes = word_prefixes_at(&element, &word_prefixes)?;
                    child = Some((element.into_owned(), prefixes, None));
                }
                Event::Text(text) if depth == 1 && !text.iter().all(u8::is_ascii_whitespace) => {
                    return Ok(None);
                }
                Event::CData(_) if depth == 1 => return Ok(None),
                Event::End(_) => depth = depth.saturating_sub(1),
                Event::Eof => break,
                _ => {}
            }
            if let Some((element, prefixes, raw)) = child
                && !properties.read_child(&element, &prefixes)?
            {
                properties.raw_xml.push(match raw {
                    Some(raw) => raw,
                    None => capture_empty_element(&element)?,
                });
            }
            buffer.clear();
        }
        Ok(Some(properties))
    }

    /// Record one modelled `w:rubyPr` child, reporting whether it was modelled.
    ///
    /// Every child but `w:dirty` carries a required `w:val`. One without it
    /// names no value, so it is reported unmodelled and kept as raw rather
    /// than normalised away, which is the policy `w:kern` already follows in
    /// the run property module.
    fn read_child(&mut self, element: &BytesStart<'_>, prefixes: &[String]) -> Result<bool> {
        let name = element.name();
        if is_word_element(name.as_ref(), b"rubyAlign", prefixes) {
            let Some(value) = get_word_val_attr(element, prefixes)? else {
                return Ok(false);
            };
            self.align = Some(ST_RubyAlign::from_str(&value));
        } else if is_word_element(name.as_ref(), b"hps", prefixes) {
            let Some(value) = half_point_value(element, prefixes)? else {
                return Ok(false);
            };
            self.hps = Some(value);
        } else if is_word_element(name.as_ref(), b"hpsRaise", prefixes) {
            let Some(value) = half_point_value(element, prefixes)? else {
                return Ok(false);
            };
            self.hps_raise = Some(value);
        } else if is_word_element(name.as_ref(), b"hpsBaseText", prefixes) {
            let Some(value) = half_point_value(element, prefixes)? else {
                return Ok(false);
            };
            self.hps_base_text = Some(value);
        } else if is_word_element(name.as_ref(), b"lid", prefixes) {
            let Some(value) = get_word_val_attr(element, prefixes)? else {
                return Ok(false);
            };
            self.language = Some(value);
        } else if is_word_element(name.as_ref(), b"dirty", prefixes) {
            self.dirty = Some(parse_word_toggle(element, prefixes)?);
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    /// Write `w:rubyPr` in `xsd:sequence` order with the fixed `w:` prefix.
    fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:rubyPr")))?;
        if let Some(ref align) = self.align {
            let mut element = BytesStart::new("w:rubyAlign");
            element.push_attribute(("w:val", align.as_str()));
            writer.write_event(Event::Empty(element))?;
        }
        let mut buffer = itoa::Buffer::new();
        for (tag, value) in [
            ("w:hps", self.hps),
            ("w:hpsRaise", self.hps_raise),
            ("w:hpsBaseText", self.hps_base_text),
        ] {
            if let Some(value) = value {
                let mut element = BytesStart::new(tag);
                element.push_attribute(("w:val", buffer.format(value.0)));
                writer.write_event(Event::Empty(element))?;
            }
        }
        if let Some(ref language) = self.language {
            let mut element = BytesStart::new("w:lid");
            element.push_attribute(("w:val", language.as_str()));
            writer.write_event(Event::Empty(element))?;
        }
        if let Some(dirty) = self.dirty {
            write_toggle(writer, "w:dirty", dirty)?;
        }
        for raw in &self.raw_xml {
            writer.get_mut().write_all(raw)?;
        }
        writer.write_event(Event::End(BytesEnd::new("w:rubyPr")))?;
        Ok(())
    }
}

/// Report whether the element carries an attribute other than a namespace
/// declaration.
///
/// `w:ruby`, `w:rt` and `w:rubyBase` have no attributes of their own, so one
/// that is not `xmlns` is content this model cannot write back exactly.
fn has_foreign_attributes(element: &BytesStart<'_>) -> Result<bool> {
    for attribute in element.attributes() {
        let key = attribute?.key;
        let key = key.as_ref();
        if key != b"xmlns" && !key.starts_with(b"xmlns:") {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Read a `w:val` half-point attribute, leaving the slot empty when the
/// required attribute is absent or is not a number.
fn half_point_value(element: &BytesStart<'_>, prefixes: &[String]) -> Result<Option<HalfPoint>> {
    Ok(get_word_val_attr(element, prefixes)?
        .and_then(|value| value.parse().ok())
        .map(HalfPoint))
}

/// `CT_Ruby` — one phonetic guide over a span of paragraph runs.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(non_camel_case_types)]
pub struct CT_Ruby {
    /// The phonetic geometry (`w:rubyPr`).
    pub properties: Option<CT_RubyPr>,
    /// The phonetic runs (`w:rt`). No text extraction returns these.
    pub ruby_text: Vec<CT_R>,
    /// Index of the first base run in the owning paragraph (inclusive).
    pub base_start: usize,
    /// Index one past the last base run in the owning paragraph (exclusive).
    pub base_end: usize,
    /// Unmodelled `w:ruby` children, written back after `w:rubyBase`.
    #[doc(hidden)]
    pub raw_xml: Vec<Vec<u8>>,
}

impl CT_Ruby {
    /// Parse `w:ruby` from its captured source.
    ///
    /// Returns the annotation, with `base_start` and `base_end` left at zero
    /// for the owning paragraph to fill, together with the base runs the
    /// paragraph splices into its own run list. `None` means the source
    /// carries something this model cannot write back exactly, so the caller
    /// keeps the raw subtree instead.
    pub(crate) fn from_raw(
        raw: &[u8],
        word_prefixes: &[String],
    ) -> Result<Option<(Self, Vec<CT_R>)>> {
        let mut reader = Reader::from_reader(raw);
        reader.config_mut().trim_text(false);
        let mut ruby = CT_Ruby::default();
        let mut base_runs = Vec::new();
        let mut seen_base = false;
        let mut word_prefixes = word_prefixes.to_vec();
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    if depth == 0 {
                        if has_foreign_attributes(&element)? {
                            return Ok(None);
                        }
                        word_prefixes = word_prefixes_at(&element, &word_prefixes)?;
                        depth = 1;
                    } else {
                        let prefixes = word_prefixes_at(&element, &word_prefixes)?;
                        let name = element.name().as_ref().to_vec();
                        let child = capture_element(&mut reader, &element)?;
                        if is_word_element(&name, b"rubyPr", &prefixes) {
                            let Some(properties) = CT_RubyPr::from_raw(&child, &prefixes)? else {
                                return Ok(None);
                            };
                            ruby.properties = Some(properties);
                        } else if is_word_element(&name, b"rt", &prefixes) {
                            let Some(runs) = parse_ruby_content(&child, &prefixes)? else {
                                return Ok(None);
                            };
                            ruby.ruby_text = runs;
                        } else if is_word_element(&name, b"rubyBase", &prefixes) {
                            let Some(runs) = parse_ruby_content(&child, &prefixes)? else {
                                return Ok(None);
                            };
                            seen_base = true;
                            base_runs = runs;
                        } else {
                            ruby.raw_xml.push(child);
                        }
                    }
                }
                Event::Empty(element) => {
                    if depth == 0 {
                        // `<w:ruby/>` carries no base line to annotate.
                        return Ok(None);
                    }
                    let prefixes = word_prefixes_at(&element, &word_prefixes)?;
                    let name = element.name();
                    if is_word_element(name.as_ref(), b"rubyPr", &prefixes) {
                        ruby.properties = Some(CT_RubyPr::default());
                    } else if is_word_element(name.as_ref(), b"rt", &prefixes)
                        || is_word_element(name.as_ref(), b"rubyBase", &prefixes)
                    {
                        // An empty base line has nothing to annotate, and an
                        // empty phonetic line would not survive the write
                        // back as an empty element.
                        return Ok(None);
                    } else {
                        ruby.raw_xml.push(capture_empty_element(&element)?);
                    }
                }
                Event::Text(text) if depth == 1 && !text.iter().all(u8::is_ascii_whitespace) => {
                    return Ok(None);
                }
                Event::CData(_) if depth == 1 => return Ok(None),
                Event::End(_) => depth = depth.saturating_sub(1),
                Event::Eof => break,
                _ => {}
            }
            buffer.clear();
        }
        if !seen_base || base_runs.is_empty() {
            return Ok(None);
        }
        Ok(Some((ruby, base_runs)))
    }

    /// The half-open range of owning-paragraph runs this annotation covers.
    pub fn base_range(&self) -> std::ops::Range<usize> {
        self.base_start..self.base_end
    }

    /// Write `w:ruby` up to and including the `w:rubyBase` start tag.
    ///
    /// The base runs are paragraph runs, so the owning paragraph writes them
    /// between this call and [`Self::write_end`].
    pub(crate) fn write_start<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:ruby")))?;
        if let Some(ref properties) = self.properties {
            properties.to_xml(writer)?;
        }
        writer.write_event(Event::Start(BytesStart::new("w:rt")))?;
        for run in &self.ruby_text {
            run.to_xml(writer)?;
        }
        writer.write_event(Event::End(BytesEnd::new("w:rt")))?;
        writer.write_event(Event::Start(BytesStart::new("w:rubyBase")))?;
        Ok(())
    }

    /// Close `w:rubyBase` and `w:ruby` after the base runs have been written.
    pub(crate) fn write_end<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::End(BytesEnd::new("w:rubyBase")))?;
        for raw in &self.raw_xml {
            writer.get_mut().write_all(raw)?;
        }
        writer.write_event(Event::End(BytesEnd::new("w:ruby")))?;
        Ok(())
    }
}

/// Parse the runs of a `w:rt` or `w:rubyBase`, or report that the element
/// carries something the typed model cannot write back exactly.
fn parse_ruby_content(raw: &[u8], word_prefixes: &[String]) -> Result<Option<Vec<CT_R>>> {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(false);
    let mut runs = Vec::new();
    let mut word_prefixes = word_prefixes.to_vec();
    let mut buffer = Vec::new();
    let mut depth = 0usize;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                if depth == 0 {
                    if has_foreign_attributes(&element)? {
                        return Ok(None);
                    }
                    word_prefixes = word_prefixes_at(&element, &word_prefixes)?;
                    depth = 1;
                } else {
                    let prefixes = word_prefixes_at(&element, &word_prefixes)?;
                    if !is_word_element(element.name().as_ref(), b"r", &prefixes) {
                        return Ok(None);
                    }
                    let child = capture_element(&mut reader, &element)?;
                    runs.push(parse_run_raw(&child, &prefixes)?);
                }
            }
            Event::Empty(element) => {
                if depth == 0 {
                    return Ok(None);
                }
                let prefixes = word_prefixes_at(&element, &word_prefixes)?;
                if !is_word_element(element.name().as_ref(), b"r", &prefixes) {
                    return Ok(None);
                }
                runs.push(CT_R::from_empty_root(&element, &prefixes)?);
            }
            Event::Text(text) if depth == 1 && !text.iter().all(u8::is_ascii_whitespace) => {
                return Ok(None);
            }
            Event::CData(_) if depth == 1 => return Ok(None),
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(Some(runs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::W_NS;

    fn parse(inner: &str) -> Option<(CT_Ruby, Vec<CT_R>)> {
        let source = format!(r#"<w:ruby xmlns:w="{W_NS}">{inner}</w:ruby>"#);
        CT_Ruby::from_raw(source.as_bytes(), &["w".to_owned()]).unwrap()
    }

    #[test]
    fn a_full_ruby_parses_its_properties_its_phonetic_runs_and_its_base_runs() {
        let (ruby, base) = parse(concat!(
            r#"<w:rubyPr><w:rubyAlign w:val="distributeSpace"/><w:hps w:val="10"/>"#,
            r#"<w:hpsRaise w:val="22"/><w:hpsBaseText w:val="21"/><w:lid w:val="ja-JP"/>"#,
            r#"<w:dirty w:val="false"/></w:rubyPr>"#,
            r#"<w:rt><w:r><w:t>かんじ</w:t></w:r></w:rt>"#,
            r#"<w:rubyBase><w:r><w:t>漢字</w:t></w:r></w:rubyBase>"#,
        ))
        .expect("a well formed ruby is modelled");
        let properties = ruby.properties.expect("w:rubyPr is modelled");
        assert_eq!(properties.align, Some(ST_RubyAlign::DistributeSpace));
        assert_eq!(properties.hps, Some(HalfPoint(10)));
        assert_eq!(properties.hps_raise, Some(HalfPoint(22)));
        assert_eq!(properties.hps_base_text, Some(HalfPoint(21)));
        assert_eq!(properties.language.as_deref(), Some("ja-JP"));
        assert_eq!(properties.dirty, Some(false));
        assert_eq!(ruby.ruby_text.len(), 1);
        assert_eq!(ruby.ruby_text[0].text(), "かんじ");
        assert_eq!(base.len(), 1);
        assert_eq!(base[0].text(), "漢字");
    }

    #[test]
    fn a_prefix_aliased_ruby_reads_through_its_own_alias() {
        let source = format!(
            concat!(
                r#"<q:ruby xmlns:q="{ns}"><q:rt><q:r><q:t>よみ</q:t></q:r></q:rt>"#,
                r#"<q:rubyBase><q:r><q:t>読</q:t></q:r></q:rubyBase></q:ruby>"#,
            ),
            ns = W_NS
        );
        let (ruby, base) = CT_Ruby::from_raw(source.as_bytes(), &[])
            .unwrap()
            .expect("an aliased ruby is modelled");
        assert_eq!(ruby.ruby_text[0].text(), "よみ");
        assert_eq!(base[0].text(), "読");
    }

    #[test]
    fn a_ruby_without_a_base_line_is_not_modelled() {
        assert!(parse(r#"<w:rt><w:r><w:t>よみ</w:t></w:r></w:rt>"#).is_none());
        assert!(parse(r#"<w:rt><w:r><w:t>よみ</w:t></w:r></w:rt><w:rubyBase/>"#).is_none());
    }

    #[test]
    fn a_ruby_content_child_that_is_not_a_run_is_not_modelled() {
        assert!(
            parse(concat!(
                r#"<w:rt><w:r><w:t>よみ</w:t></w:r></w:rt>"#,
                r#"<w:rubyBase><w:sdt><w:sdtContent/></w:sdt></w:rubyBase>"#,
            ))
            .is_none()
        );
    }

    /// A `w:rubyPr` child without its required `w:val` names no value, so it
    /// is kept verbatim rather than normalised into an empty modelled slot.
    #[test]
    fn a_ruby_property_child_without_its_required_value_stays_raw() {
        let (ruby, _) = parse(concat!(
            r#"<w:rubyPr><w:hps/><w:rubyAlign/></w:rubyPr>"#,
            r#"<w:rt><w:r><w:t>よみ</w:t></w:r></w:rt>"#,
            r#"<w:rubyBase><w:r><w:t>読</w:t></w:r></w:rubyBase>"#,
        ))
        .expect("a valueless property child does not reject the ruby");
        let properties = ruby.properties.as_ref().expect("w:rubyPr is modelled");
        assert_eq!(properties.hps, None);
        assert_eq!(properties.align, None);
        assert_eq!(properties.raw_xml.len(), 2);
    }

    #[test]
    fn unmodelled_ruby_property_children_are_retained_and_written_back() {
        let (ruby, _) = parse(concat!(
            r#"<w:rubyPr><w:hps w:val="10"/><w:producerOnly w:val="1"/></w:rubyPr>"#,
            r#"<w:rt><w:r><w:t>よみ</w:t></w:r></w:rt>"#,
            r#"<w:rubyBase><w:r><w:t>読</w:t></w:r></w:rubyBase>"#,
        ))
        .expect("an unmodelled property child does not reject the ruby");
        let properties = ruby.properties.as_ref().expect("w:rubyPr is modelled");
        assert_eq!(properties.raw_xml.len(), 1);

        let mut output = Vec::new();
        properties.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(
            output.contains(r#"<w:producerOnly w:val="1"/>"#),
            "{output}"
        );
        assert!(output.contains(r#"<w:hps w:val="10"/>"#), "{output}");
    }
}
