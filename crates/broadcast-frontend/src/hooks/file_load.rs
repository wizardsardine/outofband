//! Reads dropped or chosen files via `gloo-file`.

use std::collections::BTreeMap;
use std::rc::Rc;

use gloo_file::futures::read_as_bytes;
use wasm_bindgen_futures::spawn_local;
use web_sys::FileList as WebFileList;
use yew::prelude::*;

use crate::unpack::DECOMPRESSION_BUDGET_BYTES;

#[derive(Debug, PartialEq)]
pub enum LoadedFile {
    Loaded { name: String, bytes: Box<[u8]> },
    Failed { name: String, error: String },
}

#[derive(Debug, PartialEq)]
enum LoadSizeViolation {
    File(usize),
    Aggregate,
}

fn load_size_violation(sizes: &[u64]) -> Option<LoadSizeViolation> {
    if let Some(index) = sizes
        .iter()
        .position(|size| *size > DECOMPRESSION_BUDGET_BYTES)
    {
        return Some(LoadSizeViolation::File(index));
    }
    let total = sizes
        .iter()
        .try_fold(0u64, |total, size| total.checked_add(*size));
    match total {
        Some(total) if total <= DECOMPRESSION_BUDGET_BYTES => None,
        Some(_) | None => Some(LoadSizeViolation::Aggregate),
    }
}

#[derive(Default)]
struct LoadCoordinator {
    next_invocation: u64,
    next_emission: u64,
    completed: BTreeMap<u64, Vec<LoadedFile>>,
}

impl LoadCoordinator {
    fn start(&mut self) -> u64 {
        let invocation = self.next_invocation;
        self.next_invocation += 1;
        invocation
    }

    fn complete(&mut self, invocation: u64, files: Vec<LoadedFile>) -> Vec<Vec<LoadedFile>> {
        self.completed.insert(invocation, files);
        let mut ready = Vec::new();
        while let Some(files) = self.completed.remove(&self.next_emission) {
            ready.push(files);
            self.next_emission += 1;
        }
        ready
    }
}

#[hook]
pub fn use_file_load(on_loaded: Callback<Vec<LoadedFile>>) -> Callback<WebFileList> {
    let coordinator = use_mut_ref(LoadCoordinator::default);

    Callback::from(move |file_list: WebFileList| {
        let invocation = coordinator.borrow_mut().start();
        let coordinator = Rc::clone(&coordinator);
        let on_loaded = on_loaded.clone();
        let mut files: Vec<gloo_file::File> = gloo_file::FileList::from(file_list).to_vec();
        files.sort_by_key(gloo_file::File::name);

        spawn_local(async move {
            let sizes: Vec<u64> = files.iter().map(|file| file.size()).collect();
            let loaded = match load_size_violation(&sizes) {
                Some(LoadSizeViolation::File(index)) => vec![LoadedFile::Failed {
                    name: files[index].name(),
                    error: format!(
                        "file size exceeds the {} MiB load limit",
                        DECOMPRESSION_BUDGET_BYTES / (1024 * 1024)
                    ),
                }],
                Some(LoadSizeViolation::Aggregate) => vec![LoadedFile::Failed {
                    name: "Selected files".to_string(),
                    error: format!(
                        "total file size exceeds the {} MiB load limit",
                        DECOMPRESSION_BUDGET_BYTES / (1024 * 1024)
                    ),
                }],
                None => {
                    let mut loaded = Vec::with_capacity(files.len());
                    for file in &files {
                        let name = file.name();
                        match read_as_bytes(file).await {
                            Ok(bytes) => loaded.push(LoadedFile::Loaded {
                                name,
                                bytes: bytes.into_boxed_slice(),
                            }),
                            Err(err) => loaded.push(LoadedFile::Failed {
                                name,
                                error: format!("could not read file: {err:?}"),
                            }),
                        }
                    }
                    loaded
                }
            };

            for files in coordinator.borrow_mut().complete(invocation, loaded) {
                on_loaded.emit(files);
            }
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failed(name: &str) -> LoadedFile {
        LoadedFile::Failed {
            name: name.to_string(),
            error: "error".to_string(),
        }
    }

    #[test]
    fn overlapping_loads_emit_in_invocation_order() {
        let mut coordinator = LoadCoordinator::default();
        let first = coordinator.start();
        let second = coordinator.start();

        assert!(
            coordinator
                .complete(second, vec![failed("second")])
                .is_empty()
        );
        assert_eq!(
            coordinator.complete(first, vec![failed("first")]),
            vec![vec![failed("first")], vec![failed("second")]]
        );
    }

    #[test]
    fn load_sizes_enforce_file_and_aggregate_limits_before_reading() {
        assert_eq!(load_size_violation(&[DECOMPRESSION_BUDGET_BYTES]), None);
        assert_eq!(
            load_size_violation(&[DECOMPRESSION_BUDGET_BYTES + 1]),
            Some(LoadSizeViolation::File(0))
        );
        assert_eq!(
            load_size_violation(&[
                DECOMPRESSION_BUDGET_BYTES / 2,
                DECOMPRESSION_BUDGET_BYTES / 2 + 1,
            ]),
            Some(LoadSizeViolation::Aggregate)
        );
    }
}
