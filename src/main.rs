use std::env;
use std::fs;
use regex::Regex;
use std::path::{Path, PathBuf};

fn transform_enums(contents: &str) -> (String, bool) {
    let enum_regex = Regex::new(r"enum\s+(\w+)\s+\{([^}]+)\}").unwrap();

    let mut transformed = false;

    let result = enum_regex.replace_all(&contents, |caps: &regex::Captures| {

        transformed = true;

        let enum_name = &caps[1];
        let enum_body = &caps[2];

        let const_body: String = enum_body.split(',')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.split('=').collect::<Vec<_>>())
            .map(|kv| {
                let kv1 = kv.get(1).unwrap_or(&"").trim();
                format!("{}: {},", kv[0].trim(), kv1)
            })
            .collect::<Vec<_>>()
            .join("\n  ");

        format!("const {} = {{\n  {}\n}} as const;\n\nexport type {} = (typeof {})[keyof typeof {}];", enum_name, const_body, enum_name, enum_name, enum_name)
    }).to_string();

    (result, transformed)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: enum_nuker <root_dir>");

        std::process::exit(1);
    }

    let root_path = &args[1];

    if let Err(e) = visit_dirs(Path::new(root_path), &process_file) {
        eprintln!("Error: {}", e);
    }

}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&PathBuf)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.to_string_lossy().contains("__") {
                continue
            }
            if path.to_string_lossy().contains("node_modules") {
                continue
            }

            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&path);
            }
        }
    }
    Ok(())
}

fn process_file(path: &PathBuf) {
    if let Some(ext) = path.extension() {
        if ext == "ts" {
            println!("👀 - {:?}", path);

            let contents = fs::read_to_string(path).expect("Error reading file");

            let (transformed, changed) = transform_enums(&contents);
            
            if changed {
                println!("  👨‍💻 - {:?}", path);
                fs::write(path, transformed).expect("Error writing file");
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_enum_conversion() {
        let ts_enum = r#"export enum BasicEnum { FIRST = 'first', SECOND = 'second' }"#;
        let expected_output = r#"export const BasicEnum = {
  FIRST: 'first',
  SECOND: 'second',
} as const;

export type BasicEnum = (typeof BasicEnum)[keyof typeof BasicEnum];"#;

        let (result, transformed) = transform_enums(ts_enum);
        assert!(transformed);
        assert_eq!(result.trim(), expected_output);
    }

    #[test]
    fn test_enum_with_trailing_comma() {
        let ts_enum = r#"export enum TrailingCommaEnum { FIRST = 'first', SECOND = 'second', }"#;
        let expected_output = r#"export const TrailingCommaEnum = {
  FIRST: 'first',
  SECOND: 'second',
} as const;

export type TrailingCommaEnum = (typeof TrailingCommaEnum)[keyof typeof TrailingCommaEnum];"#;

        let (result, transformed) = transform_enums(ts_enum);
        assert!(transformed);
        assert_eq!(result.trim(), expected_output);
    }

    #[test]
    fn test_enum_with_additional_spacing() {
        let ts_enum = r#"export enum SpacingEnum { 
    FIRST = 'first', 
    SECOND = 'second' 
}"#;
        let expected_output = r#"export const SpacingEnum = {
  FIRST: 'first',
  SECOND: 'second',
} as const;

export type SpacingEnum = (typeof SpacingEnum)[keyof typeof SpacingEnum];"#;

        let (result, transformed) = transform_enums(ts_enum);
        assert!(transformed);
        assert_eq!(result.trim(), expected_output);
    }

    #[test]
    fn test_no_enums_present() {
        let ts_content = r#"export const someVariable = 123;"#;
        let (result, transformed) = transform_enums(ts_content);
        assert!(!transformed);
        assert_eq!(result.trim(), ts_content);
    }

}

