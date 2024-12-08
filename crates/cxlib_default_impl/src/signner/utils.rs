use cxlib_error::SignError;
use std::path::PathBuf;

pub fn find_latest_pic(pic_dir: &PathBuf) -> Result<PathBuf, SignError> {
    let pic_path = {
        let pic_dir = std::fs::read_dir(pic_dir)?;
        let mut all_files_in_dir = pic_dir
            .filter_map(|entry| {
                entry.ok().and_then(|entry| {
                    let file_type = entry.file_type().ok()?;
                    if file_type.is_file() && {
                        let file_name = entry.file_name();
                        file_name.to_str().is_some_and(|file_name| {
                            file_name
                                .split('.')
                                .last()
                                .is_some_and(|file_ext| file_ext == "png" || file_ext == "jpg")
                        })
                    } {
                        Some(entry)
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        all_files_in_dir.sort_by(|a, b| {
            b.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&a.metadata().unwrap().modified().unwrap())
        });
        all_files_in_dir.first().map(|d| d.path()).ok_or_else(|| {
            SignError::SignDataNotFound("图片文件夹下没有图片（`png` 或 `jpg` 文件）！".to_owned())
        })?
    };
    Ok(pic_path)
}
