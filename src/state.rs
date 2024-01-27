use std::{collections::HashMap, fs, path::PathBuf};

use ignore::Walk;
use lsp_types::{Url, WorkspaceFolder};
use markdown::mdast::Node;

use crate::links::parse_wiki_links;

#[derive(Debug)]
struct MdFile {
    buffer: String,
    ast: Node,
}

#[derive(Debug, Default)]
pub struct State {
    md_files: HashMap<Url, MdFile>,
    pub workspace_folder: Option<WorkspaceFolder>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace_folder(&self) -> Option<&WorkspaceFolder> {
        self.workspace_folder.as_ref()
    }

    pub fn set_workspace_folder(&mut self, workspace_folder: WorkspaceFolder) {
        self.workspace_folder = Some(workspace_folder);
    }

    pub fn set_buffer(&mut self, uri: &Url, buffer: String) {
        let ast = markdown::to_mdast(&buffer, &markdown::ParseOptions::gfm()).unwrap();
        if let Some(md_file) = self.md_files.get_mut(uri) {
            md_file.ast = ast;
            parse_wiki_links(&mut md_file.ast);
            md_file.buffer = buffer;
        } else {
            let md_file = MdFile { buffer, ast };
            self.md_files.insert(uri.clone(), md_file);
        }
    }

    pub fn ast_for_uri(&self, uri: &Url) -> Option<&Node> {
        self.md_files.get(uri).map(|md_file| &md_file.ast)
    }

    pub fn buffer_for_uri(&self, uri: &Url) -> Option<&str> {
        self.md_files.get(uri).map(|md_file| md_file.buffer.as_str())
    }

    pub fn index_md_files(&mut self, workspace_folders: &[WorkspaceFolder]) {
        let md_files = self.find_md_files(workspace_folders);
        self.md_files = md_files
            .into_iter()
            .map(|file| {
                let buffer = fs::read_to_string(&file).unwrap();
                let ast = markdown::to_mdast(&buffer, &markdown::ParseOptions::gfm()).unwrap();
                let uri = Url::from_file_path(&file).unwrap();

                (
                    uri,
                    MdFile {
                        buffer,
                        ast,
                    },
                )
            })
            .collect();
    }

    fn find_md_files(&self, workspace_folders: &[WorkspaceFolder]) -> Vec<PathBuf> {
        let mut md_files = Vec::new();
        for folder in workspace_folders {
            if let Ok(f) = folder.uri.to_file_path() {
                for entry in Walk::new(f).flatten() {
                    let path = entry.into_path();
                    if path.extension().is_some_and(|ext| ext == "md") {
                        md_files.push(path)
                    }
                }
            }
        }
        md_files
    }
}
