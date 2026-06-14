mod core;
mod project;

use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{
    metadata_path_for, write_file, write_text_file, Chip, ProjectImage, RemovePolicy, RomBank,
    RomLayout, CHIPS, LAYOUTS,
};
use crate::project::ProjectSettings;
use eframe::egui;

const HEX_PREVIEW_BYTES: usize = 512;
const ROM_NAME_FIELD_WIDTH: f32 = 210.0;
const ROM_NAME_MAX_CHARS: usize = 30;
const ROM_NAME_SUGGESTION_LIMIT: usize = 30;

const BANK_UNUSED_COLOR: egui::Color32 = egui::Color32::from_rgb(68, 74, 86);
const BANK_FILLED_COLOR: egui::Color32 = egui::Color32::from_rgb(76, 132, 96);
const BANK_DUPLICATE_COLOR: egui::Color32 = egui::Color32::from_rgb(214, 132, 40);

const APP_ICON_PNG: &[u8] = include_bytes!("../images/puzzle.png");

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1120.0, 780.0])
        .with_maximized(true);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(APP_ICON_PNG) {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "ROM Builder Beta",
        options,
        Box::new(|_cc| Box::new(RomBuilderApp::default())),
    )
}

struct RomBuilderApp {
    image: ProjectImage,
    selected_layout: usize,
    selected_chip: usize,
    selected_bank: Option<usize>,
    bank_map_page: usize,
    multi_rom_image: bool,
    allow_larger_chip: bool,
    remove_policy: RemovePolicy,
    pad_text: String,
    status: String,
    full_hex_open: bool,
    hex_preview_visible: bool,
    hex_preview_offset: usize,
    pending_trim_replace: Option<PendingTrimReplace>,
    pending_remove_bank: Option<usize>,
    pending_remove_slot: Option<(usize, usize)>,
    pending_name_suggestions: Option<NameSuggestions>,
    pending_oversized_add: Option<PendingOversizedAdd>,
    pending_chip_change: Option<PendingChipChange>,
    pending_layout_change: Option<(RomLayout, usize)>,
    maximized: bool,
    project_active: bool,
    current_project_path: Option<PathBuf>,
    dirty: bool,
    pending_close: bool,
    allow_close: bool,
    show_platform_picker: bool,
    platform_filter: String,
    show_ic_picker: bool,
    ic_filter: String,
    show_about: bool,
    about_logo: Option<egui::TextureHandle>,
}

struct PendingOversizedAdd {
    path: PathBuf,
    label: String,
    data: Vec<u8>,
    bank: usize,
}

struct PendingTrimReplace {
    bank: usize,
    path: PathBuf,
    label: String,
    data: Vec<u8>,
}

struct NameSuggestions {
    bank: usize,
    slot: usize,
    suggestions: Vec<String>,
}

struct PendingChipChange {
    chip: Chip,
    selected_chip: usize,
    /// When the change was triggered by unchecking Multi-rom image, cancelling
    /// the confirmation re-enables the checkbox.
    reenable_multi_rom: bool,
}

impl Default for RomBuilderApp {
    fn default() -> Self {
        let chip = CHIPS[0];
        let layout = LAYOUTS[0];

        Self {
            image: ProjectImage::new(chip, layout),
            selected_layout: 0,
            selected_chip: 0,
            selected_bank: None,
            bank_map_page: 0,
            multi_rom_image: true,
            allow_larger_chip: false,
            remove_policy: RemovePolicy::Blank,
            pad_text: "FF".to_owned(),
            status: "Ready".to_owned(),
            full_hex_open: false,
            hex_preview_visible: false,
            hex_preview_offset: 0,
            pending_trim_replace: None,
            pending_remove_bank: None,
            pending_remove_slot: None,
            pending_name_suggestions: None,
            pending_oversized_add: None,
            pending_chip_change: None,
            pending_layout_change: None,
            maximized: false,
            project_active: false,
            current_project_path: None,
            dirty: false,
            pending_close: false,
            allow_close: false,
            show_platform_picker: false,
            platform_filter: String::new(),
            show_ic_picker: false,
            ic_filter: String::new(),
            show_about: false,
            about_logo: None,
        }
    }
}

impl eframe::App for RomBuilderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Ensure the window opens maximized to the full screen, even where the
        // initial viewport flag is not honored at launch.
        if !self.maximized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            self.maximized = true;
        }

        self.handle_shortcuts(ctx);

        // Intercept the window close when there are unsaved changes.
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending_close = true;
        }

        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::same(10.0)))
            .show(ctx, |ui| {
                self.toolbar(ui);
            });

        egui::SidePanel::left("settings")
            .resizable(false)
            .default_width(280.0)
            .frame(
                egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin {
                    left: 10.0,
                    right: 8.0,
                    top: 10.0,
                    bottom: 8.0,
                }),
            )
            .show(ctx, |ui| {
                self.settings_panel(ui);
            });

        egui::TopBottomPanel::bottom("status")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(&self.status);
                });
            });

        egui::TopBottomPanel::bottom("info")
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(245, 245, 245))
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0)),
            )
            .show(ctx, |ui| {
                self.info_bar(ui);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin {
                    left: 10.0,
                    right: 8.0,
                    top: 10.0,
                    bottom: 8.0,
                }),
            )
            .show(ctx, |ui| {
                ui.set_enabled(self.project_active);
                if self.hex_preview_visible && self.selected_rom().is_some() {
                    egui::TopBottomPanel::bottom("hex_preview")
                        .resizable(true)
                        .default_height(260.0)
                        .height_range(140.0..=520.0)
                        .show_inside(ui, |ui| {
                            self.hex_preview(ui);
                        });
                }

                ui.heading("Bank Map");
                self.memory_map(ui);
                ui.add_space(12.0);
                self.bank_table(ui);
            });

        if self.full_hex_open {
            let mut open = self.full_hex_open;
            egui::Window::new("Full Hex View")
                .open(&mut open)
                .vscroll(true)
                .resizable(true)
                .default_size([920.0, 620.0])
                .show(ctx, |ui| {
                    if let Some((bank, rom)) = self.selected_rom() {
                        ui.label(format!("Bank {bank}: {}", rom.primary_label()));
                        ui.separator();
                        let hex = format_hex(&rom.data, 0, rom.data.len());
                        ui.add(
                            egui::TextEdit::multiline(&mut hex.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    } else {
                        ui.label("Select a populated bank to view its content.");
                    }
                });
            self.full_hex_open = open;
        }

        if self.pending_trim_replace.is_some() {
            self.trim_replace_dialog(ctx);
        }

        if self.pending_remove_bank.is_some() {
            self.remove_confirmation_dialog(ctx);
        }

        if self.pending_remove_slot.is_some() {
            self.remove_slot_confirmation_dialog(ctx);
        }

        if self.pending_name_suggestions.is_some() {
            self.name_suggestions_dialog(ctx);
        }

        if self.pending_oversized_add.is_some() {
            self.oversized_add_dialog(ctx);
        }

        if self.show_about {
            self.about_window(ctx);
        }

        if self.pending_chip_change.is_some() {
            self.chip_change_dialog(ctx);
        }

        if self.pending_layout_change.is_some() {
            self.layout_change_dialog(ctx);
        }

        if self.pending_close {
            self.close_confirm_dialog(ctx);
        }

        if self.show_platform_picker {
            self.platform_picker_dialog(ctx);
        }

        if self.show_ic_picker {
            self.ic_picker_dialog(ctx);
        }
    }
}

impl RomBuilderApp {
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (new, open, save, export) = ctx.input(|i| {
            let cmd = i.modifiers.command;
            (
                cmd && i.key_pressed(egui::Key::N),
                cmd && i.key_pressed(egui::Key::O),
                cmd && i.key_pressed(egui::Key::S),
                cmd && i.key_pressed(egui::Key::E),
            )
        });

