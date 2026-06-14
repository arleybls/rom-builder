use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RomLayout {
    pub id: &'static str,
    pub name: &'static str,
    pub unit_size: usize,
}

pub const LAYOUTS: [RomLayout; 42] = [
    RomLayout {
        id: "zx-spectrum-16k",
        name: "ZX Spectrum 16K",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "sinclair-ql-48k",
        name: "Sinclair QL 48K",
        unit_size: 48 * 1024,
    },
    RomLayout {
        id: "zx80",
        name: "ZX80 ROM",
        unit_size: 4 * 1024,
    },
    RomLayout {
        id: "zx81",
        name: "ZX81 ROM",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "zx-spectrum-128-rom0",
        name: "ZX Spectrum 128 ROM 0",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "zx-spectrum-128-rom1",
        name: "ZX Spectrum 128 ROM 1",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "zx-spectrum-plus3-rom0",
        name: "ZX Spectrum +3 ROM 0",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "zx-spectrum-plus3-rom1",
        name: "ZX Spectrum +3 ROM 1",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "zx-spectrum-plus3-rom2",
        name: "ZX Spectrum +3 ROM 2",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "zx-spectrum-plus3-rom3",
        name: "ZX Spectrum +3 ROM 3",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "jupiter-ace",
        name: "Jupiter Ace ROM",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "cpc-464-os",
        name: "CPC 464 OS",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "cpc-464-basic",
        name: "CPC 464 BASIC",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "cpc-6128-os",
        name: "CPC 6128 OS",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "cpc-6128-basic",
        name: "CPC 6128 BASIC",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "cpc-6128-amsdos",
        name: "CPC 6128 AMSDOS",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "bbc-micro-b-os",
        name: "BBC Micro B OS",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "bbc-micro-b-basic",
        name: "BBC Micro B BASIC",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "acorn-electron",
        name: "Acorn Electron ROM",
        unit_size: 32 * 1024,
    },
    RomLayout {
        id: "vic-20-basic",
        name: "VIC-20 BASIC",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "vic-20-kernal",
        name: "VIC-20 KERNAL",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "vic-20-char",
        name: "VIC-20 Character",
        unit_size: 4 * 1024,
    },
    RomLayout {
        id: "c64-basic",
        name: "C64 BASIC",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "c64-kernal",
        name: "C64 KERNAL",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "c64-char",
        name: "C64 Character",
        unit_size: 4 * 1024,
    },
    RomLayout {
        id: "cbm-plus4-basic",
        name: "Plus/4 BASIC",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "cbm-plus4-kernal",
        name: "Plus/4 KERNAL",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "atari-xl-xe-os",
        name: "Atari XL/XE OS",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "atari-xl-xe-basic",
        name: "Atari XL/XE BASIC",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "atari-400-800-basic",
        name: "Atari 400/800 BASIC",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "apple-2-plus",
        name: "Apple II+ ROM",
        unit_size: 12 * 1024,
    },
    RomLayout {
        id: "apple-2e",
        name: "Apple IIe ROM",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "msx1",
        name: "MSX1 Main ROM",
        unit_size: 32 * 1024,
    },
    RomLayout {
        id: "oric-atmos",
        name: "Oric Atmos ROM",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "dragon-32",
        name: "Dragon 32 ROM",
        unit_size: 16 * 1024,
    },
    RomLayout {
        id: "trs80-coco-color",
        name: "CoCo Color BASIC",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "trs80-coco-ext",
        name: "CoCo Extended BASIC",
        unit_size: 8 * 1024,
    },
    RomLayout {
        id: "trs80-model-1",
        name: "TRS-80 Model I ROM",
        unit_size: 12 * 1024,
    },
    // Generic ROM sizes (size-only layouts).
    RomLayout {
        id: "rom-64k",
        name: "64 KiB ROM",
        unit_size: 64 * 1024,
    },
    RomLayout {
        id: "rom-128k",
        name: "128 KiB ROM",
        unit_size: 128 * 1024,
    },
    RomLayout {
        id: "rom-256k",
        name: "256 KiB ROM",
        unit_size: 256 * 1024,
    },
    RomLayout {
        id: "rom-512k",
        name: "512 KiB ROM",
        unit_size: 512 * 1024,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Chip {
    pub id: &'static str,
    pub name: &'static str,
    pub capacity: usize,
    pub pad_byte: u8,
}

impl Chip {
    pub fn bank_count(self, layout: RomLayout) -> usize {
        self.capacity / layout.unit_size
    }

    pub fn remainder(self, layout: RomLayout) -> usize {
        self.capacity % layout.unit_size
    }

    pub fn bank_range(self, layout: RomLayout, bank: usize) -> Option<std::ops::Range<usize>> {
        if bank >= self.bank_count(layout) {
            return None;
        }

        let start = bank * layout.unit_size;
        Some(start..start + layout.unit_size)
    }
}

pub const CHIPS: [Chip; 26] = [
    Chip {
        id: "W27C512",
        name: "W27C512 - 64 KiB",
        capacity: 64 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "AM29F040B",
        name: "AM29F040B - 512 KiB",
        capacity: 512 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "SST39SF040",
        name: "SST39SF040 - 512 KiB",
        capacity: 512 * 1024,
        pad_byte: 0xFF,
    },
    // UV EPROMs
    Chip {
        id: "2716",
        name: "2716 - 2 KiB",
        capacity: 2 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "2732",
        name: "2732 - 4 KiB",
        capacity: 4 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "2764",
        name: "2764 - 8 KiB",
        capacity: 8 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "27128",
        name: "27128 - 16 KiB",
        capacity: 16 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "27256",
        name: "27256 - 32 KiB",
        capacity: 32 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "27512",
        name: "27512 - 64 KiB",
        capacity: 64 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "27C010",
        name: "27C010 - 128 KiB",
        capacity: 128 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "27C020",
        name: "27C020 - 256 KiB",
        capacity: 256 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "27C040",
        name: "27C040 - 512 KiB",
        capacity: 512 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "27C080",
        name: "27C080 - 1 MiB",
        capacity: 1024 * 1024,
        pad_byte: 0xFF,
    },
    // Winbond reusable EEPROM-type
    Chip {
        id: "W27C010",
        name: "W27C010 - 128 KiB",
        capacity: 128 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "W27C020",
        name: "W27C020 - 256 KiB",
        capacity: 256 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "W27C040",
        name: "W27C040 - 512 KiB",
        capacity: 512 * 1024,
        pad_byte: 0xFF,
    },
    // Parallel EEPROMs
    Chip {
        id: "28C16",
        name: "28C16 - 2 KiB",
        capacity: 2 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "28C64",
        name: "28C64 - 8 KiB",
        capacity: 8 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "AT28C256",
        name: "AT28C256 - 32 KiB",
        capacity: 32 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "AT28C010",
        name: "AT28C010 - 128 KiB",
        capacity: 128 * 1024,
        pad_byte: 0xFF,
    },
    // NOR flash
    Chip {
        id: "SST39SF010",
        name: "SST39SF010 - 128 KiB",
        capacity: 128 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "SST39SF020",
        name: "SST39SF020 - 256 KiB",
        capacity: 256 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "Am29F010",
        name: "Am29F010 - 128 KiB",
        capacity: 128 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "Am29F032B",
        name: "Am29F032B - 4 MiB",
        capacity: 4 * 1024 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "AT29C256",
        name: "AT29C256 - 32 KiB",
        capacity: 32 * 1024,
        pad_byte: 0xFF,
    },
    Chip {
        id: "AT29C040",
        name: "AT29C040 - 512 KiB",
        capacity: 512 * 1024,
        pad_byte: 0xFF,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemovePolicy {
    Blank,
    Compact,
}

impl RemovePolicy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Blank => "Keep Blank",
            Self::Compact => "Collapse Over Blank",
        }
    }
}

/// One ROM placed inside a bank. Slots are laid contiguously from the bank
/// start; several can share a bank, with blank free space trailing them.
#[derive(Clone, Debug)]
pub struct BankSlot {
    pub label: String,
    pub source_path: Option<PathBuf>,
    pub len: usize,
    /// True for a slot kept as blank padding in place (removed with Keep Blank).
    pub blank: bool,
}

#[derive(Clone, Debug)]
pub struct RomBank {
    /// Full bank image (always the bank size): slot contents packed from the
    /// start, with the trailing free space filled with the pad byte.
    pub data: Vec<u8>,
    /// Occupied ROM slots, laid contiguously from offset 0.
    pub slots: Vec<BankSlot>,
    /// Editable name for the trailing free space.
    pub remainder_label: String,
}

impl RomBank {
    pub fn from_path(layout: RomLayout, path: &Path) -> Result<Self, String> {
        let data =
            fs::read(path).map_err(|err| format!("Could not read {}: {err}", path.display()))?;
        validate_rom_size(layout, &data)?;

        let label = path
            .file_stem()
            .and_then(|name| name.to_str())
            .or_else(|| path.file_name().and_then(|name| name.to_str()))
            .unwrap_or("ROM")
            .to_owned();

        let len = data.len();
        Ok(Self {
            slots: vec![BankSlot {
                label,
                source_path: Some(path.to_path_buf()),
                len,
                blank: false,
            }],
            data,
            remainder_label: default_remainder_label(),
        })
    }

    pub fn from_slice(layout: RomLayout, label: String, data: &[u8]) -> Result<Self, String> {
        validate_rom_size(layout, data)?;

        Ok(Self {
            slots: vec![BankSlot {
                label,
                source_path: None,
                len: data.len(),
                blank: false,
            }],
            data: data.to_vec(),
            remainder_label: default_remainder_label(),
        })
    }

    /// Builds a bank from a ROM smaller than (or equal to) the bank size,
    /// padding the remainder with `pad_byte`.
    pub fn from_partial(
        layout: RomLayout,
        label: String,
        data: &[u8],
        pad_byte: u8,
    ) -> Result<Self, String> {
        if data.is_empty() {
            return Err("ROM is empty".to_owned());
        }
        if data.len() > layout.unit_size {
            return Err(format!(
                "{} ROM must be at most {} bytes, got {} bytes",
                layout.name,
                layout.unit_size,
                data.len()
            ));
        }

        let len = data.len();
        let mut full = data.to_vec();
        full.resize(layout.unit_size, pad_byte);

        Ok(Self {
            slots: vec![BankSlot {
                label,
                source_path: None,
                len,
                blank: false,
            }],
            data: full,
            remainder_label: default_remainder_label(),
        })
    }

    pub fn used(&self) -> usize {
        self.slots.iter().map(|slot| slot.len).sum()
    }

    pub fn free(&self) -> usize {
        self.data.len() - self.used()
    }

    pub fn slot_start(&self, index: usize) -> usize {
        self.slots[..index].iter().map(|slot| slot.len).sum()
    }

    pub fn primary_label(&self) -> &str {
        self.slots
            .first()
            .map(|slot| slot.label.as_str())
            .unwrap_or("")
    }

    /// Places a ROM into the bank's trailing free space, creating a new slot.
    pub fn add_into_free(
        &mut self,
        label: String,
        source_path: Option<PathBuf>,
        content: &[u8],
    ) -> Result<(), String> {
        if content.is_empty() {
            return Err("ROM is empty".to_owned());
        }
        let free = self.free();
        if content.len() > free {
            return Err(format!(
                "ROM is {} bytes but only {} bytes are free in this bank",
                content.len(),
                free
            ));
        }

        let start = self.used();
        self.data[start..start + content.len()].copy_from_slice(content);
        self.slots.push(BankSlot {
            label,
            source_path,
            len: content.len(),
            blank: false,
        });
        Ok(())
    }

    /// Removes a slot according to the policy. `Blank` pads the slot in place and
    /// keeps it (later slots keep their offsets); `Compact` deletes it, shifts
    /// later slots down, and pads the freed tail.
    pub fn remove_slot(&mut self, index: usize, policy: RemovePolicy, pad_byte: u8) {
        if index >= self.slots.len() {
            return;
        }
        let start = self.slot_start(index);
        let len = self.slots[index].len;

        match policy {
            RemovePolicy::Blank => {
                for byte in &mut self.data[start..start + len] {
                    *byte = pad_byte;
                }
                let slot = &mut self.slots[index];
                slot.blank = true;
                slot.source_path = None;
            }
            RemovePolicy::Compact => {
                let used = self.used();
                self.data.copy_within(start + len..used, start);
                for byte in &mut self.data[used - len..used] {
                    *byte = pad_byte;
                }
                self.slots.remove(index);
            }
        }
    }

    /// Folds a bank back down once its content is gone. Returns `true` when every
    /// slot is blank (the bank should become a single unused bank). Otherwise, if
    /// only the primary slot is occupied and every sub-slot is blank, the blank
    /// sub-slots are dropped so their space merges into one trailing free region.
    pub fn collapse_if_unused(&mut self) -> bool {
        if self.slots.iter().all(|slot| slot.blank) {
            return true;
        }
        if self.slots.len() > 1 && self.slots[1..].iter().all(|slot| slot.blank) {
            self.slots.truncate(1);
        }
        false
    }

    /// Rebuilds a bank from a full bank image plus a slot layout (used when
    /// re-importing with a metadata sidecar).
    pub fn from_layout(
        bank_data: &[u8],
        slots: &[SlotSpec],
        remainder_label: String,
    ) -> Result<Self, String> {
        let total: usize = slots.iter().map(|slot| slot.len).sum();
        if total > bank_data.len() {
            return Err(format!(
                "slot layout needs {total} bytes but the bank is only {} bytes",
                bank_data.len()
            ));
        }

        Ok(Self {
            data: bank_data.to_vec(),
            slots: slots
                .iter()
                .map(|slot| BankSlot {
                    label: slot.label.clone(),
                    source_path: None,
                    len: slot.len,
                    blank: slot.blank,
                })
                .collect(),
            remainder_label,
        })
    }

    pub fn checksum16(&self) -> u16 {
        self.data
            .iter()
            .fold(0u16, |sum, byte| sum.wrapping_add(*byte as u16))
    }

    pub fn slot_checksum16(&self, index: usize) -> u16 {
        let start = self.slot_start(index);
        let end = start + self.slots[index].len;
        self.data[start..end]
            .iter()
            .fold(0u16, |sum, byte| sum.wrapping_add(*byte as u16))
    }

    /// Whether any non-blank slot's checksum is in `duplicates`.
    pub fn has_duplicate_slot(&self, duplicates: &HashSet<u16>) -> bool {
        (0..self.slots.len()).any(|index| {
            !self.slots[index].blank && duplicates.contains(&self.slot_checksum16(index))
        })
    }
}

fn default_remainder_label() -> String {
    "Free".to_owned()
}

/// One slot entry parsed from a metadata sidecar.
#[derive(Clone, Debug)]
pub struct SlotSpec {
    pub len: usize,
    pub label: String,
    pub blank: bool,
}

/// A bank's slot configuration parsed from a metadata sidecar.
#[derive(Clone, Debug, Default)]
pub struct BankLayout {
    pub slots: Vec<SlotSpec>,
    pub remainder_label: String,
}

/// Parses the machine-readable `slot|`/`free|` lines emitted by
/// [`ProjectImage::metadata_text`] back into a per-bank slot layout.
pub fn parse_slot_layout(text: &str) -> HashMap<usize, BankLayout> {
    let mut layout: HashMap<usize, BankLayout> = HashMap::new();

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("slot|") {
            // bank|index|len|blank|label  (label may contain '|')
            let parts: Vec<&str> = rest.splitn(5, '|').collect();
            if parts.len() == 5 {
                if let (Ok(bank), Ok(len)) = (parts[0].parse::<usize>(), parts[2].parse::<usize>())
                {
                    layout.entry(bank).or_default().slots.push(SlotSpec {
                        len,
                        blank: parts[3] == "1",
                        label: parts[4].to_owned(),
                    });
                }
            }
        } else if let Some(rest) = line.strip_prefix("free|") {
            // bank|label
            let parts: Vec<&str> = rest.splitn(2, '|').collect();
            if parts.len() == 2 {
                if let Ok(bank) = parts[0].parse::<usize>() {
                    layout.entry(bank).or_default().remainder_label = parts[1].to_owned();
                }
            }
        }
    }

    layout
}

