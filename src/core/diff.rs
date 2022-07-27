use polars::prelude::*;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

use crate::utils::util::get_keys;

pub fn diff_column(file: &PathBuf, remap_csv: &PathBuf) -> Result<Vec<String>> {
    let (ori_key, _) = get_keys(remap_csv.to_str().unwrap())
        .unwrap_or_else(|e| panic!("{:?} {}", remap_csv, e));
    let source = ori_key.join("\n");
    let df = CsvReader::from_path(file)?.with_skip_rows(3).finish()?;
    let col = df.get_column_names();
    let target = col.join("\n");

    let diff = TextDiff::from_lines(&source, &target);

    let result = vec![];
    for (_idx, group) in diff.grouped_ops(3).iter().enumerate() {
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                if change.old_index() == change.new_index() {
                    continue;
                }
                let line = String::new();
                print!(
                    "{} | {: <4} {: <4} | ",
                    &sign,
                    if let Some(i) = change.old_index() {
                        i.to_string()
                    } else {
                        " ".to_string()
                    },
                    if let Some(i) = change.new_index() {
                        i.to_string()
                    } else {
                        " ".to_string()
                    },
                );
                let text = change.iter_strings_lossy().fold(
                    String::new(),
                    |mut v, (_, s)| {
                        v.push_str(&s);
                        v
                    },
                );
                print!("{}", text);
                if change.missing_newline() {
                    println!("");
                }
            }
        }
    }

    Ok(result)
}
