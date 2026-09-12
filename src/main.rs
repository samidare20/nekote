slint::include_modules!();

use slint::{ModelRc, SharedString, VecModel};
use std::fs;
use std::path::Path;
use std::rc::Rc;

fn is_supported_image(path: &Path) -> bool {
    let supported_extensions = ["jpg", "jpeg", "png", "webp", "gif", "bmp"];
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_ascii_lowercase();
        supported_extensions.contains(&ext_lower.as_str())
    } else {
        false
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_window = MainWindow::new()?;

    let window_weak = main_window.as_weak();
    main_window.on_open_folder_clicked(move || {
        let Some(window) = window_weak.upgrade() else {
            return;
        };

        let folder = rfd::FileDialog::new().pick_folder();
        let Some(folder_path) = folder else {
            return;
        };

        window.set_folder_path(SharedString::from(folder_path.to_string_lossy().as_ref()));

        let mut files: Vec<SharedString> = Vec::new();
        if let Ok(entries) = fs::read_dir(&folder_path) {
            let mut entry_paths: Vec<_> = entries
                .filter_map(|res| res.ok().map(|e| e.path()))
                .filter(|path| path.is_file() && is_supported_image(path))
                .collect();

            entry_paths.sort();

            for path in entry_paths {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    files.push(SharedString::from(name));
                }
            }
        }

        let model = Rc::new(VecModel::from(files));
        window.set_file_list(ModelRc::from(model));
    });

    main_window.run()?;
    Ok(())
}