#[derive(Clone, Debug)]
pub struct ProjectImage {
    pub chip: Chip,
    pub layout: RomLayout,
    pub pad_byte: u8,
    pub banks: Vec<Option<RomBank>>,
}

impl ProjectImage {
    pub fn new(chip: Chip, layout: RomLayout) -> Self {
        Self {
            chip,
            layout,
            pad_byte: chip.pad_byte,
            banks: vec![None; chip.bank_count(layout)],
        }
    }

    /// Switches the target chip, growing or clipping the bank list to the new
    /// capacity. Banks beyond the new chip's bank count are dropped.
    pub fn set_chip(&mut self, chip: Chip) {
        self.chip = chip;
        self.banks
            .resize_with(chip.bank_count(self.layout), || None);
    }

    /// Whether switching to `chip` would drop a populated bank (any populated
    /// bank sitting beyond the new chip's bank count, even with gaps before it).
    pub fn chip_change_drops_data(&self, chip: Chip) -> bool {
        let new_count = chip.bank_count(self.layout);
        self.banks.iter().skip(new_count).any(Option::is_some)
    }

    /// Number of populated banks that would be dropped by switching to `chip`.
    pub fn dropped_bank_count(&self, chip: Chip) -> usize {
        let new_count = chip.bank_count(self.layout);
        self.banks
            .iter()
            .skip(new_count)
            .filter(|b| b.is_some())
            .count()
    }

