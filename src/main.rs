pub mod cli;
pub mod source;

#[cfg(feature = "gui")]
pub mod gui;

#[cfg(feature = "gui")]
pub fn main() -> Result<(), eframe::Error> {
    cli::cli::<eframe::Error>(Some(&gui::run_app))
}

#[cfg(not(any(feature = "gui")))]
pub fn main() {
    cli::cli(None);
}
