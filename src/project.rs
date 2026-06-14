use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::core::{BankSlot, Chip, ProjectImage, RemovePolicy, RomBank, RomLayout, CHIPS, LAYOUTS};

const PROJECT_VERSION: u32 = 1;
const METADATA_NAME: &str = "project.json";

/// UI parameters persisted alongside the image so a reopened project restores
/// the full editing state.
#[derive(Clone, Copy)]
pub struct ProjectSettings {
    pub remove_policy: RemovePolicy,
    pub multi_rom_image: bool,
    pub allow_larger_chip: bool,
    pub layout_by_size: bool,
}

/// Result of loading a `.romproj` file.
pub struct LoadedProject {
    pub image: ProjectImage,
    pub settings: ProjectSettings,
}

#[derive(Serialize, Deserialize)]
struct ProjectFile {
    version: u32,
    chip_id: String,
    layout_id: String,
    pad_byte: u8,
    remove_policy: String,
    multi_rom_image: bool,
    allow_larger_chip: bool,
    layout_by: String,
    banks: Vec<BankEntry>,
}

#[derive(Serialize, Deserialize)]
struct BankEntry {
    index: usize,
    binary: String,
    remainder_label: String,
    slots: Vec<SlotEntry>,
}

#[derive(Serialize, Deserialize)]
struct SlotEntry {
    label: String,
    len: usize,
    blank: bool,
    source: Option<String>,
}

fn bank_binary_name(index: usize) -> String {
    format!("banks/bank_{index:03}.bin")
}

/// Suggests a project file name from the current chip.
pub fn suggested_name(image: &ProjectImage) -> String {
    format!("{}_project.romproj", image.chip.id.to_lowercase())
}

