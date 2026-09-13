slint::include_modules!();

use slint::{Image, Model, ModelRc, SharedString, VecModel};
use i_slint_core::DataTransfer;
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

fn preview_name(path: &Path, position: usize, item_count: usize) -> SharedString {
    let padding = 3.max(item_count.to_string().len());
    let extension = path.extension().and_then(|extension| extension.to_str()).unwrap_or_default();
    SharedString::from(format!("{:0padding$}.{}", position + 1, extension))
}

fn refresh_preview_names(items: &mut [ImageItem]) {
    let item_count = items.len();
    for (position, item) in items.iter_mut().enumerate() {
        let extension = item.name.rsplit_once('.').map(|(_, extension)| extension).unwrap_or_default();
        let padding = 3.max(item_count.to_string().len());
        item.preview_name = SharedString::from(format!("{:0padding$}.{}", position + 1, extension));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_window = MainWindow::new()?;
    let file_model = Rc::new(VecModel::<ImageItem>::default());
    main_window.set_file_list(ModelRc::from(file_model.clone()));

    let drag_window_weak = main_window.as_weak();
    main_window.on_prepare_drag(move |index| {
        if let Some(window) = drag_window_weak.upgrade() {
            window.set_drag_data(DataTransfer::from(SharedString::from(index.to_string())));
        }
    });

    let reorder_model = file_model.clone();
    main_window.on_reorder_requested(move |from, to, after| {
        let (Ok(from), Ok(to)) = (usize::try_from(from), usize::try_from(to)) else {
            return;
        };
        if from >= reorder_model.row_count() || to >= reorder_model.row_count() || from == to {
            return;
        }

        let item = reorder_model.remove(from);
        // Dropping on the top/bottom half inserts before/after the target row.
        let insertion_point = to + usize::from(after);
        let insert_at = if from < insertion_point {
            insertion_point - 1
        } else {
            insertion_point
        };
        reorder_model.insert(insert_at, item);

        let mut items: Vec<_> = (0..reorder_model.row_count())
            .filter_map(|index| reorder_model.row_data(index))
            .collect();
        refresh_preview_names(&mut items);
        reorder_model.set_vec(items);
    });

    let window_weak = main_window.as_weak();
    let load_model = file_model.clone();
    main_window.on_open_folder_clicked(move || {
        let Some(window) = window_weak.upgrade() else {
            return;
        };

        let folder = rfd::FileDialog::new().pick_folder();
        let Some(folder_path) = folder else {
            return;
        };

        window.set_folder_path(SharedString::from(folder_path.to_string_lossy().as_ref()));

        let mut items: Vec<ImageItem> = Vec::new();
        if let Ok(entries) = fs::read_dir(&folder_path) {
            let mut entry_paths: Vec<_> = entries
                .filter_map(|res| res.ok().map(|e| e.path()))
                .filter(|path| path.is_file() && is_supported_image(path))
                .collect();

            entry_paths.sort();

            let item_count = entry_paths.len();
            for path in entry_paths {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let thumbnail = Image::load_from_path(&path).unwrap_or_default();
                    items.push(ImageItem {
                        name: SharedString::from(name),
                        preview_name: preview_name(&path, items.len(), item_count),
                        thumbnail,
                    });
                }
            }
        }

        load_model.set_vec(items);
    });

    main_window.run()?;
    Ok(())
}