    pub fn change_layout(&mut self, layout: RomLayout) -> Result<(), String> {
        if self.populated_count() > 0 {
            return Err("Create a new image before changing layout size".to_owned());
        }

        self.layout = layout;
        self.banks
            .resize_with(self.chip.bank_count(layout), || None);
        Ok(())
    }

    /// Re-divides the whole chip image into banks of a new layout size, splitting
    /// (or merging) the existing binary content. Per-bank names and sub-slots are
    /// not preserved; populated banks become a single slot of the new size.
    pub fn relayout(&mut self, layout: RomLayout) {
        let bytes = self.export_bytes();
        let pad_byte = self.pad_byte;
        if let Ok((image, _warning)) =
            Self::import_bytes_with(self.chip, layout, &bytes, pad_byte, &HashMap::new())
        {
            *self = image;
        }
    }

    pub fn populated_count(&self) -> usize {
        self.banks.iter().filter(|bank| bank.is_some()).count()
    }

    /// Checksums shared by two or more populated slots across all banks (and
    /// their sub-slots). Blank slots are ignored. Used to flag duplicates.
    pub fn duplicate_checksums(&self) -> HashSet<u16> {
        let mut counts: HashMap<u16, usize> = HashMap::new();
        for rom in self.banks.iter().flatten() {
            for index in 0..rom.slots.len() {
                if rom.slots[index].blank {
                    continue;
                }
                *counts.entry(rom.slot_checksum16(index)).or_default() += 1;
            }
        }
        counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(checksum, _)| checksum)
            .collect()
    }

    pub fn first_empty_bank(&self) -> Option<usize> {
        self.banks.iter().position(Option::is_none)
    }

    pub fn add_rom(&mut self, rom: RomBank) -> Result<usize, String> {
        let bank = self
            .first_empty_bank()
            .ok_or_else(|| format!("No empty banks remain for {}", self.chip.id))?;
        self.set_bank(bank, rom)?;
        Ok(bank)
    }

    pub fn set_bank(&mut self, bank: usize, rom: RomBank) -> Result<(), String> {
        validate_bank(self.chip, self.layout, bank)?;
        validate_rom_size(self.layout, &rom.data)?;
        self.banks[bank] = Some(rom);
        Ok(())
    }

    pub fn remove_bank(&mut self, bank: usize, policy: RemovePolicy) -> Result<(), String> {
        validate_bank(self.chip, self.layout, bank)?;

        match policy {
            RemovePolicy::Blank => {
                self.banks[bank] = None;
            }
            RemovePolicy::Compact => {
                self.banks.remove(bank);
                self.banks.push(None);
            }
        }

        Ok(())
    }

    pub fn export_bytes(&self) -> Vec<u8> {
        let mut output = vec![self.pad_byte; self.chip.capacity];

        for (bank_index, bank) in self.banks.iter().enumerate() {
            if let Some(rom) = bank {
                if let Some(range) = self.chip.bank_range(self.layout, bank_index) {
                    output[range].copy_from_slice(&rom.data);
                }
            }
        }

        output
    }

    pub fn metadata_text(&self, binary_name: &str) -> String {
        let mut lines = vec![
            "ROM Builder metadata".to_owned(),
            format!("Binary: {binary_name}"),
            format!("Chip: {} ({})", self.chip.id, self.chip.name),
            format!("Layout: {} ({})", self.layout.name, self.layout.id),
            format!("ROM unit size: {} bytes", self.layout.unit_size),
            format!("Chip capacity: {} bytes", self.chip.capacity),
            format!("Padding byte: 0x{:02X}", self.pad_byte),
            format!("Populated banks: {}", self.populated_count()),
            "Banks:".to_owned(),
        ];

        for (bank_index, bank) in self.banks.iter().enumerate() {
            let range = self
                .chip
                .bank_range(self.layout, bank_index)
                .expect("metadata bank range should be valid");
            if let Some(rom) = bank {
                let source = rom
                    .slots
                    .first()
                    .and_then(|slot| slot.source_path.as_ref())
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "imported from merged image".to_owned());
                lines.push(format!(
                    "- bank {bank_index}: {} | 0x{:05X}-0x{:05X} | checksum {:04X} | source: {}",
                    rom.primary_label(),
                    range.start,
                    range.end - 1,
                    rom.checksum16(),
                    source
                ));
            } else {
                lines.push(format!(
                    "- bank {bank_index}: unused | 0x{:05X}-0x{:05X}",
                    range.start,
                    range.end - 1
                ));
            }
        }

        if self.chip.remainder(self.layout) > 0 {
            let start = self.chip.bank_count(self.layout) * self.layout.unit_size;
            lines.push(format!(
                "Unused remainder: 0x{:05X}-0x{:05X} ({} bytes)",
                start,
                self.chip.capacity - 1,
                self.chip.remainder(self.layout)
            ));
        }

        // Machine-readable slot layout so re-import can rebuild sub-slots.
        lines.push(String::new());
        lines.push("Slot layout (for re-import, do not edit):".to_owned());
        for (bank_index, bank) in self.banks.iter().enumerate() {
            if let Some(rom) = bank {
                for (slot_index, slot) in rom.slots.iter().enumerate() {
                    lines.push(format!(
                        "slot|{bank_index}|{slot_index}|{}|{}|{}",
                        slot.len,
                        u8::from(slot.blank),
                        slot.label
                    ));
                }
                if rom.free() > 0 {
                    lines.push(format!("free|{bank_index}|{}", rom.remainder_label));
                }
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }

    /// Imports a chip-sized image from disk. Returns the project plus an
    /// optional warning when trailing bytes fall outside the bank grid.
    pub fn import_image(
        chip: Chip,
        layout: RomLayout,
        path: &Path,
        pad_byte: u8,
    ) -> Result<(Self, Option<String>), String> {
        let data =
            fs::read(path).map_err(|err| format!("Could not read {}: {err}", path.display()))?;
        if data.len() != chip.capacity {
            return Err(format!(
                "{} is {} bytes, but {} requires {} bytes",
                path.display(),
                data.len(),
                chip.id,
                chip.capacity
            ));
        }

        // Use the metadata sidecar (if any) to rebuild slot boundaries/names.
        let slot_layout = fs::read_to_string(metadata_path_for(path))
            .ok()
            .map(|text| parse_slot_layout(&text))
            .unwrap_or_default();

        Self::import_bytes_with(chip, layout, &data, pad_byte, &slot_layout)
    }

    /// Maps a chip-sized buffer into banks. Banks present in `slot_layout` are
    /// rebuilt with their recorded sub-slots; the rest are placed linearly as a
    /// single slot (skipping all-padding banks). Any remainder region beyond the
    /// last full bank is checked for real data so the caller can warn.
    pub fn import_bytes_with(
        chip: Chip,
        layout: RomLayout,
        data: &[u8],
        pad_byte: u8,
        slot_layout: &HashMap<usize, BankLayout>,
    ) -> Result<(Self, Option<String>), String> {
        if data.len() != chip.capacity {
            return Err(format!(
                "image is {} bytes, but {} requires {} bytes",
                data.len(),
                chip.id,
                chip.capacity
            ));
        }

        let mut image = Self::new(chip, layout);
        image.pad_byte = pad_byte;

        for bank_index in 0..chip.bank_count(layout) {
            let range = chip
                .bank_range(layout, bank_index)
                .expect("bank range should be valid during import");
            let bank_data = &data[range];

            if let Some(bank_layout) = slot_layout.get(&bank_index) {
                if !bank_layout.slots.is_empty() {
                    let remainder_label = if bank_layout.remainder_label.is_empty() {
                        default_remainder_label()
                    } else {
                        bank_layout.remainder_label.clone()
                    };
                    image.banks[bank_index] = Some(RomBank::from_layout(
                        bank_data,
                        &bank_layout.slots,
                        remainder_label,
                    )?);
                    continue;
                }
            }

            if bank_data.iter().all(|byte| *byte == pad_byte) {
                continue;
            }

            image.banks[bank_index] = Some(RomBank::from_slice(
                layout,
                format!("{} bank {bank_index}", chip.id),
                bank_data,
            )?);
        }

        // Bytes past the last full bank cannot become a bank in this model. If
        // they are all padding the import still round-trips losslessly; if they
        // carry data, warn so the user knows it will be lost on export.
        let mapped = chip.bank_count(layout) * layout.unit_size;
        let warning = if mapped < data.len() && data[mapped..].iter().any(|byte| *byte != pad_byte)
        {
            Some(format!(
                "{} trailing byte(s) at 0x{:05X}-0x{:05X} fall beyond the last full {} bank \
                 and were not imported; they will be padded on export.",
                data.len() - mapped,
                mapped,
                data.len() - 1,
                layout.name
            ))
        } else {
            None
        };

        Ok((image, warning))
    }
}