        if new {
            self.new_project();
        }
        if open {
            self.open_project();
        }
        if save && self.project_active && self.dirty {
            self.save_project();
        }
        if export && self.project_active {
            self.export_image();
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().button_padding = egui::vec2(5.0, 5.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("New Project (Ctrl+N)").clicked() {
                self.new_project();
            }
            if ui.button("Open Project (Ctrl+O)").clicked() {
                self.open_project();
            }

            let active = self.project_active;
            let save_button = egui::Button::new("Save (Ctrl+S)");
            let save_button = if self.dirty {
                save_button.fill(egui::Color32::from_rgb(70, 130, 210))
            } else {
                save_button
            };
            if ui.add_enabled(active && self.dirty, save_button).clicked() {
                self.save_project();
            }
            if ui
                .add_enabled(active, egui::Button::new("Save As"))
                .clicked()
            {
                self.save_project_as();
            }

            ui.separator();

            if ui
                .add_enabled(active, egui::Button::new("Add ROM"))
                .clicked()
            {
                self.add_roms();
            }
            if ui
                .add_enabled(active, egui::Button::new("Export Binary (Ctrl+E)"))
                .clicked()
            {
                self.export_image();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("About").clicked() {
                    self.show_about = true;
                }
            });
        });
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.add_space(8.0);
        ui.set_enabled(self.project_active);

        let layout_size = self.current_layout().unit_size;
        ui.label("ROM Layout");
        let mut sizes = LAYOUTS
            .iter()
            .map(|layout| layout.unit_size)
            .collect::<Vec<_>>();
        sizes.sort_unstable();
        sizes.dedup();
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_source("rom_layout_combo")
                .selected_text(layout_size_label(layout_size))
                .show_ui(ui, |ui| {
                    for size in sizes {
                        if ui
                            .selectable_label(layout_size == size, layout_size_label(size))
                            .clicked()
                        {
                            if let Some(index) =
                                LAYOUTS.iter().position(|layout| layout.unit_size == size)
                            {
                                self.apply_layout_change(LAYOUTS[index], index);
                            }
                            ui.close_menu();
                        }
                    }
                });
            if ui
                .button("<-")
                .on_hover_text("Pick a platform's ROM size")
                .clicked()
            {
                self.show_platform_picker = true;
                self.platform_filter.clear();
            }
        });

        ui.add_space(8.0);
        ui.label("Target IC size");
        let mut ic_sizes = (0..CHIPS.len())
            .filter(|&index| self.chip_is_eligible(CHIPS[index], layout_size))
            .map(|index| CHIPS[index].capacity)
            .collect::<Vec<_>>();
        ic_sizes.sort_unstable();
        ic_sizes.dedup();
        let current_capacity = self.current_chip().capacity;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_source("target_chip_combo")
                .selected_text(layout_size_label(current_capacity))
                .show_ui(ui, |ui| {
                    for size in ic_sizes {
                        if ui
                            .selectable_label(current_capacity == size, layout_size_label(size))
                            .clicked()
                        {
                            if size != current_capacity {
                                if let Some(index) = (0..CHIPS.len()).find(|&index| {
                                    CHIPS[index].capacity == size
                                        && self.chip_is_eligible(CHIPS[index], layout_size)
                                }) {
                                    self.apply_chip_change(CHIPS[index], index);
                                }
                            }
                            ui.close_menu();
                        }
                    }
                });
            if ui
                .button("<-")
                .on_hover_text("Pick a specific IC")
                .clicked()
            {
                self.show_ic_picker = true;
                self.ic_filter.clear();
            }
        });

        if ui
            .checkbox(&mut self.multi_rom_image, "Multi-rom image")
            .changed()
        {
            self.dirty = true;
            let disabling = !self.multi_rom_image;
            // Entering single-ROM mode defaults to the smallest fitting chip.
            if disabling {
                self.allow_larger_chip = true;
            }
            self.enforce_chip_eligibility();
            // If disabling triggered a clip confirmation, cancelling it should
            // restore the checkbox.
            if disabling {
                if let Some(pending) = &mut self.pending_chip_change {
                    pending.reenable_multi_rom = true;
                }
            }
        }

        if !self.multi_rom_image
            && ui
                .checkbox(&mut self.allow_larger_chip, "Smallest chip ≥ layout size")
                .changed()
        {
            self.dirty = true;
            self.enforce_chip_eligibility();
        }

        ui.add_space(8.0);
        ui.label("Pad byte");
        let response = ui.text_edit_singleline(&mut self.pad_text);
        if response.changed() {
            match parse_pad_byte(&self.pad_text) {
                Ok(byte) => {
                    self.image.pad_byte = byte;
                    self.dirty = true;
                    self.status = format!("Padding byte set to 0x{byte:02X}");
                }
                Err(message) => {
                    self.status = message;
                }
            }
        }

        ui.add_space(8.0);
        ui.label("Remove policy");
        let mut policy_changed = false;
        egui::ComboBox::from_id_source("remove_policy_combo")
            .selected_text(self.remove_policy.label())
            .show_ui(ui, |ui| {
                policy_changed |= ui
                    .selectable_value(
                        &mut self.remove_policy,
                        RemovePolicy::Blank,
                        RemovePolicy::Blank.label(),
                    )
                    .changed();
                policy_changed |= ui
                    .selectable_value(
                        &mut self.remove_policy,
                        RemovePolicy::Compact,
                        RemovePolicy::Compact.label(),
                    )
                    .changed();
            });
        if policy_changed {
            self.dirty = true;
        }
    }

    fn info_bar(&mut self, ui: &mut egui::Ui) {
        ui.visuals_mut().override_text_color = Some(egui::Color32::BLACK);
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Chip size: {} bytes ({})",
                self.image.chip.capacity,
                kib_label(self.image.chip.capacity)
            ));
            ui.separator();
            ui.label(format!(
                "ROM unit: {} bytes ({})",
                self.image.layout.unit_size,
                kib_label(self.image.layout.unit_size)
            ));
            ui.separator();
            ui.label(format!(
                "Banks: {}",
                self.image.chip.bank_count(self.image.layout)
            ));
            ui.separator();
            ui.label(format!("Populated: {}", self.image.populated_count()));
            ui.separator();
            let output_len = self.image.export_bytes().len();
            ui.label(format!(
                "Output: {output_len} bytes ({})",
                kib_label(output_len)
            ));
            let remainder = self.image.chip.remainder(self.image.layout);
            if remainder > 0 {
                ui.separator();
                ui.label(format!(
                    "Padded remainder: {remainder} bytes ({})",
                    kib_label(remainder)
                ));
            }
        });
    }

    fn bank_map_legend(ui: &mut egui::Ui) {
        fn swatch(ui: &mut egui::Ui, color: egui::Color32, label: &str) {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, color);
            ui.label(label);
        }

        ui.horizontal(|ui| {
            swatch(ui, BANK_UNUSED_COLOR, "Unused");
            ui.add_space(10.0);
            swatch(ui, BANK_FILLED_COLOR, "Filled");
            ui.add_space(10.0);
            swatch(ui, BANK_DUPLICATE_COLOR, "Duplicated");
        });
    }

    fn memory_map(&mut self, ui: &mut egui::Ui) {
        const MAX_ROWS: usize = 4;
        Self::bank_map_legend(ui);
        ui.add_space(6.0);

        let duplicates = self.image.duplicate_checksums();
        let bank_count = self.image.chip.bank_count(self.image.layout);
        let columns = if bank_count <= 4 { 4 } else { 12 };
        let per_page = columns * MAX_ROWS;
        let total_pages = bank_count.div_ceil(per_page).max(1);

        if self.bank_map_page >= total_pages {
            self.bank_map_page = total_pages - 1;
        }

        let start = self.bank_map_page * per_page;
        let end = (start + per_page).min(bank_count);

        if total_pages > 1 {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.bank_map_page > 0, egui::Button::new("Prev"))
                    .clicked()
                {
                    self.bank_map_page -= 1;
                }
                ui.label(format!(
                    "Banks {start}-{} (page {} of {total_pages})",
                    end.saturating_sub(1),
                    self.bank_map_page + 1
                ));
                if ui
                    .add_enabled(
                        self.bank_map_page + 1 < total_pages,
                        egui::Button::new("Next"),
                    )
                    .clicked()
                {
                    self.bank_map_page += 1;
                }
            });
        }

        egui::Grid::new("memory_map_grid")
            .num_columns(columns)
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                for bank in start..end {
                    let occupied = self.image.banks[bank].is_some();
                    let duplicate = self.image.banks[bank]
                        .as_ref()
                        .is_some_and(|rom| rom.has_duplicate_slot(&duplicates));
                    let selected = self.selected_bank == Some(bank);
                    let text = format!("{bank}");
                    let mut button = egui::Button::new(text).min_size([54.0, 32.0].into());

                    button = button.fill(if duplicate {
                        BANK_DUPLICATE_COLOR
                    } else if occupied {
                        BANK_FILLED_COLOR
                    } else {
                        BANK_UNUSED_COLOR
                    });

                    if selected {
                        button = button.stroke(egui::Stroke::new(2.0, egui::Color32::YELLOW));
                    }

                    let range = self
                        .image
                        .chip
                        .bank_range(self.image.layout, bank)
                        .expect("visible bank should be valid");
                    let response = ui.add(button).on_hover_text(format!(
                        "0x{:05X}-0x{:05X}\nDouble-click to load a ROM",
                        range.start,
                        range.end - 1
                    ));

                    if response.clicked() {
                        self.select_bank(bank);
                    }

                    if response.double_clicked() {
                        self.add_rom_into_bank(bank);
                    }

                    if (bank + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn bank_table(&mut self, ui: &mut egui::Ui) {
        ui.heading("Banks");

        let duplicates = self.image.duplicate_checksums();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("bank_table")
                .num_columns(8)
                .striped(true)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    ui.strong("Bank");
                    ui.strong("Offset");
                    ui.strong("Name");
                    ui.strong("Suggest");
                    ui.strong("Checksum");
                    ui.strong("Replace");
                    ui.strong("Extract");
                    ui.strong("Remove");
                    ui.end_row();

                    for bank in 0..self.image.chip.bank_count(self.image.layout) {
                        let range = self
                            .image
                            .chip
                            .bank_range(self.image.layout, bank)
                            .expect("visible bank should be valid");

                        if self.image.banks[bank].is_none() {
                            let selected = self.selected_bank == Some(bank);
                            let label = if selected {
                                format!("> {bank}")
                            } else {
                                bank.to_string()
                            };
                            let response = ui.selectable_label(selected, label);
                            if response.clicked() {
                                self.select_bank(bank);
                            }
                            if response.double_clicked() {
                                self.add_rom_into_bank(bank);
                            }
                            ui.monospace(format!("0x{:05X}-0x{:05X}", range.start, range.end - 1));
                            ui.label(format!(
                                "Unused ({})",
                                unused_size_label(range.end - range.start)
                            ));
                            ui.add_enabled(false, egui::Button::new("Suggest"));
                            ui.label("-");
                            if ui.button("Replace").clicked() {
                                self.replace_bank(bank);
                            }
                            ui.add_enabled(false, egui::Button::new("Extract"));
                            ui.add_enabled(false, egui::Button::new("Remove"));
                            ui.end_row();
                            continue;
                        }

                        let slot_count = self.image.banks[bank].as_ref().unwrap().slots.len();
                        let free = self.image.banks[bank].as_ref().unwrap().free();

                        for slot in 0..slot_count {
                            let (slot_start, slot_len, checksum, blank) = {
                                let rom = self.image.banks[bank].as_ref().unwrap();
                                (
                                    range.start + rom.slot_start(slot),
                                    rom.slots[slot].len,
                                    rom.slot_checksum16(slot),
                                    rom.slots[slot].blank,
                                )
                            };
                            let duplicate_slot = !blank && duplicates.contains(&checksum);

                            let selected = self.selected_bank == Some(bank) && slot == 0;
                            let index = if slot == 0 {
                                bank.to_string()
                            } else {
                                format!("{bank}.{slot}")
                            };
                            let display = if selected {
                                format!("> {index}")
                            } else {
                                index.clone()
                            };
                            let display = if duplicate_slot {
                                egui::RichText::new(display).color(BANK_DUPLICATE_COLOR)
                            } else {
                                egui::RichText::new(display)
                            };
                            let response = ui.selectable_label(selected, display);
                            if response.clicked() {
                                self.select_bank(bank);
                            }
                            if response.double_clicked() {
                                if slot == 0 {
                                    self.add_rom_into_bank(bank);
                                } else {
                                    self.load_into_slot(bank, slot);
                                }
                            }

                            let offset_text = egui::RichText::new(format!(
                                "0x{slot_start:05X}-0x{:05X}",
                                slot_start + slot_len - 1
                            ))
                            .monospace();
                            ui.label(if duplicate_slot {
                                offset_text.color(BANK_DUPLICATE_COLOR)
                            } else {
                                offset_text
                            });

                            // A blanked sub-slot behaves like an unused bank.
                            if blank {
                                ui.label(format!("Unused ({})", unused_size_label(slot_len)));
                                ui.add_enabled(false, egui::Button::new("Suggest"));
                                ui.label("-");
                            } else {
                                let name_changed = {
                                    let rom = self.image.banks[bank].as_mut().unwrap();
                                    let response = ui.add_sized(
                                        [ROM_NAME_FIELD_WIDTH, 22.0],
                                        egui::TextEdit::singleline(&mut rom.slots[slot].label)
                                            .desired_width(ROM_NAME_FIELD_WIDTH)
                                            .char_limit(ROM_NAME_MAX_CHARS),
                                    );
                                    trim_to_char_limit(
                                        &mut rom.slots[slot].label,
                                        ROM_NAME_MAX_CHARS,
                                    );
                                    let changed = response.changed();
                                    if let Some(path) = &rom.slots[slot].source_path {
                                        response.on_hover_text(path.display().to_string());
                                    }
                                    changed
                                };
                                if name_changed {
                                    self.dirty = true;
                                }

                                if ui.button("Suggest").clicked() {
                                    let suggestions = {
                                        let rom = self.image.banks[bank].as_ref().unwrap();
                                        let start = rom.slot_start(slot);
                                        suggest_rom_names(
                                            &rom.data[start..start + rom.slots[slot].len],
                                        )
                                    };
                                    if suggestions.is_empty() {
                                        self.status =
                                            format!("No name suggestions found for slot {index}");
                                    } else {
                                        self.pending_name_suggestions = Some(NameSuggestions {
                                            bank,
                                            slot,
                                            suggestions,
                                        });
                                    }
                                }

                                let checksum_text =
                                    egui::RichText::new(format!("{checksum:04X}")).monospace();
                                ui.label(if duplicate_slot {
                                    checksum_text.color(BANK_DUPLICATE_COLOR)
                                } else {
                                    checksum_text
                                });
                            }

                            if slot == 0 {
                                // Primary slot acts on the whole bank.
                                if ui.button("Replace").clicked() {
                                    self.replace_bank(bank);
                                }
                                if ui.button("Extract").clicked() {
                                    self.extract_bank(bank);
                                }
                                if ui.button("Remove").clicked() {
                                    self.pending_remove_bank = Some(bank);
                                }
                            } else if blank {
                                // Unused sub-slot: only Replace (to fill it).
                                if ui.button("Replace").clicked() {
                                    self.load_into_slot(bank, slot);
                                }
                                ui.add_enabled(false, egui::Button::new("Extract"));
                                ui.add_enabled(false, egui::Button::new("Remove"));
                            } else {
                                // Occupied sub-slot acts on the slot.
                                if ui.button("Replace").clicked() {
                                    self.load_into_slot(bank, slot);
                                }
                                if ui.button("Extract").clicked() {
                                    self.extract_slot(bank, slot);
                                }
                                if ui.button("Remove").clicked() {
                                    self.pending_remove_slot = Some((bank, slot));
                                }
                            }

                            ui.end_row();
                        }

                        // Trailing free space shown as its own unused sub-slot,
                        // indexed after the occupied slots (bank list only).
                        if free > 0 {
                            let free_start =
                                range.start + self.image.banks[bank].as_ref().unwrap().used();
                            let response =
                                ui.selectable_label(false, format!("{bank}.{slot_count}"));
                            if response.clicked() {
                                self.select_bank(bank);
                            }
                            if response.double_clicked() {
                                self.load_into_free(bank);
                            }
                            ui.monospace(format!("0x{:05X}-0x{:05X}", free_start, range.end - 1));
                            ui.label(format!("Unused ({})", unused_size_label(free)));
                            ui.add_enabled(false, egui::Button::new("Suggest"));
                            ui.label("-");
                            if ui.button("Replace").clicked() {
                                self.load_into_free(bank);
                            }
                            ui.add_enabled(false, egui::Button::new("Extract"));
                            ui.add_enabled(false, egui::Button::new("Remove"));
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn hex_preview(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Hex Preview");
            ui.add_space(8.0);
            if ui.button("Expand").clicked() {
                self.full_hex_open = true;
            }
            if ui.button("Close").clicked() {
                self.hex_preview_visible = false;
            }
        });

        ui.separator();

        let Some(bank) = self.selected_bank else {
            ui.label("Select a populated bank to preview its initial content.");
            return;
        };
        let Some(data_len) = self
            .image
            .banks
            .get(bank)
            .and_then(Option::as_ref)
            .map(|rom| rom.data.len())
        else {
            ui.label("Select a populated bank to preview its initial content.");
            return;
        };

        // Page-aligned offset of the last window, so navigation can reach the
        // tail of the bank where neighbouring ROMs diverge from a shared header.
        let max_offset = data_len.saturating_sub(1) / HEX_PREVIEW_BYTES * HEX_PREVIEW_BYTES;
        self.hex_preview_offset = self.hex_preview_offset.min(max_offset);

        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.hex_preview_offset > 0, egui::Button::new("Prev"))
                .clicked()
            {
                self.hex_preview_offset = self.hex_preview_offset.saturating_sub(HEX_PREVIEW_BYTES);
            }
            if ui
                .add_enabled(
                    self.hex_preview_offset < max_offset,
                    egui::Button::new("Next"),
                )
                .clicked()
            {
                self.hex_preview_offset =
                    (self.hex_preview_offset + HEX_PREVIEW_BYTES).min(max_offset);
            }
        });

        let start = self.hex_preview_offset;
        let len = HEX_PREVIEW_BYTES.min(data_len - start);
        if let Some(rom) = self.image.banks[bank].as_ref() {
            ui.label(format!(
                "Bank {bank}: {} | bytes 0x{:05X}-0x{:05X} of {}",
                rom.primary_label(),
                start,
                start + len - 1,
                data_len
            ));

            let hex = format_hex(&rom.data, start, len);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut hex.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                );
            });
        }
    }

    fn selected_rom(&self) -> Option<(usize, &RomBank)> {
        let bank = self.selected_bank?;
        self.image
            .banks
            .get(bank)
            .and_then(Option::as_ref)
            .map(|rom| (bank, rom))
    }

    fn select_bank(&mut self, bank: usize) {
        self.selected_bank = Some(bank);
        self.hex_preview_offset = 0;
        self.hex_preview_visible = self.image.banks.get(bank).is_some_and(Option::is_some);
    }

    fn current_layout(&self) -> RomLayout {
        LAYOUTS[self.selected_layout]
    }

    fn current_chip(&self) -> Chip {
        CHIPS[self.selected_chip]
    }

    fn current_pad_byte(&self) -> u8 {
        parse_pad_byte(&self.pad_text).unwrap_or(self.image.pad_byte)
    }

    fn apply_layout_change(&mut self, layout: RomLayout, selected_layout: usize) {
        let current = self.image.layout;
        if layout == current {
            self.selected_layout = selected_layout;
            return;
        }

        // Same bank size: only the platform label changes, keep the banks.
        if layout.unit_size == current.unit_size {
            self.image.layout = layout;
            self.selected_layout = selected_layout;
            self.dirty = true;
            self.enforce_chip_eligibility();
            self.status = format!("ROM layout changed to {}", layout.name);
            return;
        }

        // No content yet: apply directly.
        if self.image.populated_count() == 0 {
            let _ = self.image.change_layout(layout);
            self.selected_layout = selected_layout;
            self.selected_bank = None;
            self.bank_map_page = 0;
            self.dirty = true;
            self.hex_preview_visible = false;
            self.enforce_chip_eligibility();
            self.status = format!("ROM layout changed to {}", layout.name);
            return;
        }

        // Different bank size with content: confirm. Smaller splits the banks,
        // larger consolidates and spreads them across the bigger ROM.
        let target = layout_size_label(layout.unit_size);
        self.status = if layout.unit_size < current.unit_size {
            format!("Splitting into {target} banks needs confirmation.")
        } else {
            format!("Consolidating into {target} banks needs confirmation.")
        };
        self.pending_layout_change = Some((layout, selected_layout));
    }

    fn commit_layout_change(&mut self, layout: RomLayout, selected_layout: usize) {
        let shrinking = layout.unit_size < self.image.layout.unit_size;
        let target = layout_size_label(layout.unit_size);
        self.image.relayout(layout);
        self.selected_layout = selected_layout;
        self.selected_bank = None;
        self.bank_map_page = 0;
        self.dirty = true;
        self.hex_preview_visible = false;
        self.enforce_chip_eligibility();
        self.status = if shrinking {
            format!("Split content into {target} banks")
        } else {
            format!("Consolidated content into {target} banks")
        };
    }

    fn layout_change_dialog(&mut self, ctx: &egui::Context) {
        let Some((layout, selected_layout)) = self.pending_layout_change else {
            return;
        };
        let new_banks = self.image.chip.bank_count(layout);
        let target = layout_size_label(layout.unit_size);
        let shrinking = layout.unit_size < self.image.layout.unit_size;
        let title = if shrinking {
            "Split content into smaller banks?"
        } else {
            "Consolidate content into larger banks?"
        };
        let (enter, escape) = confirm_keys(ctx);

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                if shrinking {
                    ui.label(format!(
                        "Switching to {target} re-divides the image into {new_banks} smaller banks."
                    ));
                } else {
                    ui.label(format!(
                        "Switching to {target} consolidates the banks and spreads them across the larger ROM ({new_banks} banks)."
                    ));
                }
                ui.label("Existing bank names and sub-slots are not preserved.");
                ui.separator();
                ui.horizontal(|ui| {
                    let action = if shrinking { "Split (ENTER)" } else { "Consolidate (ENTER)" };
                    if ui.button(action).clicked() || enter {
                        self.pending_layout_change = None;
                        self.commit_layout_change(layout, selected_layout);
                    }
                    if ui.button("Cancel (ESC)").clicked() || escape {
                        self.pending_layout_change = None;
                        self.status = "Layout change cancelled".to_owned();
                    }
                });
            });
    }

    /// Whether a chip may be selected for the given layout size. A chip smaller
    /// than one ROM is never eligible. In single-ROM mode without the relaxed
    /// option the chip must match the layout size exactly; otherwise any chip
    /// large enough to hold one ROM qualifies.
    fn chip_is_eligible(&self, chip: Chip, layout_size: usize) -> bool {
        if chip.capacity < layout_size {
            return false;
        }

        if self.multi_rom_image || self.allow_larger_chip {
            true
        } else {
            chip.capacity == layout_size
        }
    }

    /// Switch the target chip to an eligible one when the current selection no
    /// longer fits the layout size (too small, or not an exact match in strict
    /// single-ROM mode). Picks the smallest chip that can hold one ROM, or an
    /// exact match in strict single-ROM mode.
    fn enforce_chip_eligibility(&mut self) {
        let size = self.current_layout().unit_size;
        if self.chip_is_eligible(self.current_chip(), size) {
            return;
        }

        let strict = !self.multi_rom_image && !self.allow_larger_chip;
        let pick = if strict {
            CHIPS.iter().position(|chip| chip.capacity == size)
        } else {
            CHIPS
                .iter()
                .enumerate()
                .filter(|(_, chip)| chip.capacity >= size)
                .min_by_key(|(_, chip)| chip.capacity)
                .map(|(index, _)| index)
        };

        match pick {
            Some(index) => self.apply_chip_change(CHIPS[index], index),
            None => {
                self.status = format!(
                    "No chip can hold the {} ROM size ({} bytes)",
                    self.current_layout().name,
                    size
                );
            }
        }
    }

    fn apply_chip_change(&mut self, chip: Chip, selected_chip: usize) {
        if chip == self.image.chip {
            self.selected_chip = selected_chip;
            return;
        }

        // Shrinking onto a chip that can't hold a populated bank would drop data;
        // ask first. The combo selection is left unchanged until confirmed.
        if self.image.chip_change_drops_data(chip) {
            self.status = format!(
                "Switching to {} would drop data beyond its capacity. Confirm to clip.",
                chip.id
            );
            self.pending_chip_change = Some(PendingChipChange {
                chip,
                selected_chip,
                reenable_multi_rom: false,
            });
            return;
        }

        self.commit_chip_change(chip, selected_chip);
    }

    fn commit_chip_change(&mut self, chip: Chip, selected_chip: usize) {
        self.image.set_chip(chip);
        self.image.pad_byte = self.current_pad_byte();
        self.selected_chip = selected_chip;
        self.selected_bank = None;
        self.bank_map_page = 0;
        self.dirty = true;
        self.hex_preview_visible = false;
        self.status = format!("Target chip changed to {}", chip.id);
    }

    fn chip_change_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_chip_change.as_ref() else {
            return;
        };
        let chip = pending.chip;
        let selected_chip = pending.selected_chip;
        let reenable_multi_rom = pending.reenable_multi_rom;
        let new_count = chip.bank_count(self.image.layout);
        let dropped = self.image.dropped_bank_count(chip);
        let (enter, escape) = confirm_keys(ctx);

        egui::Window::new("Reduce chip size?")
            .collapsible(false)
            .resizable(false)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                ui.label(format!("{} holds only {new_count} bank(s).", chip.id));
                ui.label(format!(
                    "{dropped} populated bank(s) beyond bank {} will be dropped to fit.",
                    new_count.saturating_sub(1)
                ));
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Clip and Change (ENTER)").clicked() || enter {
                        self.pending_chip_change = None;
                        self.commit_chip_change(chip, selected_chip);
                        self.status = format!(
                            "Target chip changed to {} ({dropped} bank(s) dropped)",
                            chip.id
                        );
                    }
                    if ui.button("Cancel (ESC)").clicked() || escape {
                        self.pending_chip_change = None;
                        if reenable_multi_rom {
                            self.multi_rom_image = true;
                        }
                        self.status = "Chip change cancelled".to_owned();
                    }
                });
            });
    }

    fn new_project(&mut self) {
        self.image = ProjectImage::new(self.current_chip(), self.current_layout());
        self.image.pad_byte = self.current_pad_byte();
        self.selected_bank = None;
        self.bank_map_page = 0;
        self.hex_preview_visible = false;
        self.project_active = true;
        self.current_project_path = None;
        self.dirty = true;
        self.status = "New project created. Save to choose a file.".to_owned();
    }

    fn open_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ROM Builder project", &["romproj"])
            .add_filter("Binary images", &["bin", "rom"])
            .pick_file()
        {
            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("romproj"))
            {
                match project::load(&path) {
                    Ok(loaded) => {
                        self.image = loaded.image;
                        self.apply_settings(loaded.settings);
                        self.sync_selection_from_image();
                        self.selected_bank = None;
                        self.bank_map_page = 0;
                        self.hex_preview_visible = false;
                        self.project_active = true;
                        self.current_project_path = Some(path.clone());
                        self.dirty = false;
                        self.status = format!("Opened project {}", path.display());
                    }
                    Err(message) => {
                        self.status = message;
                    }
                }
            } else {
                self.import_raw_image(&path);
            }
        }
    }

    fn apply_settings(&mut self, settings: ProjectSettings) {
        self.remove_policy = settings.remove_policy;
        self.multi_rom_image = settings.multi_rom_image;
        self.allow_larger_chip = settings.allow_larger_chip;
    }

    fn project_settings(&self) -> ProjectSettings {
        ProjectSettings {
            remove_policy: self.remove_policy,
            multi_rom_image: self.multi_rom_image,
            allow_larger_chip: self.allow_larger_chip,
            layout_by_size: true,
        }
    }

    fn sync_selection_from_image(&mut self) {
        self.selected_chip = CHIPS
            .iter()
            .position(|chip| *chip == self.image.chip)
            .unwrap_or(0);
        self.selected_layout = LAYOUTS
            .iter()
            .position(|layout| *layout == self.image.layout)
            .unwrap_or(0);
        self.pad_text = format!("{:02X}", self.image.pad_byte);
    }

    fn save_project(&mut self) {
        match self.current_project_path.clone() {
            Some(path) => {
                self.write_project(&path);
            }
            None => self.save_project_as(),
        }
    }

    fn save_project_as(&mut self) {
        let suggested = project::suggested_name(&self.image);
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ROM Builder project", &["romproj"])
            .set_file_name(&suggested)
            .save_file()
        {
            if self.write_project(&path) {
                self.current_project_path = Some(path);
            }
        }
    }

    fn write_project(&mut self, path: &Path) -> bool {
        let settings = self.project_settings();
        match project::save(path, &self.image, &settings) {
            Ok(()) => {
                self.dirty = false;
                self.status = format!("Saved project to {}", path.display());
                true
            }
            Err(message) => {
                self.status = message;
                false
            }
        }
    }

    fn platform_picker_dialog(&mut self, ctx: &egui::Context) {
        let (_enter, escape) = confirm_keys(ctx);
        if escape {
            self.show_platform_picker = false;
            return;
        }

        let platforms = LAYOUTS
            .iter()
            .enumerate()
            .filter(|(_, layout)| !layout.id.starts_with("rom-"))
            .map(|(index, layout)| (index, *layout))
            .collect::<Vec<_>>();

        egui::Window::new("Platform ROMs")
            .collapsible(false)
            .resizable(false)
            .default_size([420.0, 320.0])
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                egui::TopBottomPanel::bottom("platform_picker_cancel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancel (ESC)").clicked() {
                                self.show_platform_picker = false;
                            }
                        });
                    });

                ui.add(
                    egui::TextEdit::singleline(&mut self.platform_filter)
                        .hint_text("Filter platforms")
                        .desired_width(f32::INFINITY),
                );
                let needle = self.platform_filter.trim().to_lowercase();
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for (index, layout) in platforms {
                                let label = layout_size_label(layout.unit_size);
                                if !needle.is_empty()
                                    && !format!("{} {label}", layout.name.to_lowercase())
                                        .contains(&needle)
                                {
                                    continue;
                                }
                                if ui.button(format!("{} ({label})", layout.name)).clicked() {
                                    self.apply_layout_change(layout, index);
                                    self.show_platform_picker = false;
                                }
                            }
                        });
                    });
            });
    }

    fn ic_picker_dialog(&mut self, ctx: &egui::Context) {
        let (_enter, escape) = confirm_keys(ctx);
        if escape {
            self.show_ic_picker = false;
            return;
        }

        let layout_size = self.current_layout().unit_size;
        let mut chips = (0..CHIPS.len())
            .filter(|&index| self.chip_is_eligible(CHIPS[index], layout_size))
            .map(|index| (index, CHIPS[index]))
            .collect::<Vec<_>>();
        chips.sort_by_key(|(_, chip)| chip.capacity);

        egui::Window::new("Target ICs")
            .collapsible(false)
            .resizable(false)
            .default_size([420.0, 320.0])
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                egui::TopBottomPanel::bottom("ic_picker_cancel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancel (ESC)").clicked() {
                                self.show_ic_picker = false;
                            }
                        });
                    });

                ui.add(
                    egui::TextEdit::singleline(&mut self.ic_filter)
                        .hint_text("Filter ICs")
                        .desired_width(f32::INFINITY),
                );
                let needle = self.ic_filter.trim().to_lowercase();
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for (index, chip) in chips {
                                if !needle.is_empty()
                                    && !chip.name.to_lowercase().contains(&needle)
                                    && !chip.id.to_lowercase().contains(&needle)
                                {
                                    continue;
                                }
                                if ui.button(chip.name).clicked() {
                                    self.apply_chip_change(chip, index);
                                    self.show_ic_picker = false;
                                }
                            }
                        });
                    });
            });
    }

    fn close_confirm_dialog(&mut self, ctx: &egui::Context) {
        let (_enter, escape) = confirm_keys(ctx);
        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                ui.label("This project has unsaved changes.");
                ui.label("Leave without saving?");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Leave without saving").clicked() {
                        self.pending_close = false;
                        self.allow_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Cancel (ESC)").clicked() || escape {
                        self.pending_close = false;
                    }
                });
            });
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_about;
        let (_enter, escape) = confirm_keys(ctx);
        let mut close = escape;

        if self.about_logo.is_none() {
            if let Ok(icon) = eframe::icon_data::from_png_bytes(APP_ICON_PNG) {
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [icon.width as usize, icon.height as usize],
                    &icon.rgba,
                );
                self.about_logo =
                    Some(ctx.load_texture("about_logo", image, egui::TextureOptions::default()));
            }
        }

        egui::Window::new("About")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if let Some(logo) = &self.about_logo {
                        ui.image(egui::load::SizedTexture::new(
                            logo.id(),
                            egui::vec2(72.0, 72.0),
                        ));
                        ui.add_space(4.0);
                    }
                    ui.label(egui::RichText::new("ROM Builder").heading().strong());
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.label(egui::RichText::new("Author: ArleyJR").strong());
                    ui.add_space(8.0);
                    ui.label("Built with the eframe / egui immediate-mode GUI framework (Rust).");
                    ui.label(
                        "Native file dialogs via rfd; project files via the zip and serde_json crates.",
                    );
                    ui.add_space(8.0);
                    ui.label(
                        "egui, eframe and the other crates are dual-licensed under MIT or Apache-2.0.",
                    );
                    ui.label("ROM Builder is distributed under the MIT license.");
                    ui.add_space(8.0);
                    ui.hyperlink_to(
                        "Puzzle icons created by Freepik - Flaticon",
                        "https://www.flaticon.com/free-icons/puzzle",
                    );
                    ui.add_space(16.0);
                    if ui.button("Close (ESC)").clicked() {
                        close = true;
                    }
                });
            });
        self.show_about = open && !close;
    }

    fn import_raw_image(&mut self, path: &Path) {
        let matching_chips = match matching_chip_indices_for_file(path) {
            Ok(matches) => matches,
            Err(message) => {
                self.status = message;
                return;
            }
        };

        if matching_chips.is_empty() {
            self.status = format!("{} does not match any supported chip size", path.display());
            return;
        }

        let previous_chip = self.current_chip();
        let chip_index = if matching_chips.contains(&self.selected_chip) {
            self.selected_chip
        } else {
            matching_chips[0]
        };
        self.selected_chip = chip_index;

        match ProjectImage::import_image(
            self.current_chip(),
            self.current_layout(),
            path,
            self.current_pad_byte(),
        ) {
            Ok((image, warning)) => {
                self.image = image;
                self.selected_bank = None;
                self.bank_map_page = 0;
                self.hex_preview_visible = false;
                self.project_active = true;
                self.current_project_path = None;
                self.dirty = true;
                let alternatives = matching_chips
                    .iter()
                    .copied()
                    .filter(|index| *index != chip_index)
                    .map(|index| CHIPS[index].id)
                    .collect::<Vec<_>>();
                let suggestion = if previous_chip != self.current_chip() {
                    format!(" Auto-selected {} from file size.", self.current_chip().id)
                } else {
                    format!(" {} fits the file size.", self.current_chip().id)
                };
                let also_fits = if alternatives.is_empty() {
                    String::new()
                } else {
                    format!(" Also fits: {}.", alternatives.join(", "))
                };
                let warning = warning
                    .map(|message| format!(" WARNING: {message}"))
                    .unwrap_or_default();
                self.status = format!(
                    "Imported {}.{suggestion}{also_fits}{warning}",
                    path.display()
                );
            }
            Err(message) => {
                self.status = message;
            }
        }
    }

    fn add_roms(&mut self) {
        if let Some(paths) = rfd::FileDialog::new()
            .add_filter("ROM files", &["rom", "bin"])
            .pick_files()
        {
            let unit = self.current_layout().unit_size;
            let mut added = 0usize;
            for path in paths {
                let data = match fs::read(&path) {
                    Ok(data) => data,
                    Err(err) => {
                        self.status = format!("Could not read {}: {err}", path.display());
                        return;
                    }
                };

                if data.len() == unit {
                    match RomBank::from_slice(self.current_layout(), file_label(&path), &data)
                        .and_then(|rom| self.image.add_rom(rom))
                    {
                        Ok(bank) => {
                            added += 1;
                            self.select_bank(bank);
                        }
                        Err(message) => {
                            self.status = message;
                            return;
                        }
                    }
                } else if data.len() > unit {
                    // Larger than a bank: ask the user how to place it. Stop here
                    // so the choice applies before any remaining files are added.
                    match self.image.first_empty_bank() {
                        Some(bank) => {
                            self.pending_oversized_add = Some(PendingOversizedAdd {
                                path: path.clone(),
                                label: file_label(&path),
                                data,
                                bank,
                            });
                            self.status = format!(
                                "{} is larger than the {} bank ({added} added first). Choose how to add it.",
                                file_label(&path),
                                self.current_layout().name
                            );
                            return;
                        }
                        None => {
                            self.status =
                                format!("No empty banks remain for {}", self.current_chip().id);
                            return;
                        }
                    }
                } else {
                    // Smaller than a bank: pad the remainder and add as a
                    // partial bank (the bank list shows the free remainder).
                    match RomBank::from_partial(
                        self.current_layout(),
                        file_label(&path),
                        &data,
                        self.image.pad_byte,
                    )
                    .and_then(|rom| self.image.add_rom(rom))
                    {
                        Ok(bank) => {
                            added += 1;
                            self.select_bank(bank);
                        }
                        Err(message) => {
                            self.status = message;
                            return;
                        }
                    }
                }
            }

            if added > 0 {
                self.dirty = true;
            }
            self.status = format!("Added {added} ROM file(s)");
        }
    }

    /// Loads a ROM file into a bank's trailing free space, creating a new slot.
    fn load_into_free(&mut self, bank: usize) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ROM files", &["rom", "bin"])
            .pick_file()
        {
            let data = match fs::read(&path) {
                Ok(data) => data,
                Err(err) => {
                    self.status = format!("Could not read {}: {err}", path.display());
                    return;
                }
            };

            let name = file_label(&path);
            let Some(rom) = self.image.banks[bank].as_mut() else {
                return;
            };
            match rom.add_into_free(name.clone(), Some(path), &data) {
                Ok(()) => {
                    self.dirty = true;
                    self.select_bank(bank);
                    self.status = format!("Added {name} into free space of bank {bank}");
                }
                Err(message) => {
                    self.status = message;
                }
            }
        }
    }

    /// Loads a ROM file into an existing slot's fixed region (used by a
    /// sub-slot Replace, and to refill a blanked sub-slot).
    fn load_into_slot(&mut self, bank: usize, slot: usize) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ROM files", &["rom", "bin"])
            .pick_file()
        {
            let data = match fs::read(&path) {
                Ok(data) => data,
                Err(err) => {
                    self.status = format!("Could not read {}: {err}", path.display());
                    return;
                }
            };

            let name = file_label(&path);
            let pad_byte = self.image.pad_byte;
            let Some(rom) = self.image.banks[bank].as_mut() else {
                return;
            };
            if slot >= rom.slots.len() {
                return;
            }
            let region = rom.slots[slot].len;
            if data.is_empty() {
                self.status = "ROM is empty".to_owned();
                return;
            }
            if data.len() > region {
                self.status = format!(
                    "{name} is {} bytes but slot {bank}.{slot} holds {region} bytes",
                    data.len()
                );
                return;
            }

            let start = rom.slot_start(slot);
            rom.data[start..start + data.len()].copy_from_slice(&data);
            for byte in &mut rom.data[start + data.len()..start + region] {
                *byte = pad_byte;
            }
            rom.slots[slot].label = name.clone();
            rom.slots[slot].source_path = Some(path);
            rom.slots[slot].blank = false;
            self.dirty = true;
            self.select_bank(bank);
            self.status = format!("Loaded {name} into slot {bank}.{slot}");
        }
    }

    fn extract_slot(&mut self, bank: usize, slot: usize) {
        let default_name = self.image.banks[bank]
            .as_ref()
            .and_then(|rom| rom.slots.get(slot))
            .map(|s| extract_file_name(&s.label))
            .unwrap_or_else(|| {
                format!(
                    "{}_bank_{bank}_slot_{slot}.bin",
                    self.image.chip.id.to_lowercase()
                )
            });
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .save_file()
        {
            let bytes = match self.image.banks[bank].as_ref() {
                Some(rom) if slot < rom.slots.len() => {
                    let start = rom.slot_start(slot);
                    rom.data[start..start + rom.slots[slot].len].to_vec()
                }
                _ => return,
            };
            match write_file(&path, &bytes) {
                Ok(()) => {
                    self.status = format!("Extracted slot {bank}.{slot} to {}", path.display());
                }
                Err(message) => {
                    self.status = message;
                }
            }
        }
    }

    /// Loads a ROM file into a specific bank using the same flow as Add ROM
    /// (exact size fits directly; a larger file opens the Cancel/Trim/Spread
    /// dialog targeting this bank; a smaller file is rejected).
    fn add_rom_into_bank(&mut self, bank: usize) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ROM files", &["rom", "bin"])
            .pick_file()
        {
            let data = match fs::read(&path) {
                Ok(data) => data,
                Err(err) => {
                    self.status = format!("Could not read {}: {err}", path.display());
                    return;
                }
            };

            let unit = self.current_layout().unit_size;
            let name = file_label(&path);

            if data.len() == unit {
                match RomBank::from_slice(self.current_layout(), name.clone(), &data)
                    .and_then(|rom| self.image.set_bank(bank, rom))
                {
                    Ok(()) => {
                        self.dirty = true;
                        self.select_bank(bank);
                        self.status = format!("Added {name} to bank {bank}");
                    }
                    Err(message) => {
                        self.status = message;
                    }
                }
            } else if data.len() > unit {
                self.status = format!(
                    "{name} is larger than the {} bank. Choose how to add it.",
                    self.current_layout().name
                );
                self.pending_oversized_add = Some(PendingOversizedAdd {
                    path,
                    label: name,
                    data,
                    bank,
                });
            } else {
                match RomBank::from_partial(
                    self.current_layout(),
                    name.clone(),
                    &data,
                    self.image.pad_byte,
                )
                .and_then(|rom| self.image.set_bank(bank, rom))
                {
                    Ok(()) => {
                        self.dirty = true;
                        self.select_bank(bank);
                        self.status = format!("Added {name} to bank {bank} (remainder padded)");
                    }
                    Err(message) => {
                        self.status = message;
                    }
                }
            }
        }
    }

    fn confirm_trim_add(&mut self) {
        if let Some(pending) = self.pending_oversized_add.take() {
            let unit = self.current_layout().unit_size;
            let bank = pending.bank;
            let trimmed = &pending.data[..unit.min(pending.data.len())];
            match RomBank::from_slice(self.current_layout(), pending.label, trimmed)
                .and_then(|rom| self.image.set_bank(bank, rom))
            {
                Ok(()) => {
                    self.dirty = true;
                    self.select_bank(bank);
                    self.status = format!(
                        "Trimmed {} to {} bytes and added to bank {bank}",
                        file_label(&pending.path),
                        unit
                    );
                }
                Err(message) => {
                    self.status = message;
                }
            }
        }
    }

    fn confirm_spread_add(&mut self) {
        if let Some(pending) = self.pending_oversized_add.take() {
            let unit = self.current_layout().unit_size;
            let bank_count = self.image.chip.bank_count(self.image.layout);

            let start = pending.bank;
            if start >= bank_count {
                self.status = "Target bank is no longer valid".to_owned();
                return;
            }

            // Fill from the target bank onwards, overwriting any occupied banks
            // in range. If the file is larger than the remaining banks, spread
            // what fits and drop the rest.
            let needed = pending.data.len().div_ceil(unit);
            let available = bank_count - start;
            let to_write = needed.min(available);

            for chunk_index in 0..to_write {
                let begin = chunk_index * unit;
                let end = ((chunk_index + 1) * unit).min(pending.data.len());
                let mut chunk = pending.data[begin..end].to_vec();
                chunk.resize(unit, self.image.pad_byte);
                let label = if chunk_index == 0 {
                    pending.label.clone()
                } else {
                    format!("{} (part {})", pending.label, chunk_index + 1)
                };
                if let Err(message) = RomBank::from_slice(self.current_layout(), label, &chunk)
                    .and_then(|rom| self.image.set_bank(start + chunk_index, rom))
                {
                    self.status = message;
                    return;
                }
            }

            self.dirty = true;
            self.select_bank(start);
            self.status = if to_write < needed {
                format!(
                    "Spread {} across banks {start}-{} ({to_write} of {needed} banks; remainder dropped)",
                    pending.label,
                    start + to_write - 1
                )
            } else {
                format!(
                    "Spread {} across banks {start}-{}",
                    pending.label,
                    start + to_write - 1
                )
            };
        }
    }

    fn oversized_add_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.pending_oversized_add else {
            return;
        };
        let name = file_label(&pending.path);
        let size = pending.data.len();
        let start = pending.bank;

        let unit = self.current_layout().unit_size;
        let needed = size.div_ceil(unit);
        let bank_count = self.image.chip.bank_count(self.image.layout);
        let available = bank_count - start;
        let to_write = needed.min(available);
        let (_enter, escape) = confirm_keys(ctx);

        egui::Window::new("ROM larger than bank")
            .collapsible(false)
            .resizable(false)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                ui.label(format!("{name} is {size} bytes."));
                ui.label(format!(
                    "{} bank size is {unit} bytes.",
                    self.current_layout().name
                ));
                ui.label(format!("Trim keeps the first {unit} bytes in bank {start}."));
                if to_write < needed {
                    ui.label(format!(
                        "Spread fills banks {start}-{} (partial: {to_write} of {needed} banks fit, rest dropped).",
                        start + to_write - 1
                    ));
                } else {
                    ui.label(format!("Spread fills banks {start}-{}.", start + to_write - 1));
                }
                ui.label("Spread overwrites any occupied banks in range.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Cancel (ESC)").clicked() || escape {
                        self.pending_oversized_add = None;
                        self.status = "Add cancelled".to_owned();
                    }
                    if ui.button("Trim to bank").clicked() {
                        self.confirm_trim_add();
                    }
                    if ui.button("Spread across banks").clicked() {
                        self.confirm_spread_add();
                    }
                });
            });
    }

    fn replace_bank(&mut self, bank: usize) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ROM files", &["rom", "bin"])
            .pick_file()
        {
            match fs::read(&path) {
                Ok(data) => {
                    if data.len() == self.current_layout().unit_size {
                        match RomBank::from_path(self.current_layout(), &path)
                            .and_then(|rom| self.image.set_bank(bank, rom))
                        {
                            Ok(()) => {
                                self.dirty = true;
                                self.select_bank(bank);
                                self.status =
                                    format!("Replaced bank {bank} with {}", path.display());
                            }
                            Err(message) => {
                                self.status = message;
                            }
                        }
                    } else if data.len() > self.current_layout().unit_size {
                        let label = path
                            .file_stem()
                            .and_then(|name| name.to_str())
                            .or_else(|| path.file_name().and_then(|name| name.to_str()))
                            .unwrap_or("ROM")
                            .to_owned();
                        self.pending_trim_replace = Some(PendingTrimReplace {
                            bank,
                            path: path.clone(),
                            label,
                            data,
                        });
                        self.status = format!(
                            "{} is too large for {}. Confirm trim to replace bank {bank}.",
                            path.display(),
                            self.current_layout().name
                        );
                    } else {
                        self.status = format!(
                            "{} is {} bytes, but {} requires {} bytes. Replace blocked.",
                            path.display(),
                            data.len(),
                            self.current_layout().name,
                            self.current_layout().unit_size
                        );
                    }
                }
                Err(err) => {
                    self.status = format!("Could not read {}: {err}", path.display());
                }
            }
        }
    }

    fn extract_bank(&mut self, bank: usize) {
        let default_name = self.image.banks[bank]
            .as_ref()
            .and_then(|rom| rom.slots.first())
            .map(|slot| extract_file_name(&slot.label))
            .unwrap_or_else(|| format!("{}_bank_{bank}.bin", self.image.chip.id.to_lowercase()));
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .save_file()
        {
            if let Some(rom) = &self.image.banks[bank] {
                match write_file(&path, &rom.data) {
                    Ok(()) => {
                        self.status = format!("Extracted bank {bank} to {}", path.display());
                    }
                    Err(message) => {
                        self.status = message;
                    }
                }
            }
        }
    }

    fn remove_bank(&mut self, bank: usize) {
        match self.image.remove_bank(bank, self.remove_policy) {
            Ok(()) => {
                self.dirty = true;
                self.selected_bank =
                    Some(bank.min(self.image.chip.bank_count(self.image.layout) - 1));
                self.hex_preview_visible = self.selected_rom().is_some();
                self.status = format!("Removed bank {bank} using {}", self.remove_policy.label());
            }
            Err(message) => {
                self.status = message;
            }
        }
    }

    fn remove_bank_slot(&mut self, bank: usize, slot: usize) {
        let pad_byte = self.image.pad_byte;
        let policy = self.remove_policy;
        let became_unused = if let Some(rom) = self.image.banks[bank].as_mut() {
            if slot >= rom.slots.len() {
                return;
            }
            rom.remove_slot(slot, policy, pad_byte);
            rom.collapse_if_unused()
        } else {
            return;
        };
        if became_unused {
            self.image.banks[bank] = None;
        }
        self.dirty = true;
        self.select_bank(bank);
        self.status = format!("Removed slot {bank}.{slot} using {}", policy.label());
    }

    fn confirm_trim_replace(&mut self) {
        if let Some(pending) = self.pending_trim_replace.take() {
            let trimmed = &pending.data[..self.current_layout().unit_size];
            match RomBank::from_slice(self.current_layout(), pending.label, trimmed)
                .and_then(|rom| self.image.set_bank(pending.bank, rom))
            {
                Ok(()) => {
                    self.dirty = true;
                    self.select_bank(pending.bank);
                    self.status = format!(
                        "Trimmed {} to {} bytes and replaced bank {}",
                        pending.path.display(),
                        self.current_layout().unit_size,
                        pending.bank
                    );
                }
                Err(message) => {
                    self.status = message;
                }
            }
        }
    }

    fn trim_replace_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.pending_trim_replace else {
            return;
        };
        let bank = pending.bank;
        let path = pending.path.display().to_string();
        let size = pending.data.len();
        let required = self.current_layout().unit_size;
        let (enter, escape) = confirm_keys(ctx);

        egui::Window::new("Trim ROM to fit bank?")
            .collapsible(false)
            .resizable(false)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                ui.label(format!("{path} is {size} bytes."));
                ui.label(format!(
                    "{} bank size is {required} bytes.",
                    self.current_layout().name
                ));
                ui.label(format!(
                    "Trim the file to the first {required} bytes and replace bank {bank}?"
                ));
                ui.horizontal(|ui| {
                    if ui.button("Trim and Replace (ENTER)").clicked() || enter {
                        self.confirm_trim_replace();
                    }
                    if ui.button("Cancel (ESC)").clicked() || escape {
                        self.pending_trim_replace = None;
                        self.status = "Replace cancelled".to_owned();
                    }
                });
            });
    }

    fn remove_confirmation_dialog(&mut self, ctx: &egui::Context) {
        let Some(bank) = self.pending_remove_bank else {
            return;
        };
        let (enter, escape) = confirm_keys(ctx);

        egui::Window::new("Confirm remove")
            .collapsible(false)
            .resizable(false)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                ui.label(format!(
                    "Remove bank {bank} using {}?",
                    self.remove_policy.label()
                ));
                ui.horizontal(|ui| {
                    if ui.button("Remove (ENTER)").clicked() || enter {
                        self.pending_remove_bank = None;
                        self.remove_bank(bank);
                    }
                    if ui.button("Cancel (ESC)").clicked() || escape {
                        self.pending_remove_bank = None;
                        self.status = "Remove cancelled".to_owned();
                    }
                });
            });
    }

    fn remove_slot_confirmation_dialog(&mut self, ctx: &egui::Context) {
        let Some((bank, slot)) = self.pending_remove_slot else {
            return;
        };
        let (enter, escape) = confirm_keys(ctx);

        egui::Window::new("Confirm remove slot")
            .collapsible(false)
            .resizable(false)
            .frame(dialog_frame(ctx))
            .show(ctx, |ui| {
                ui.label(format!(
                    "Remove slot {bank}.{slot}? Later slots shift down."
                ));
                ui.horizontal(|ui| {
                    if ui.button("Remove (ENTER)").clicked() || enter {
                        self.pending_remove_slot = None;
                        self.remove_bank_slot(bank, slot);
                    }
                    if ui.button("Cancel (ESC)").clicked() || escape {
                        self.pending_remove_slot = None;
                        self.status = "Remove cancelled".to_owned();
                    }
                });
            });
    }

    fn name_suggestions_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.pending_name_suggestions else {
            return;
        };
        let bank = pending.bank;
        let slot = pending.slot;
        let suggestions = pending.suggestions.clone();
        let (_enter, escape) = confirm_keys(ctx);
        if escape {
            self.pending_name_suggestions = None;
            self.status = "Name suggestion cancelled".to_owned();
            return;
        }

        egui::Window::new("Name suggestions")
            .collapsible(false)
            .resizable(false)
            .default_size([420.0, 260.0])
            .show(ctx, |ui| {
                egui::TopBottomPanel::bottom("name_suggestions_cancel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancel (ESC)").clicked() {
                                self.pending_name_suggestions = None;
                                self.status = "Name suggestion cancelled".to_owned();
                            }
                        });
                    });

                ui.label(format!("Pick a suggested name for bank {bank}."));
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for suggestion in suggestions {
                                if ui.button(&suggestion).clicked() {
                                    if let Some(slot) = self
                                        .image
                                        .banks
                                        .get_mut(bank)
                                        .and_then(Option::as_mut)
                                        .and_then(|rom| rom.slots.get_mut(slot))
                                    {
                                        slot.label = suggestion;
                                        trim_to_char_limit(&mut slot.label, ROM_NAME_MAX_CHARS);
                                        self.dirty = true;
                                        self.status = format!("Updated name for bank {bank}");
                                    }
                                    self.pending_name_suggestions = None;
                                }
                            }
                        });
                    });
            });
    }

    fn export_image(&mut self) {
        let default_name = format!("{}_image.bin", self.image.chip.id.to_lowercase());
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .save_file()
        {
            let output = self.image.export_bytes();
            let metadata_path = metadata_path_for(&path);
            let binary_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("exported image");
            let metadata = self.image.metadata_text(binary_name);

            match write_file(&path, &output)
                .and_then(|()| write_text_file(&metadata_path, &metadata))
            {
                Ok(()) => {
                    self.status = format!(
                        "Exported {} bytes and metadata to {}",
                        output.len(),
                        metadata_path.display()
                    );
                }
                Err(message) => {
                    self.status = message;
                }
            }
        }
    }
}

