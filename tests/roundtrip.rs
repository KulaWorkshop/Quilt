use std::fs::{self, File};
use std::io::BufReader;

use quilt::archive::encoder::ArchiveEncoder;
use quilt::archive::{Archive, ArchiveType};

fn names(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("LEVEL {i}")).collect()
}

fn roundtrip(ty: ArchiveType, names: &[String], no_filenames: bool) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in");
    let output = dir.path().join("out");
    fs::create_dir(&input).unwrap();
    fs::create_dir(&output).unwrap();

    // create archive
    let mut encoder = ArchiveEncoder::new(ty);
    let mut expected = Vec::new();
    for (i, name) in names.iter().enumerate() {
        let data = format!("{name} contents {i}\n")
            .repeat(100 + i)
            .into_bytes();
        let path = input.join(name);
        fs::write(&path, &data).unwrap();
        encoder.add(&path).unwrap();
        expected.push(data);
    }

    let archive_path = dir.path().join("test.pak");
    encoder.pack(&archive_path, no_filenames).unwrap();

    // read it back
    let reader = BufReader::new(File::open(&archive_path).unwrap());
    let mut archive = Archive::open(reader).unwrap();
    let entries: Vec<_> = archive.entries().unwrap().collect();
    assert_eq!(entries.len(), names.len(), "entry count");

    for (i, entry) in entries.iter().enumerate() {
        if ty == ArchiveType::Pak {
            let want = if no_filenames {
                "FILE"
            } else {
                names[i].as_str()
            };
            assert_eq!(entry.name.as_deref(), Some(want), "name of entry {i}");
        }
        let out = output.join(i.to_string());
        entry.unpack(&out).unwrap();
        assert_eq!(
            fs::read(&out).unwrap(),
            expected[i],
            "contents of entry {i}"
        );
    }
}

#[test]
fn pak_one_file() {
    roundtrip(ArchiveType::Pak, &names(1), false);
}

#[test]
fn pak_two_files() {
    roundtrip(ArchiveType::Pak, &names(2), false);
}

#[test]
fn pak_twenty_files() {
    roundtrip(ArchiveType::Pak, &names(20), false);
}

#[test]
fn pak_no_filenames() {
    roundtrip(ArchiveType::Pak, &names(3), true);
}

#[test]
fn kub_roundtrip() {
    roundtrip(ArchiveType::Kub, &names(5), false);
}