pub fn validate_rom_size(layout: RomLayout, data: &[u8]) -> Result<(), String> {
    if data.len() == layout.unit_size {
        Ok(())
    } else {
        Err(format!(
            "{} ROM must be exactly {} bytes, got {} bytes",
            layout.name,
            layout.unit_size,
            data.len()
        ))
    }
}

pub fn validate_bank(chip: Chip, layout: RomLayout, bank: usize) -> Result<(), String> {
    if bank < chip.bank_count(layout) {
        Ok(())
    } else {
        Err(format!(
            "Bank {bank} is outside the valid range 0..{} for {} using {}",
            chip.bank_count(layout).saturating_sub(1),
            chip.id,
            layout.name
        ))
    }
}

pub fn metadata_path_for(binary_path: &Path) -> PathBuf {
    binary_path.with_extension("metadata.txt")
}

pub fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::write(path, bytes).map_err(|err| format!("Could not write {}: {err}", path.display()))
}

pub fn write_text_file(path: &Path, text: &str) -> Result<(), String> {
    fs::write(path, text).map_err(|err| format!("Could not write {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_bank_counts_follow_layout_size() {
        assert_eq!(CHIPS[0].bank_count(LAYOUTS[0]), 4);
        assert_eq!(CHIPS[1].bank_count(LAYOUTS[0]), 32);
        assert_eq!(CHIPS[2].bank_count(LAYOUTS[0]), 32);
        assert_eq!(CHIPS[0].bank_count(LAYOUTS[1]), 1);
        assert_eq!(CHIPS[1].bank_count(LAYOUTS[1]), 10);
    }

    #[test]
    fn bank_ranges_are_linear() {
        let chip = CHIPS[0];
        let layout = LAYOUTS[0];

        assert_eq!(chip.bank_range(layout, 0), Some(0x0000..0x4000));
        assert_eq!(chip.bank_range(layout, 1), Some(0x4000..0x8000));
        assert_eq!(chip.bank_range(layout, 3), Some(0xC000..0x10000));
        assert_eq!(chip.bank_range(layout, 4), None);
    }

    #[test]
    fn export_is_chip_sized_and_padded() {
        let chip = CHIPS[0];
        let layout = LAYOUTS[0];
        let mut image = ProjectImage::new(chip, layout);
        image.pad_byte = 0xAA;
        image
            .set_bank(
                1,
                RomBank::from_slice(layout, "test".to_owned(), &[0x11; 16 * 1024]).unwrap(),
            )
            .unwrap();

        let output = image.export_bytes();

        assert_eq!(output.len(), chip.capacity);
        assert_eq!(output[0], 0xAA);
        assert_eq!(output[layout.unit_size], 0x11);
        assert_eq!(output[layout.unit_size * 2], 0xAA);
    }

    #[test]
    fn compact_remove_shifts_later_banks() {
        let chip = CHIPS[0];
        let layout = LAYOUTS[0];
        let mut image = ProjectImage::new(chip, layout);
        image
            .set_bank(
                0,
                RomBank::from_slice(layout, "a".to_owned(), &[0x10; 16 * 1024]).unwrap(),
            )
            .unwrap();
        image
            .set_bank(
                1,
                RomBank::from_slice(layout, "b".to_owned(), &[0x20; 16 * 1024]).unwrap(),
            )
            .unwrap();

        image.remove_bank(0, RemovePolicy::Compact).unwrap();

        assert_eq!(image.banks[0].as_ref().unwrap().data[0], 0x20);
        assert!(image.banks[3].is_none());
    }

    #[test]
    fn metadata_lists_rom_names() {
        let layout = LAYOUTS[0];
        let mut image = ProjectImage::new(CHIPS[0], layout);
        image
            .set_bank(
                0,
                RomBank::from_slice(layout, "Diagnostics".to_owned(), &[0x42; 16 * 1024]).unwrap(),
            )
            .unwrap();

        let metadata = image.metadata_text("test.bin");

        assert!(metadata.contains("Diagnostics"));
        assert!(metadata.contains("bank 0"));
    }

    #[test]
    fn partial_bank_pads_remainder() {
        let layout = LAYOUTS[0]; // 16 KiB
        let mut rom =
            RomBank::from_partial(layout, "Tiny".to_owned(), &[0x11; 1000], 0xFF).unwrap();

        assert_eq!(rom.data.len(), layout.unit_size);
        assert_eq!(rom.used(), 1000);
        assert_eq!(rom.free(), layout.unit_size - 1000);
        assert_eq!(rom.data[999], 0x11);
        assert_eq!(rom.data[1000], 0xFF);
        assert_eq!(rom.slot_checksum16(0), 17_000);

        // Fill part of the free space with a second ROM -> a new slot.
        rom.add_into_free("More".to_owned(), None, &[0x22; 500])
            .unwrap();
        assert_eq!(rom.slots.len(), 2);
        assert_eq!(rom.used(), 1500);
        assert_eq!(rom.slot_start(1), 1000);
        assert_eq!(rom.data[1000], 0x22);
        assert_eq!(rom.data[1499], 0x22);
        assert_eq!(rom.data[1500], 0xFF);
        assert_eq!(rom.slot_checksum16(1), 17_000); // 0x22 * 500

        // A ROM larger than the bank is rejected.
        assert!(
            RomBank::from_partial(layout, "Big".to_owned(), &[0x00; 16 * 1024 + 1], 0xFF).is_err()
        );
        // A ROM larger than the remaining free space is rejected.
        assert!(rom
            .add_into_free("X".to_owned(), None, &[0x00; 16 * 1024])
            .is_err());
    }

    #[test]
    fn remove_slot_shifts_and_pads() {
        let layout = LAYOUTS[0];
        let mut rom = RomBank::from_partial(layout, "A".to_owned(), &[0x11; 1000], 0xFF).unwrap();
        rom.add_into_free("B".to_owned(), None, &[0x22; 500])
            .unwrap();
        rom.add_into_free("C".to_owned(), None, &[0x33; 200])
            .unwrap();
        assert_eq!(rom.used(), 1700);

        rom.remove_slot(1, RemovePolicy::Compact, 0xFF); // remove B; C shifts into its place

        assert_eq!(rom.slots.len(), 2);
        assert_eq!(rom.slots[0].label, "A");
        assert_eq!(rom.slots[1].label, "C");
        assert_eq!(rom.used(), 1200);
        assert_eq!(rom.data[999], 0x11);
        assert_eq!(rom.data[1000], 0x33);
        assert_eq!(rom.data[1199], 0x33);
        assert_eq!(rom.data[1200], 0xFF);
    }

    #[test]
    fn remove_slot_blank_keeps_position() {
        let layout = LAYOUTS[0];
        let mut rom = RomBank::from_partial(layout, "A".to_owned(), &[0x11; 1000], 0xFF).unwrap();
        rom.add_into_free("B".to_owned(), None, &[0x22; 500])
            .unwrap();
        rom.add_into_free("C".to_owned(), None, &[0x33; 200])
            .unwrap();

        rom.remove_slot(1, RemovePolicy::Blank, 0xFF); // blank B in place

        assert_eq!(rom.slots.len(), 3); // slot kept
        assert!(rom.slots[1].blank);
        assert_eq!(rom.used(), 1700); // occupied space retained
        assert_eq!(rom.data[1000], 0xFF); // B region blanked
        assert_eq!(rom.data[1499], 0xFF);
        assert_eq!(rom.data[1500], 0x33); // C unchanged in place
    }

    #[test]
    fn duplicate_checksums_finds_repeats() {
        let chip = CHIPS[0];
        let layout = LAYOUTS[0];
        let mut image = ProjectImage::new(chip, layout);
        image
            .set_bank(
                0,
                RomBank::from_slice(layout, "a".to_owned(), &[0x11; 16 * 1024]).unwrap(),
            )
            .unwrap();
        image
            .set_bank(
                1,
                RomBank::from_slice(layout, "b".to_owned(), &[0x11; 16 * 1024]).unwrap(),
            )
            .unwrap();
        image
            .set_bank(
                2,
                RomBank::from_slice(layout, "c".to_owned(), &[0x22; 16 * 1024]).unwrap(),
            )
            .unwrap();

        let duplicates = image.duplicate_checksums();
        let shared = image.banks[0].as_ref().unwrap().slot_checksum16(0);
        let unique = image.banks[2].as_ref().unwrap().slot_checksum16(0);
        assert!(duplicates.contains(&shared));
        assert!(!duplicates.contains(&unique));
    }

    #[test]
    fn duplicate_detection_spans_sub_slots() {
        let chip = CHIPS[0];
        let layout = LAYOUTS[0]; // 16 KiB
        let mut image = ProjectImage::new(chip, layout);

        // Bank 0: unique primary plus a sub-slot that matches bank 1's ROM.
        let mut bank0 = RomBank::from_partial(layout, "a".to_owned(), &[0x11; 4000], 0xFF).unwrap();
        bank0
            .add_into_free("dup".to_owned(), None, &[0x55; 2000])
            .unwrap();
        image.set_bank(0, bank0).unwrap();
        image
            .set_bank(
                1,
                RomBank::from_partial(layout, "b".to_owned(), &[0x55; 2000], 0xFF).unwrap(),
            )
            .unwrap();

        let duplicates = image.duplicate_checksums();
        // Both banks should be flagged: the match is between bank 0's sub-slot
        // and bank 1's primary slot.
        assert!(image.banks[0]
            .as_ref()
            .unwrap()
            .has_duplicate_slot(&duplicates));
        assert!(image.banks[1]
            .as_ref()
            .unwrap()
            .has_duplicate_slot(&duplicates));
        // Bank 0's primary content is unique.
        let primary = image.banks[0].as_ref().unwrap().slot_checksum16(0);
        assert!(!duplicates.contains(&primary));
    }

    #[test]
    fn relayout_splits_content_into_smaller_banks() {
        let chip = CHIPS[0]; // W27C512, 64 KiB
        let big = LAYOUTS
            .iter()
            .copied()
            .find(|layout| layout.unit_size == 32 * 1024)
            .unwrap();
        let small = LAYOUTS[0]; // 16 KiB

        let mut image = ProjectImage::new(chip, big);
        image
            .set_bank(
                0,
                RomBank::from_slice(big, "a".to_owned(), &[0x11; 32 * 1024]).unwrap(),
            )
            .unwrap();
        assert_eq!(image.banks.len(), 2);

        image.relayout(small);

        assert_eq!(image.layout.unit_size, 16 * 1024);
        assert_eq!(image.banks.len(), 4);
        assert!(image.banks[0].is_some());
        assert!(image.banks[1].is_some());
        assert!(image.banks[2].is_none());
        assert!(image.banks[3].is_none());
        assert_eq!(image.banks[0].as_ref().unwrap().data[0], 0x11);
    }

    #[test]
    fn chip_change_detects_and_clips_data_beyond_capacity() {
        let layout = LAYOUTS[0]; // 16 KiB banks
        let big = CHIPS[1]; // AM29F040B, 32 banks
        let small = CHIPS[0]; // W27C512, 4 banks

        let mut image = ProjectImage::new(big, layout);
        image
            .set_bank(
                0,
                RomBank::from_slice(layout, "a".to_owned(), &[0x11; 16 * 1024]).unwrap(),
            )
            .unwrap();
        // Bank 10 is populated with a gap before it; count-based checks miss this.
        image
            .set_bank(
                10,
                RomBank::from_slice(layout, "b".to_owned(), &[0x22; 16 * 1024]).unwrap(),
            )
            .unwrap();

        assert!(image.chip_change_drops_data(small));
        assert_eq!(image.dropped_bank_count(small), 1);

        image.set_chip(small);
        assert_eq!(image.banks.len(), 4);
        assert!(image.banks[0].is_some());

        // A chip that still covers every populated bank drops nothing.
        let mut keepable = ProjectImage::new(big, layout);
        keepable
            .set_bank(
                3,
                RomBank::from_slice(layout, "c".to_owned(), &[0x33; 16 * 1024]).unwrap(),
            )
            .unwrap();
        assert!(!keepable.chip_change_drops_data(small));
    }

    #[test]
    fn collapse_if_unused_folds_blank_slots() {
        let layout = LAYOUTS[0];
        let mut rom = RomBank::from_partial(layout, "A".to_owned(), &[0x11; 1000], 0xFF).unwrap();
        rom.add_into_free("B".to_owned(), None, &[0x22; 500])
            .unwrap();

        // Blank the only sub-slot: primary stays, blank sub-slot folds into free.
        rom.remove_slot(1, RemovePolicy::Blank, 0xFF);
        assert!(!rom.collapse_if_unused());
        assert_eq!(rom.slots.len(), 1);
        assert_eq!(rom.slots[0].label, "A");
        assert_eq!(rom.used(), 1000);

        // Blank the primary too: now everything is unused.
        rom.remove_slot(0, RemovePolicy::Blank, 0xFF);
        assert!(rom.collapse_if_unused());
    }

    #[test]
    fn slot_layout_round_trips_through_metadata() {
        let chip = CHIPS[0]; // 64 KiB
        let layout = LAYOUTS[0]; // 16 KiB banks
        let mut image = ProjectImage::new(chip, layout);
        let mut rom =
            RomBank::from_partial(layout, "First".to_owned(), &[0x11; 1000], 0xFF).unwrap();
        rom.add_into_free("Second".to_owned(), None, &[0x22; 500])
            .unwrap();
        rom.remainder_label = "Spare".to_owned();
        image.set_bank(0, rom).unwrap();

        let data = image.export_bytes();
        let metadata = image.metadata_text("test.bin");
        let parsed = parse_slot_layout(&metadata);
        let (reimported, _warning) =
            ProjectImage::import_bytes_with(chip, layout, &data, 0xFF, &parsed).unwrap();

        let bank0 = reimported.banks[0].as_ref().unwrap();
        assert_eq!(bank0.slots.len(), 2);
        assert_eq!(bank0.slots[0].len, 1000);
        assert_eq!(bank0.slots[0].label, "First");
        assert_eq!(bank0.slots[1].len, 500);
        assert_eq!(bank0.slots[1].label, "Second");
        assert_eq!(bank0.remainder_label, "Spare");
        assert_eq!(bank0.data[0], 0x11);
        assert_eq!(bank0.data[1000], 0x22);
    }

    #[test]
    fn import_warns_only_when_remainder_has_data() {
        // W27C512 (64 KiB) + QL 48K layout = 1 full bank, 16 KiB remainder.
        let chip = CHIPS[0];
        let layout = LAYOUTS[1];
        let pad = 0xFF;
        assert!(chip.remainder(layout) > 0);

        // Bank populated, remainder all padding: lossless, no warning.
        let mut data = vec![pad; chip.capacity];
        data[..layout.unit_size].fill(0x11);
        let (image, warning) =
            ProjectImage::import_bytes_with(chip, layout, &data, pad, &HashMap::new()).unwrap();
        assert_eq!(image.populated_count(), 1);
        assert!(warning.is_none());
        assert_eq!(
            image.export_bytes(),
            data,
            "padding-only tail must round-trip"
        );

        // Real data in the remainder region: bank still imported, but warn.
        data[layout.unit_size] = 0x42;
        let (image, warning) =
            ProjectImage::import_bytes_with(chip, layout, &data, pad, &HashMap::new()).unwrap();
        assert_eq!(image.populated_count(), 1);
        assert!(warning.is_some());
    }
}