fn kib_label(bytes: usize) -> String {
    if bytes.is_multiple_of(1024) {
        format!("{} KiB", bytes / 1024)
    } else {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    }
}

fn dialog_frame(ctx: &egui::Context) -> egui::Frame {
    egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::same(20.0))
}

fn confirm_keys(ctx: &egui::Context) -> (bool, bool) {
    ctx.input(|i| {
        (
            i.key_pressed(egui::Key::Enter),
            i.key_pressed(egui::Key::Escape),
        )
    })
}

/// Builds an extract filename from a slot's name: keeps a `.rom`/`.bin`
/// extension if present, otherwise appends `.bin`.
fn extract_file_name(label: &str) -> String {
    let base = label.trim();
    let base = if base.is_empty() { "rom" } else { base };
    let lower = base.to_ascii_lowercase();
    if lower.ends_with(".rom") || lower.ends_with(".bin") {
        base.to_owned()
    } else {
        format!("{base}.bin")
    }
}

fn unused_size_label(bytes: usize) -> String {
    if bytes.is_multiple_of(1024) {
        format!("{}kb", bytes / 1024)
    } else {
        format!("{bytes}b")
    }
}

fn file_label(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .or_else(|| path.file_name().and_then(|name| name.to_str()))
        .unwrap_or("ROM")
        .to_owned()
}

