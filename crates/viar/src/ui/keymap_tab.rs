use std::collections::HashMap;

use eframe::egui;
use tracing::{
    info,
    warn,
};
use via_protocol::{
    Keycode,
    ViaProtocol,
};

use crate::{
    types::{
        KeyChange,
        StatusMessage,
        ViarApp,
    },
    util::{
        CategoryStyle, is_disconnect_error, themed_tab
    },
};

impl ViarApp {
    pub fn render_keymap_tab(&mut self, ui: &mut egui::Ui) {
        let Some(data) = &mut self.keymap_data else {
            ui.label("No keymap data loaded.");
            return;
        };

        // Layer tabs
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            for layer in 0..data.layer_count as usize {
                let label = format!("Layer {layer}");
                let selected = data.selected_layer == layer;
                if themed_tab(ui, selected, &label, &self.theme).clicked() {
                    data.selected_layer = layer;
                    data.selected_key = None;
                }
            }
        });

        ui.separator();

        // Keyboard visualization area
        let available = ui.available_size();
        let layout = &data.layout;

        let layout_w = layout.width();
        let layout_h = layout.height();
        let padding = 40.0;
        let key_size = ((available.x - padding * 2.0) / layout_w)
            .min((available.y - padding * 2.0) / layout_h)
            .clamp(32.0, 64.0);
        let gap = 8.0;

        let total_w = layout_w * key_size;
        let total_h = layout_h * key_size;
        let x_offset = (available.x - total_w) / 2.0;
        let y_offset = (available.y - total_h) / 2.0;

        let origin = ui.min_rect().min + egui::vec2(x_offset, y_offset);
        let painter = ui.painter();

        let layer_idx = data.selected_layer;
        let mut clicked_key = None;
        let mut selected_key_rect: Option<egui::Rect> = None;

        // Build combo info: map each keycode to the combos it participates in.
        // Each combo gets a unique color derived from its index via hue rotation.
        struct ComboInfo {
            color:       egui::Color32,
            description: String,
        }
        fn combo_color(index: usize, total: usize) -> egui::Color32 {
            let hue = if total <= 1 {
                0.35 // green
            } else {
                (index as f64) / (total as f64) // spread evenly across 0..1
            };
            // HSL to RGB with S=0.5, L=0.55 for muted but distinct pastels
            let s = 0.5_f64;
            let l = 0.55_f64;
            let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
            let h_prime = hue * 6.0;
            let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
            let (r1, g1, b1) = match h_prime as u32 {
                0 => (c, x, 0.0),
                1 => (x, c, 0.0),
                2 => (0.0, c, x),
                3 => (0.0, x, c),
                4 => (x, 0.0, c),
                _ => (c, 0.0, x),
            };
            let m = l - c / 2.0;
            egui::Color32::from_rgb(
                ((r1 + m) * 255.0) as u8,
                ((g1 + m) * 255.0) as u8,
                ((b1 + m) * 255.0) as u8,
            )
        }
        let combo_map: HashMap<u16, Vec<ComboInfo>> = self
            .dynamic_data
            .as_ref()
            .map(|d| {
                let active_combos: Vec<_> = d
                    .combos
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.input.iter().any(|&k| k != 0))
                    .collect();
                let total = active_combos.len();
                let mut map: HashMap<u16, Vec<ComboInfo>> = HashMap::new();
                for (color_idx, (combo_idx, combo)) in active_combos.iter().enumerate() {
                    let active_inputs: Vec<u16> =
                        combo.input.iter().copied().filter(|&k| k != 0).collect();
                    let color = combo_color(color_idx, total);
                    let input_names: Vec<String> =
                        active_inputs.iter().map(|&kc| Keycode(kc).name()).collect();
                    let output_name = Keycode(combo.output).name();
                    let desc = format!(
                        "Combo {}: {} -> {}",
                        combo_idx,
                        input_names.join(" + "),
                        output_name
                    );
                    for &kc in &active_inputs {
                        map.entry(kc).or_default().push(ComboInfo {
                            color,
                            description: desc.clone(),
                        });
                    }
                }
                map
            })
            .unwrap_or_default();

        // Build tap dance summaries for tooltip display
        // Also build tap dance keycap labels: (tap_label, hold_label)
        let mut td_summaries: HashMap<u16, String> = HashMap::new();
        let mut td_keycap_labels: HashMap<u16, (String, String)> = HashMap::new();
        if let Some(d) = self.dynamic_data.as_ref() {
            for (i, td) in d.tap_dances.iter().enumerate() {
                if td.is_empty() {
                    continue;
                }
                let kc_raw = 0x5700u16 | i as u16;
                let mut parts = Vec::new();
                if td.on_tap != 0 {
                    parts.push(format!("Tap: {}", Keycode(td.on_tap).name()));
                }
                if td.on_hold != 0 {
                    parts.push(format!("Hold: {}", Keycode(td.on_hold).name()));
                }
                if td.on_double_tap != 0 {
                    parts.push(format!("DTap: {}", Keycode(td.on_double_tap).name()));
                }
                if td.on_tap_hold != 0 {
                    parts.push(format!("THold: {}", Keycode(td.on_tap_hold).name()));
                }
                if td.tapping_term > 0 {
                    parts.push(format!("{}ms", td.tapping_term));
                }
                td_summaries.insert(kc_raw, parts.join("  |  "));

                // Keycap: show tap key on top, TD index on bottom
                let tap_label = if td.on_tap != 0 {
                    Keycode(td.on_tap).name()
                } else {
                    format!("TD{i}")
                };
                td_keycap_labels.insert(kc_raw, (tap_label, format!("TD{i}")));
            }
        }

        for (key_idx, key_pos) in layout.keys.iter().enumerate() {
            let raw_kc = data
                .keymap
                .get(layer_idx)
                .and_then(|l| l.get(key_pos.row as usize))
                .and_then(|r| r.get(key_pos.col as usize))
                .copied()
                .unwrap_or(0);
            let keycode = Keycode(raw_kc);

            let px = origin.x + key_pos.x * key_size;
            let py = origin.y + key_pos.y * key_size;
            let pw = key_pos.w * key_size - gap;
            let ph = key_pos.h * key_size - gap;

            let rect = egui::Rect::from_min_size(egui::pos2(px, py), egui::vec2(pw, ph));

            let is_selected = data.selected_key == Some(key_idx);
            let is_hovered = ui.rect_contains_pointer(rect);

            if is_selected {
                selected_key_rect = Some(rect);
            }

            let bg_color = if is_selected {
                egui::Color32::from_rgb(70, 130, 180)
            } else if is_hovered {
                egui::Color32::from_rgb(80, 80, 90)
            } else {
                keycode.category().bg()
            };

            let border_color = if is_selected {
                egui::Color32::from_rgb(100, 180, 255)
            } else {
                egui::Color32::from_rgb(60, 60, 65)
            };

            let rounding = egui::CornerRadius::same(4);
            painter.rect_filled(rect, rounding, bg_color);
            painter.rect_stroke(
                rect,
                rounding,
                egui::Stroke::new(1.0, border_color),
                egui::StrokeKind::Outside,
            );

            let label = keycode.name();

            let text_color = if is_selected {
                egui::Color32::WHITE
            } else if raw_kc == 0 || raw_kc == 1 {
                egui::Color32::from_rgb(100, 100, 110)
            } else {
                egui::Color32::from_rgb(220, 220, 230)
            };

            // Determine split labels: TD keycap labels take priority, then dual_labels
            let split_labels = td_keycap_labels
                .get(&raw_kc)
                .cloned()
                .or_else(|| keycode.dual_labels());

            if let Some((tap, hold)) = split_labels {
                // Split keycap: tap label on top, hold label on bottom
                let tap_font_size = if tap.len() <= 2 {
                    key_size * 0.32
                } else if tap.len() <= 5 {
                    key_size * 0.22
                } else {
                    key_size * 0.16
                };
                let hold_font_size = key_size * 0.14;

                let tap_pos = egui::pos2(rect.center().x, rect.min.y + rect.height() * 0.35);
                let hold_pos = egui::pos2(rect.center().x, rect.min.y + rect.height() * 0.75);

                painter.text(
                    tap_pos,
                    egui::Align2::CENTER_CENTER,
                    &tap,
                    egui::FontId::proportional(tap_font_size),
                    text_color,
                );

                let hold_color = if is_selected {
                    egui::Color32::from_rgb(180, 200, 230)
                } else {
                    egui::Color32::from_rgb(140, 140, 160)
                };
                painter.text(
                    hold_pos,
                    egui::Align2::CENTER_CENTER,
                    &hold,
                    egui::FontId::proportional(hold_font_size),
                    hold_color,
                );
            } else {
                let font_size = if label.len() <= 2 {
                    key_size * 0.35
                } else if label.len() <= 5 {
                    key_size * 0.25
                } else {
                    key_size * 0.18
                };
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &label,
                    egui::FontId::proportional(font_size),
                    text_color,
                );
            }

            // Combo indicators: colored dots in top-right corner, one per combo
            if let Some(combos) = combo_map.get(&raw_kc) {
                let dot_radius = key_size * 0.06;
                for (i, combo) in combos.iter().enumerate() {
                    let dot_pos = egui::pos2(
                        rect.max.x - dot_radius * 2.5 - (i as f32) * dot_radius * 3.0,
                        rect.min.y + dot_radius * 2.5,
                    );
                    painter.circle_filled(dot_pos, dot_radius, combo.color);
                }
            }

            if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                clicked_key = Some(key_idx);
            }

            // Tooltip with debug info and combo details
            if is_hovered {
                egui::Tooltip::always_open(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    ui.id().with(("key_tip", key_idx)),
                    egui::PopupAnchor::Pointer,
                )
                .show(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{}\n0x{:04X}  matrix ({},{})",
                            label, raw_kc, key_pos.row, key_pos.col
                        ))
                        .monospace()
                        .size(12.0),
                    );
                    if let Some(combos) = combo_map.get(&raw_kc) {
                        ui.add_space(4.0);
                        for combo in combos {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(
                                    egui::vec2(8.0, 8.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(r.center(), 4.0, combo.color);
                                ui.label(
                                    egui::RichText::new(&combo.description)
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(180, 200, 220)),
                                );
                            });
                        }
                    }
                    if let Some(td_info) = td_summaries.get(&raw_kc) {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!("Tap Dance: {td_info}"))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(200, 180, 140)),
                        );
                    }
                });
            }
        }

        if let Some(idx) = clicked_key {
            if data.selected_key == Some(idx) {
                data.selected_key = None;
            } else {
                data.selected_key = Some(idx);
            }
        }

        // Close picker on Escape
        if data.selected_key.is_some() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            data.selected_key = None;
        }

        // Close picker on click outside keys and picker
        if data.selected_key.is_some() && clicked_key.is_none() {
            let click_pos = ui.input(|i| {
                if i.pointer.primary_clicked() {
                    i.pointer.interact_pos()
                } else {
                    None
                }
            });
            if let Some(pos) = click_pos {
                let picker_id = egui::Id::new("kc_picker");
                let in_picker = ui
                    .ctx()
                    .memory(|mem| mem.area_rect(picker_id))
                    .is_some_and(|r| r.contains(pos));
                if !in_picker {
                    data.selected_key = None;
                }
            }
        }

        // Floating popover picker
        if let (Some(key_idx), Some(key_rect)) = (data.selected_key, selected_key_rect) {
            let key_pos = &layout.keys[key_idx];
            let raw_kc = data
                .keymap
                .get(layer_idx)
                .and_then(|l| l.get(key_pos.row as usize))
                .and_then(|r| r.get(key_pos.col as usize))
                .copied()
                .unwrap_or(0);
            let keycode = Keycode(raw_kc);
            let kc_name = keycode.name();
            let kc_category = format!("{:?}", keycode.category());

            let popover_w = 420.0_f32;
            let popover_h = 260.0_f32;
            let screen_rect = ui.ctx().content_rect();

            let mut pop_x = key_rect.center().x - popover_w / 2.0;
            let mut pop_y = key_rect.max.y + 8.0;

            if pop_y + popover_h > screen_rect.max.y - 10.0 {
                pop_y = key_rect.min.y - popover_h - 8.0;
            }
            pop_x = pop_x.clamp(screen_rect.min.x + 5.0, screen_rect.max.x - popover_w - 5.0);
            pop_y = pop_y.clamp(screen_rect.min.y + 5.0, screen_rect.max.y - popover_h - 5.0);

            let mut open = true;
            egui::Window::new("Keycode Picker")
                .id(egui::Id::new("kc_picker"))
                .open(&mut open)
                .fixed_pos(egui::pos2(pop_x, pop_y))
                .fixed_size(egui::vec2(popover_w, popover_h))
                .collapsible(false)
                .title_bar(false)
                .show(ui.ctx(), |ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Layer {} [{},{}]",
                                layer_idx, key_pos.row, key_pos.col
                            ))
                            .strong()
                            .size(13.0),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("{kc_name}  {:#06x}", raw_kc))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(160, 160, 175)),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(&kc_category)
                                .size(11.0)
                                .color(egui::Color32::from_rgb(120, 120, 135)),
                        );
                    });

                    ui.add_space(4.0);

                    // Raw keycode hex input
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Hex:")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(130, 130, 145)),
                        );
                        let hex_id = egui::Id::new("picker_hex_input");
                        let mut hex_str: String = ui
                            .memory(|mem| mem.data.get_temp(hex_id))
                            .unwrap_or_else(|| format!("{:04X}", raw_kc));
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut hex_str)
                                .desired_width(60.0)
                                .font(egui::TextStyle::Monospace),
                        );
                        ui.memory_mut(|mem| mem.data.insert_temp(hex_id, hex_str.clone()));

                        if resp.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && let Ok(v) = u16::from_str_radix(hex_str.trim(), 16)
                        {
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(egui::Id::new("pending_keycode"), v);
                                mem.data
                                    .insert_temp(egui::Id::new("pending_key_idx"), key_idx);
                            });
                        }

                        let preview_kc = u16::from_str_radix(hex_str.trim(), 16).unwrap_or(0);
                        if preview_kc != 0 {
                            let preview = Keycode(preview_kc);
                            ui.label(
                                egui::RichText::new(format!("→ {}", preview.name()))
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(140, 170, 200)),
                            );
                        }

                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("Set").size(11.0))
                                    .corner_radius(egui::CornerRadius::same(3)),
                            )
                            .clicked()
                            && let Ok(v) = u16::from_str_radix(hex_str.trim(), 16)
                        {
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(egui::Id::new("pending_keycode"), v);
                                mem.data
                                    .insert_temp(egui::Id::new("pending_key_idx"), key_idx);
                            });
                        }
                    });

                    ui.add_space(2.0);

                    // Group tabs
                    ui.horizontal_wrapped(|ui| {
                        for (i, group) in self.picker_groups.iter().enumerate() {
                            let sel = self.picker_selected_group == i;
                            let label = egui::RichText::new(group.name).size(11.5);
                            if themed_tab(ui, sel, label, &self.theme).clicked() {
                                self.picker_selected_group = i;
                            }
                        }
                    });

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // Keycode grid
                    let group_idx = self.picker_selected_group;
                    let mut picked_kc: Option<u16> = None;

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                                if let Some(group) = self.picker_groups.get(group_idx) {
                                    for kc in &group.codes {
                                        let name = kc.name();
                                        let is_current = kc.0 == raw_kc;
                                        let size = egui::vec2(44.0, 28.0);
                                        let (rect, response) =
                                            ui.allocate_exact_size(size, egui::Sense::click());
                                        let is_hovered = response.hovered();

                                        let bg = if is_current {
                                            egui::Color32::from_rgb(70, 130, 180)
                                        } else if is_hovered {
                                            egui::Color32::from_rgb(80, 80, 90)
                                        } else {
                                            kc.category().bg()
                                        };
                                        let border = if is_current {
                                            egui::Color32::from_rgb(100, 180, 255)
                                        } else {
                                            egui::Color32::from_rgb(60, 60, 65)
                                        };
                                        let text_col = if is_current {
                                            egui::Color32::WHITE
                                        } else if kc.0 == 0 || kc.0 == 1 {
                                            egui::Color32::from_rgb(100, 100, 110)
                                        } else {
                                            egui::Color32::from_rgb(220, 220, 230)
                                        };

                                        let rounding = egui::CornerRadius::same(4);
                                        ui.painter().rect_filled(rect, rounding, bg);
                                        ui.painter().rect_stroke(
                                            rect,
                                            rounding,
                                            egui::Stroke::new(1.0, border),
                                            egui::StrokeKind::Outside,
                                        );

                                        let font_size = if name.len() <= 2 {
                                            12.0
                                        } else if name.len() <= 5 {
                                            10.5
                                        } else {
                                            8.5
                                        };
                                        ui.painter().text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            &name,
                                            egui::FontId::proportional(font_size),
                                            text_col,
                                        );

                                        if response.clicked() {
                                            picked_kc = Some(kc.0);
                                        }
                                        response.on_hover_text(kc.description());
                                    }
                                }
                            });
                        });

                    if let Some(new_kc) = picked_kc {
                        ui.memory_mut(|mem| {
                            mem.data
                                .insert_temp(egui::Id::new("pending_keycode"), new_kc);
                            mem.data
                                .insert_temp(egui::Id::new("pending_key_idx"), key_idx);
                        });
                    }
                });

            // Close popover => deselect key
            if !open && let Some(data) = &mut self.keymap_data {
                data.selected_key = None;
            }
        }

        // Handle deferred keycode application
        let pending: Option<(usize, u16)> = ui.memory(|mem| {
            let kc: Option<u16> = mem.data.get_temp(egui::Id::new("pending_keycode"));
            let idx: Option<usize> = mem.data.get_temp(egui::Id::new("pending_key_idx"));
            match (kc, idx) {
                (Some(kc), Some(idx)) => Some((idx, kc)),
                _ => None,
            }
        });

        if let Some((key_idx, new_kc)) = pending {
            ui.memory_mut(|mem| {
                mem.data.remove::<u16>(egui::Id::new("pending_keycode"));
                mem.data.remove::<usize>(egui::Id::new("pending_key_idx"));
            });
            self.apply_keycode(key_idx, new_kc);
        }
    }

    pub fn apply_keycode(&mut self, key_idx: usize, new_keycode: u16) {
        let Some(data) = &mut self.keymap_data else {
            return;
        };
        let key_pos = &data.layout.keys[key_idx];
        let layer = data.selected_layer;
        let row = key_pos.row;
        let col = key_pos.col;

        let old_keycode = data
            .keymap
            .get(layer)
            .and_then(|l| l.get(row as usize))
            .and_then(|r| r.get(col as usize))
            .copied()
            .unwrap_or(0);

        if old_keycode == new_keycode {
            return;
        }

        if let Some(layer_data) = data.keymap.get_mut(layer)
            && let Some(row_data) = layer_data.get_mut(row as usize)
            && let Some(cell) = row_data.get_mut(col as usize)
        {
            *cell = new_keycode;
        }

        data.undo_stack.push(KeyChange {
            layer,
            row,
            col,
            key_idx,
            old_keycode,
            new_keycode,
        });
        data.dirty = true;

        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            match proto.set_keycode(layer as u8, row, col, new_keycode) {
                Ok(()) => {
                    let kc_name = Keycode(new_keycode).name();
                    info!(
                        layer,
                        row,
                        col,
                        keycode = kc_name,
                        "keycode written to device"
                    );
                    self.set_status(StatusMessage::info(format!(
                        "Set [{row},{col}] -> {kc_name}"
                    )));
                }
                Err(e) => {
                    let err_str = format!("{e}");
                    warn!(error = %e, "failed to write keycode to device");
                    self.set_status(StatusMessage::error(format!("Write failed: {e}")));
                    if is_disconnect_error(&err_str) {
                        self.handle_disconnect();
                    }
                }
            }
        }
    }

    pub fn reload_keymap(&mut self) {
        if let (Some(dev), Some(data)) = (&self.connected_device, &mut self.keymap_data) {
            let proto = ViaProtocol::new(dev);
            match proto.read_entire_keymap(data.layer_count, data.layout.rows, data.layout.cols) {
                Ok(km) => {
                    info!("keymap reloaded");
                    data.keymap = km;
                    data.dirty = false;
                    data.undo_stack.clear();
                    self.set_status(StatusMessage::info("Keymap reloaded from device"));
                }
                Err(e) => {
                    warn!(error = %e, "failed to reload keymap");
                    self.set_status(StatusMessage::error(format!("Reload failed: {e}")));
                }
            }
        }
    }

    pub fn undo(&mut self) {
        let Some(data) = &mut self.keymap_data else {
            return;
        };
        let Some(change) = data.undo_stack.pop() else {
            return;
        };

        if let Some(layer_data) = data.keymap.get_mut(change.layer)
            && let Some(row_data) = layer_data.get_mut(change.row as usize)
            && let Some(cell) = row_data.get_mut(change.col as usize)
        {
            *cell = change.old_keycode;
        }

        if data.undo_stack.is_empty() {
            data.dirty = false;
        }

        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            match proto.set_keycode(
                change.layer as u8,
                change.row,
                change.col,
                change.old_keycode,
            ) {
                Ok(()) => {
                    let name = Keycode(change.old_keycode).name();
                    info!(
                        layer = change.layer,
                        row = change.row,
                        col = change.col,
                        keycode = name,
                        "undo applied"
                    );
                    self.set_status(StatusMessage::info(format!(
                        "Undo: [{},{}] -> {name}",
                        change.row, change.col
                    )));
                }
                Err(e) => {
                    let err_str = format!("{e}");
                    warn!(error = %e, "undo write failed");
                    self.set_status(StatusMessage::error(format!("Undo write failed: {e}")));
                    if is_disconnect_error(&err_str) {
                        self.handle_disconnect();
                    }
                }
            }
        }
    }

    pub fn export_keymap(&mut self) {
        let Some(data) = &self.keymap_data else {
            return;
        };

        let mut layers = Vec::new();
        for (layer_idx, layer) in data.keymap.iter().enumerate() {
            let mut rows = Vec::new();
            for (row_idx, row) in layer.iter().enumerate() {
                let keys: Vec<serde_json::Value> = row
                    .iter()
                    .enumerate()
                    .map(|(col_idx, &raw_kc)| {
                        serde_json::json!({
                            "col": col_idx,
                            "raw": raw_kc,
                            "name": Keycode(raw_kc).name(),
                        })
                    })
                    .collect();
                rows.push(serde_json::json!({
                    "row": row_idx,
                    "keys": keys,
                }));
            }
            layers.push(serde_json::json!({
                "layer": layer_idx,
                "rows": rows,
            }));
        }

        let dump = serde_json::json!({
            "viar_version": 1,
            "layout": data.layout.name,
            "matrix_rows": data.layout.rows,
            "matrix_cols": data.layout.cols,
            "layer_count": data.layer_count,
            "layers": layers,
        });

        let path = "viar_keymap.json";
        let json_str = match serde_json::to_string_pretty(&dump) {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "failed to serialize keymap");
                self.set_status(StatusMessage::error(format!("Export failed: {e}")));
                return;
            }
        };
        match std::fs::write(path, json_str) {
            Ok(_) => {
                info!("keymap exported to {path}");
                if let Some(data) = &mut self.keymap_data {
                    data.dirty = false;
                }
                self.set_status(StatusMessage::info(format!("Exported to {path}")));
            }
            Err(e) => {
                warn!(error = %e, "failed to export keymap");
                self.set_status(StatusMessage::error(format!("Export failed: {e}")));
            }
        }
    }

    pub fn import_keymap(&mut self) {
        let path = "viar_keymap.json";
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, "failed to read keymap file");
                self.set_status(StatusMessage::error(format!("Import failed: {e}")));
                return;
            }
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                self.set_status(StatusMessage::error(format!("Invalid JSON: {e}")));
                return;
            }
        };

        let Some(data) = &self.keymap_data else {
            return;
        };

        let file_rows = json["matrix_rows"].as_u64().unwrap_or(0) as u8;
        let file_cols = json["matrix_cols"].as_u64().unwrap_or(0) as u8;
        let expected_rows = data.layout.rows;
        let expected_cols = data.layout.cols;
        let _ = data;

        if file_rows != expected_rows || file_cols != expected_cols {
            self.set_status(StatusMessage::error(format!(
                "Matrix mismatch: file is {file_rows}x{file_cols}, keyboard is {expected_rows}x{expected_cols}",
            )));
            return;
        }

        let Some(layers) = json["layers"].as_array() else {
            self.set_status(StatusMessage::error("No layers array in file"));
            return;
        };

        let Some(data) = &mut self.keymap_data else {
            return;
        };

        let mut new_keymap = data.keymap.clone();
        for layer_obj in layers {
            let layer_idx = layer_obj["layer"].as_u64().unwrap_or(0) as usize;
            if layer_idx >= new_keymap.len() {
                continue;
            }
            let Some(rows) = layer_obj["rows"].as_array() else {
                continue;
            };
            for row_obj in rows {
                let row_idx = row_obj["row"].as_u64().unwrap_or(0) as usize;
                if row_idx >= new_keymap[layer_idx].len() {
                    continue;
                }
                let Some(keys) = row_obj["keys"].as_array() else {
                    continue;
                };
                for key_obj in keys {
                    let col_idx = key_obj["col"].as_u64().unwrap_or(0) as usize;
                    let raw = key_obj["raw"].as_u64().unwrap_or(0) as u16;
                    if col_idx < new_keymap[layer_idx][row_idx].len() {
                        new_keymap[layer_idx][row_idx][col_idx] = raw;
                    }
                }
            }
        }

        let mut changed = 0usize;
        let mut errors = 0usize;
        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            for (layer, layer_keys) in new_keymap.iter().enumerate() {
                for (row, row_keys) in layer_keys.iter().enumerate() {
                    for (col, &new) in row_keys.iter().enumerate() {
                        let old = data.keymap[layer][row][col];
                        if old != new {
                            match proto.set_keycode(layer as u8, row as u8, col as u8, new) {
                                Ok(()) => changed += 1,
                                Err(e) => {
                                    warn!(error = %e, layer, row, col, "failed to write key");
                                    errors += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        data.keymap = new_keymap;

        if errors > 0 {
            self.set_status(StatusMessage::error(format!(
                "Imported with {errors} write errors ({changed} keys updated)"
            )));
        } else {
            info!(changed, "keymap imported from {path}");
            self.set_status(StatusMessage::info(format!(
                "Imported {changed} key changes from {path}"
            )));
        }
    }
}
