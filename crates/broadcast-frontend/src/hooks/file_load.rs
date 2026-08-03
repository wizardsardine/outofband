//! Reads dropped or chosen files via `gloo-file`.

use std::collections::BTreeMap;
use std::rc::Rc;

use gloo_file::futures::read_as_bytes;
use wasm_bindgen_futures::spawn_local;
use web_sys::FileList as WebFileList;
use yew::prelude::*;


#[derive(Debug, PartialEq)]
pub enum LoadedFile {
    Loaded { name: String, bytes: Box<[u8]> },
    Failed { name: String, error: String },
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

}
