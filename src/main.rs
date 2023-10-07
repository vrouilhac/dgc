use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

#[derive(Debug)]
struct Page {
    metadata: Metadata<String>,
    content: String,
    filename: String,
}

impl Page {
    fn new(metadata: Metadata<String>, content: String, filename: String) -> Self {
        Self {
            metadata,
            content,
            filename,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Metadata<AT> {
    id: Option<String>,
    aliases: Option<AT>,
    title: Option<String>,
    tags: Option<Vec<String>>,

    #[serde(rename = "createdAt")]
    created_at: Option<String>,

    #[serde(rename = "updatedAt")]
    updated_at: Option<String>,

    dg: Option<bool>,
    published: Option<bool>,
    dg_path: Option<String>,
}

fn read_from_path(path: &str) -> String {
    let content = fs::read_to_string(Path::new(path)).unwrap();
    content
}

fn parse_file(content: String, filename: String) -> Page {
    let content: Vec<&str> = content.split("---").collect();
    let page_content = &content[2..];
    let content = content.get(1).unwrap();
    let metadata: Result<Metadata<String>, serde_yaml::Error> = serde_yaml::from_str(&content);
    if let Ok(m) = metadata {
        let page = Page::new(m, page_content.join("").to_string(), filename);
        return page;
    }

    let metadata: Metadata<Vec<String>> = serde_yaml::from_str(&content).unwrap();
    let metadata: Metadata<String> = Metadata {
        id: metadata.id,
        tags: metadata.tags,
        published: metadata.published,
        dg: metadata.dg,
        title: metadata.title,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        aliases: match metadata.aliases {
            Some(v) => Some(v[0].clone()),
            None => Some(String::from("")),
        },
        dg_path: metadata.dg_path,
    };
    let page = Page::new(metadata, page_content.join("").to_string(), filename);
    page
}

fn has_file_diff(origin: &String, comparee: &String) -> bool {
    if origin.trim() == comparee.trim() {
        return false;
    }

    true
}

fn build_path(dg_path: String) -> String {
    format!("./dist/{dg_path}")
}

fn check_update_need(page: &Page) -> bool {
    if let Some(dg_path) = page.metadata.dg_path.clone() {
        let str_path = build_path(dg_path);

        if !Path::new(&str_path).is_dir() {
            fs::create_dir_all(&str_path).expect(&format!(
                "Error: Could not create a directory at \"{}\"",
                str_path
            ));
        }

        let file_path = format!("{}/index.md", &str_path);
        let dest_path = Path::new(&file_path);
        let content = fs::read_to_string(dest_path);

        return match content {
            Ok(c) => {
                let has_diff = has_file_diff(&page.content, &c);
                has_diff
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    return true;
                }

                false
            }
        };
    }

    false
}

fn filename_to_destname(filename: String) -> String {
    filename["xxxxxxxxxxxxx".len()..].to_string() // this string is due to zettelkasten formatting
                                                  // on each and every notes e.g. 202301011200__<Title Here> (_ = space)
}

fn write_update_to_file(str_path: &String, content: &str) {
    let dest_path = Path::new(str_path);
    fs::write(dest_path, content).unwrap();
}

fn check_and_update_file(filename: String) -> bool {
    let path = format!("./origin/{}", filename); // here ./origin is arbitrary
    let raw_content = read_from_path(&path);
    let page = parse_file(raw_content, filename.to_string());
    let published = page.metadata.published.unwrap_or(false);
    let dg = page.metadata.dg.unwrap_or(false);

    if !dg {
        return false;
    }

    if check_update_need(&page) {
        if dg && published {
            let dg_path = page.metadata.dg_path.unwrap();
            let str_path = build_path(dg_path);
            let file_path = format!("{}/index.md", &str_path);
            write_update_to_file(&file_path, page.content.trim());
            return true;
        }
    }

    false
}

fn main() {
    let rd = fs::read_dir("./origin").unwrap();
    let mut updated_files = 0;

    for entry in rd {
        if let Ok(e) = entry {
            if e.path().is_file() {
                let filename = e.file_name();
                let filename = filename.to_str();

                if let Some(filename) = filename {
                    let updated = check_and_update_file(filename.to_string());

                    if updated {
                        updated_files += 1;
                    }
                }
            }
        }
    }

    println!("{updated_files} updated files");
}
