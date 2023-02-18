use std::process::exit;

use eframe::egui;
use crate::source::*;
// 
pub fn run_app() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native(
        "PACTL Mixer",
        options,
        Box::new(|_cc| Box::<Mixer>::default()),
    )
}

pub struct Mixer {
    sources: Sources,
}

impl Default for Mixer {
    fn default() -> Self {
        Self { sources: Sources::new() }
    }
}

impl eframe::App for Mixer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mixer");

            ui.vertical(|ui| {
                for source in self.sources.iter_mut() {
                    ui.label(format!("{} (id: {})", source.name(), source.id()));
                    ui.horizontal(|ui| {
                        ui.label("Volume");
                        if ui.add(egui::Slider::new(&mut source.volume, 0..=200)).changed() {
                            source.flush().unwrap_or_else(|e| {
                                eprintln!("error (when changing {}.volume): {e}", source.id());
                            });
                        }

                        if ui.toggle_value(&mut source.mute, "Mute").changed() {
                            source.flush().unwrap_or_else(|e| {
                                eprintln!("error (when changing {}.mute): {e}", source.id());
                            });
                        }
                    });
                }
            });

            if ui.button("Update").clicked() {
                self.sources.update();
            }

            if ui.button("Exit").clicked() {
                exit(0);
            }
        });
    }
}