fn layout_size_label(unit_size: usize) -> String {
    if unit_size.is_multiple_of(1024 * 1024) {
        format!("{}M", unit_size / (1024 * 1024))
    } else if unit_size.is_multiple_of(1024) {
        format!("{}k", unit_size / 1024)
    } else {
        format!("{unit_size}B")
    }
}

fn parse_pad_byte(text: &str) -> Result<u8, String> {
    let trimmed = text.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    u8::from_str_radix(hex, 16)
        .map_err(|_| "Pad byte must be hexadecimal, for example FF or 0xFF".to_owned())
}

fn matching_chip_indices_for_file(path: &std::path::Path) -> Result<Vec<usize>, String> {
    let size = fs::metadata(path)
        .map_err(|err| format!("Could not inspect {}: {err}", path.display()))?
        .len() as usize;

    Ok(CHIPS
        .iter()
        .enumerate()
        .filter_map(|(index, chip)| (chip.capacity == size).then_some(index))
        .collect())
}

fn trim_to_char_limit(value: &mut String, max_chars: usize) {
    if let Some((byte_index, _)) = value.char_indices().nth(max_chars) {
        value.truncate(byte_index);
    }
}

fn suggest_rom_names(data: &[u8]) -> Vec<String> {
    let mut scored = Vec::new();

    for (offset, text) in printable_strings(data) {
        for candidate in name_candidates_from_text(&text) {
            // Phrase quality drives the ranking: multi-word, alphabetic-rich
            // candidates score highest. Position only nudges ties (capped at
            // 100) so that neighbouring ROMs sharing a leading loader/header
            // stub no longer collapse to the same suggestion list.
            let word_bonus = (candidate.split_whitespace().count() as isize) * 200;
            let alpha_bonus = candidate
                .chars()
                .filter(|char| char.is_alphabetic())
                .count() as isize;
            let position_bonus = if data.is_empty() {
                0
            } else {
                100 - ((offset.min(data.len()) * 100) / data.len()) as isize
            };
            scored.push((word_bonus + alpha_bonus + position_bonus, offset, candidate));
        }
    }

    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.len().cmp(&right.2.len()))
    });

    let mut suggestions = Vec::new();
    for (_, _, candidate) in scored {
        if suggestions
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&candidate))
        {
            continue;
        }

        suggestions.push(candidate);
        if suggestions.len() == ROM_NAME_SUGGESTION_LIMIT {
            break;
        }
    }

    suggestions
}

