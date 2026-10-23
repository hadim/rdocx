use std::io::{Cursor, Write};
use oxml_opc::OpcPackage;

fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut out);
    let options = zip::write::SimpleFileOptions::default();
    for (name, data) in entries {
        zip.start_file(*name, options).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
    out.into_inner()
}

fn written_and_read(package: &OpcPackage) -> Result<OpcPackage, String> {
    let mut bytes = Cursor::new(Vec::new());
    package.write_to(&mut bytes).map_err(|e| format!("write: {e}"))?;
    bytes.set_position(0);
    OpcPackage::from_reader(bytes).map_err(|e| format!("read: {e}"))
}

fn compare(label: &str, edited: &OpcPackage, read: &OpcPackage) {
    let reread = edited.clone().reread(read).map_err(|e| e.to_string());
    let zip = written_and_read(edited);
    match (&reread, &zip) {
        (Ok(a), Ok(b)) => println!("{label}: both Ok, equal = {}", a == b),
        _ => println!("{label}: reread = {:?}, zip = {:?}", reread.as_ref().map(|_| "Ok"), zip.as_ref().map(|_| "Ok")),
    }
}

fn main() {
    misuse();
    let ct = br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/></Types>"#;
    let rels = br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="a.xml"/></Relationships>"#;
    let root_rels = br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#;

    // 1. A source package with a root ".rels" entry: the reader keeps it as
    //    the relationships of owner "/", which every write refuses.
    let bytes = zip_of(&[("[Content_Types].xml", ct), ("_rels/.rels", rels), (".rels", root_rels), ("a.xml", b"<a/>")]);
    let read = OpcPackage::from_reader(Cursor::new(bytes)).unwrap();
    println!("owners read: {:?}", read.part_rels.keys().collect::<Vec<_>>());
    let mut edited = read.clone();
    edited.set_part("/a.xml", b"<a>edited</a>".to_vec());
    compare("root .rels owner", &edited, &read);

    // 2. A new part whose name ends with a backslash: the ZIP reader takes it
    //    for a directory and drops it.
    let bytes = zip_of(&[("[Content_Types].xml", ct), ("_rels/.rels", rels), ("a.xml", b"<a/>")]);
    let read = OpcPackage::from_reader(Cursor::new(bytes)).unwrap();
    let mut edited = read.clone();
    edited.set_part("/b\\", b"x".to_vec());
    compare("backslash part", &edited, &read);
    let reread = edited.clone().reread(&read).unwrap();
    let zip = written_and_read(&edited).unwrap();
    println!("  reread keeps /b\\: {}, zip keeps it: {}", reread.parts.contains_key("/b\\"), zip.parts.contains_key("/b\\"));
}

#[allow(dead_code)]
fn misuse() {
    // 3. `read` built in memory instead of read from bytes: a shared part
    //    holding a forbidden character is not checked.
    let mut read = OpcPackage::with_main_part("/a.xml", "application/xml");
    read.set_part("/a.xml", b"<a>\x01</a>".to_vec());
    let edited = read.clone();
    compare("in-memory read with forbidden char", &edited, &read);
}
