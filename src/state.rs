use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use ignore::Walk;
use itertools::Itertools;
use lsp_types::{Range, Url, WorkspaceFolder};
use markdown::mdast::Node;

use crate::links::parse_wiki_links;

#[derive(Debug)]
pub struct MdFile {
    buffer: String,
    pub ast: Node,
}

#[derive(Debug, Default)]
pub struct State {
    pub md_files: HashMap<Url, MdFile>,
    workspace_folder: Option<WorkspaceFolder>,
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
        self.md_files
            .get(uri)
            .map(|md_file| md_file.buffer.as_str())
    }

    pub fn index_md_files(&mut self, workspace_folders: &[WorkspaceFolder]) {
        let md_files = self.find_md_files(workspace_folders);
        self.md_files = md_files
            .into_iter()
            .map(|file| {
                log::info!("INDEXING: {:#?}", &file);
                let buffer = fs::read_to_string(&file).unwrap();
                let mut ast = markdown::to_mdast(&buffer, &markdown::ParseOptions::gfm()).unwrap();
                parse_wiki_links(&mut ast);
                let uri = Url::from_file_path(&file).unwrap();

                (uri, MdFile { buffer, ast })
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

    pub fn buffer_range_for_uri(&self, uri: &Url, range: &Range) -> Option<String> {
        let doc = self.buffer_for_uri(uri)?;
        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;
        let start_column = range.start.character as usize;
        let end_column = range.end.character as usize;

        let sliced = doc
            .lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                if line_num == start_line {
                    line.get(start_column..)
                } else if line_num == end_line {
                    line.get(..end_column)
                } else if line_num >= start_line && line_num <= end_line {
                    Some(line)
                } else {
                    None
                }
            })
            .join("\n");
        Some(sliced)
    }

    pub fn cursor_char(&self, uri: &Url, pos: &lsp_types::Position) -> Option<char> {
        let doc = self.buffer_for_uri(uri)?;
        let line = doc.lines().nth(pos.line as usize)?;
        line.chars().nth((pos.character.checked_sub(1))? as usize)
    }

    pub fn peek_behind_position(&self, uri: &Url, pos: &lsp_types::Position) -> Option<char> {
        let doc = self.buffer_for_uri(uri)?;
        let line = doc.lines().nth(pos.line as usize)?;
        line.chars().nth((pos.character.checked_sub(2))? as usize)
    }

    pub fn get_file_list(&self) -> Vec<(&Url, String)> {
        self.md_files
            .keys()
            .filter_map(|url| {
                self.workspace_folder().and_then(|wsf| {
                    let root = PathBuf::from(&wsf.uri.path());
                    let file_path = url.to_file_path().ok()?;
                    let path_from_root = path_from_root(&root, &file_path)?;
                    Some((url, path_from_root))
                })
            })
            .collect()
    }
}

pub fn path_from_root(from: &Path, to: &Path) -> Option<String> {
    if let Ok(rel) = to.strip_prefix(from) {
        Some(format!("/{}", rel.to_string_lossy()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_workspacefolder() -> WorkspaceFolder {
        let workspacefolder_dir = std::env::current_dir().expect("Current directory should exist");
        let s = format!("file:///{}", workspacefolder_dir.to_str().unwrap());
        let wsf_url = Url::parse(&s).expect("Parsing URL failed");
        let wsf_name = workspacefolder_dir.file_name().unwrap().to_str().unwrap();
        WorkspaceFolder {
            uri: wsf_url,
            name: wsf_name.into(),
        }
    }

    fn init_state() -> State {
        let workspace_folder = create_workspacefolder();
        let mut state = State::new();
        state.set_workspace_folder(workspace_folder);
        state
    }

    #[test]
    fn test_path_from_root() {
        let path1 = PathBuf::from("/foo/bar");
        let path2 = PathBuf::from("/foo/bar/baz");
        let path3 = PathBuf::from("/other/bar/baz");
        let from_root1 = path_from_root(&path1, &path2);
        assert_eq!(from_root1, Some("/baz".to_string()));
        let from_root2 = path_from_root(&path1, &path3);
        assert_eq!(from_root2, None);
    }

    #[test]
    fn test_index_md_files() {
        let workspace_folder = create_workspacefolder();
        let root_path = workspace_folder
            .uri
            .to_file_path()
            .expect("workspace_folder URI should be a valid file path");
        let mut state = init_state();
        state.index_md_files(&[workspace_folder]);
        assert!(!state.md_files.is_empty());
        let mut file_names: Vec<String> = state
            .md_files
            .keys()
            .filter_map(|url| {
                url.to_file_path()
                    .ok()
                    .and_then(|path| path.strip_prefix(&root_path).ok().map(|p| p.to_path_buf()))
                    .map(|rel_path| rel_path.display().to_string())
            })
            .collect();
        file_names.sort();
        insta::assert_debug_snapshot!(file_names);
    }
}