fn printable_strings(data: &[u8]) -> Vec<(usize, String)> {
    let mut strings = Vec::new();
    let mut start = None;
    let mut bytes = Vec::new();

    for (index, byte) in data.iter().copied().enumerate() {
        if byte.is_ascii_graphic() || byte == b' ' {
            if start.is_none() {
                start = Some(index);
            }
            bytes.push(byte);
        } else if bytes.len() >= 4 {
            strings.push((
                start.unwrap_or(index),
                String::from_utf8_lossy(&bytes).to_string(),
            ));
            start = None;
            bytes.clear();
        } else {
            start = None;
            bytes.clear();
        }
    }

    if bytes.len() >= 4 {
        strings.push((
            start.unwrap_or(data.len()),
            String::from_utf8_lossy(&bytes).to_string(),
        ));
    }

    strings
}

fn name_candidates_from_text(text: &str) -> Vec<String> {
    let words = text
        .split(|char: char| !char.is_ascii_alphanumeric())
        .filter(|word| word.len() >= 3)
        .filter(|word| word.chars().any(|char| char.is_ascii_alphabetic()))
        .filter(|word| !is_low_value_word(word))
        .take(12)
        .map(title_word)
        .collect::<Vec<_>>();

    let mut candidates = Vec::new();

    for window_size in (1..=3).rev() {
        for window in words.windows(window_size) {
            let candidate = window.join(" ");
            if candidate.chars().count() <= ROM_NAME_MAX_CHARS {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

fn title_word(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => format!(
            "{}{}",
            first.to_ascii_uppercase(),
            chars.collect::<String>()
        ),
        None => String::new(),
    }
}

fn is_low_value_word(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "the"
            | "and"
            | "for"
            | "with"
            | "from"
            | "this"
            | "that"
            | "copyright"
            | "error"
            | "bytes"
            | "basic"
            | "rom"
            | "version"
            | "release"
            | "reserved"
            | "unknown"
    )
}

fn format_hex(data: &[u8], start: usize, len: usize) -> String {
    let end = (start + len).min(data.len());
    let mut output = String::new();

    for (line_index, chunk) in data[start..end].chunks(16).enumerate() {
        let offset = start + line_index * 16;
        output.push_str(&format!("{offset:08X}  "));

        for index in 0..16 {
            if let Some(byte) = chunk.get(index) {
                output.push_str(&format!("{byte:02X} "));
            } else {
                output.push_str("   ");
            }

            if index == 7 {
                output.push(' ');
            }
        }

        output.push(' ');
        for byte in chunk {
            let ascii = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            output.push(ascii);
        }
        output.push('\n');
    }

    output
}