pub fn save(path: &Path, image: &ProjectImage, settings: &ProjectSettings) -> Result<(), String> {
    let file =
        File::create(path).map_err(|err| format!("Could not create {}: {err}", path.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut banks = Vec::new();
    for (index, bank) in image.banks.iter().enumerate() {
        let Some(rom) = bank else { continue };

        let binary = bank_binary_name(index);
        zip.start_file(&binary, options)
            .map_err(|err| format!("Could not write bank {index}: {err}"))?;
        zip.write_all(&rom.data)
            .map_err(|err| format!("Could not write bank {index}: {err}"))?;

        banks.push(BankEntry {
            index,
            binary,
            remainder_label: rom.remainder_label.clone(),
            slots: rom
                .slots
                .iter()
                .map(|slot| SlotEntry {
                    label: slot.label.clone(),
                    len: slot.len,
                    blank: slot.blank,
                    source: slot
                        .source_path
                        .as_ref()
                        .map(|path| path.display().to_string()),
                })
                .collect(),
        });
    }

    let project = ProjectFile {
        version: PROJECT_VERSION,
        chip_id: image.chip.id.to_owned(),
        layout_id: image.layout.id.to_owned(),
        pad_byte: image.pad_byte,
        remove_policy: remove_policy_id(settings.remove_policy).to_owned(),
        multi_rom_image: settings.multi_rom_image,
        allow_larger_chip: settings.allow_larger_chip,
        layout_by: if settings.layout_by_size {
            "size".to_owned()
        } else {
            "platform".to_owned()
        },
        banks,
    };

    let json = serde_json::to_string_pretty(&project)
        .map_err(|err| format!("Could not encode project metadata: {err}"))?;
    zip.start_file(METADATA_NAME, options)
        .map_err(|err| format!("Could not write project metadata: {err}"))?;
    zip.write_all(json.as_bytes())
        .map_err(|err| format!("Could not write project metadata: {err}"))?;

    zip.finish()
        .map_err(|err| format!("Could not finalize {}: {err}", path.display()))?;
    Ok(())
}

pub fn load(path: &Path) -> Result<LoadedProject, String> {
    let file =
        File::open(path).map_err(|err| format!("Could not open {}: {err}", path.display()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|err| format!("{} is not a valid project: {err}", path.display()))?;

    let json = {
        let mut entry = archive
            .by_name(METADATA_NAME)
            .map_err(|err| format!("Missing project metadata: {err}"))?;
        let mut text = String::new();
        entry
            .read_to_string(&mut text)
            .map_err(|err| format!("Could not read project metadata: {err}"))?;
        text
    };

    let project: ProjectFile =
        serde_json::from_str(&json).map_err(|err| format!("Invalid project metadata: {err}"))?;

    let chip = chip_by_id(&project.chip_id)
        .ok_or_else(|| format!("Unknown chip '{}'", project.chip_id))?;
    let layout = layout_by_id(&project.layout_id)
        .ok_or_else(|| format!("Unknown layout '{}'", project.layout_id))?;

    let mut image = ProjectImage::new(chip, layout);
    image.pad_byte = project.pad_byte;

    for entry in &project.banks {
        if entry.index >= image.banks.len() {
            continue;
        }

        let mut data = Vec::new();
        archive
            .by_name(&entry.binary)
            .map_err(|err| format!("Missing bank data {}: {err}", entry.binary))?
            .read_to_end(&mut data)
            .map_err(|err| format!("Could not read bank data {}: {err}", entry.binary))?;

        if data.len() != layout.unit_size {
            return Err(format!(
                "Bank {} is {} bytes but {} expects {} bytes",
                entry.index,
                data.len(),
                layout.name,
                layout.unit_size
            ));
        }

        let slots = entry
            .slots
            .iter()
            .map(|slot| BankSlot {
                label: slot.label.clone(),
                source_path: slot.source.clone().map(PathBuf::from),
                len: slot.len,
                blank: slot.blank,
            })
            .collect::<Vec<_>>();

        image.banks[entry.index] = Some(RomBank {
            data,
            slots,
            remainder_label: entry.remainder_label.clone(),
        });
    }

    Ok(LoadedProject {
        image,
        settings: ProjectSettings {
            remove_policy: remove_policy_from_id(&project.remove_policy),
            multi_rom_image: project.multi_rom_image,
            allow_larger_chip: project.allow_larger_chip,
            layout_by_size: project.layout_by != "platform",
        },
    })
}

fn remove_policy_id(policy: RemovePolicy) -> &'static str {
    match policy {
        RemovePolicy::Blank => "blank",
        RemovePolicy::Compact => "compact",
    }
}

fn remove_policy_from_id(id: &str) -> RemovePolicy {
    match id {
        "compact" => RemovePolicy::Compact,
        _ => RemovePolicy::Blank,
    }
}

fn chip_by_id(id: &str) -> Option<Chip> {
    CHIPS.iter().copied().find(|chip| chip.id == id)
}

fn layout_by_id(id: &str) -> Option<RomLayout> {
    LAYOUTS.iter().copied().find(|layout| layout.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_round_trips_through_zip() {
        let chip = CHIPS[1]; // AM29F040B, 512 KiB
        let layout = LAYOUTS[0]; // 16 KiB banks
        let mut image = ProjectImage::new(chip, layout);
        image.pad_byte = 0xEE;

        let mut rom =
            RomBank::from_partial(layout, "Diag".to_owned(), &[0x11; 4000], image.pad_byte)
                .unwrap();
        rom.add_into_free("Patch".to_owned(), None, &[0x22; 600])
            .unwrap();
        rom.remainder_label = "Spare".to_owned();
        image.set_bank(2, rom).unwrap();

        let settings = ProjectSettings {
            remove_policy: RemovePolicy::Compact,
            multi_rom_image: false,
            allow_larger_chip: true,
            layout_by_size: false,
        };

        let mut path = std::env::temp_dir();
        path.push(format!("rom_builder_test_{}.romproj", std::process::id()));

        save(&path, &image, &settings).unwrap();
        let loaded = load(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(loaded.image.chip.id, chip.id);
        assert_eq!(loaded.image.layout.id, layout.id);
        assert_eq!(loaded.image.pad_byte, 0xEE);
        assert_eq!(loaded.image.export_bytes(), image.export_bytes());

        let bank = loaded.image.banks[2].as_ref().unwrap();
        assert_eq!(bank.slots.len(), 2);
        assert_eq!(bank.slots[0].label, "Diag");
        assert_eq!(bank.slots[0].len, 4000);
        assert_eq!(bank.slots[1].label, "Patch");
        assert_eq!(bank.remainder_label, "Spare");

        assert_eq!(loaded.settings.remove_policy, RemovePolicy::Compact);
        assert!(!loaded.settings.multi_rom_image);
        assert!(loaded.settings.allow_larger_chip);
        assert!(!loaded.settings.layout_by_size);
    }
}
