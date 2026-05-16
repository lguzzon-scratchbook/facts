use anyhow::{Result, bail};

use crate::init::{self, SKILLS};
use crate::project;

pub enum SkillsAction {
    List,
    Show { name: String },
    Update,
}

pub fn run(action: &SkillsAction) -> Result<()> {
    match action {
        SkillsAction::List => list(),
        SkillsAction::Show { name } => show(name),
        SkillsAction::Update => update(),
    }
}

fn list() -> Result<()> {
    for (name, content) in SKILLS {
        let desc = extract_description(content);
        println!("{name:20} {desc}");
    }
    println!();
    println!("Use `facts skills show <name>` to read a skill.");
    println!("Use `facts skills update` to install or update skills in your project.");
    Ok(())
}

fn show(name: &str) -> Result<()> {
    for (skill_name, content) in SKILLS {
        if *skill_name == name {
            print!("{content}");
            return Ok(());
        }
    }
    bail!("unknown skill '{name}'. Run `facts skills` to see available skills.")
}

fn update() -> Result<()> {
    let root = project::find_project_root()?;

    for (name, content) in SKILLS {
        init::install_skill(&root, name, content)?;
    }

    if init::is_claude_available(&root) {
        for (name, _) in SKILLS {
            init::link_skill_for_claude(&root, name)?;
        }
    }

    Ok(())
}

fn extract_description(content: &str) -> String {
    let mut in_frontmatter = false;
    let mut in_description = false;
    let mut parts = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        if trimmed.starts_with("description:") {
            in_description = true;
            let rest = trimmed.strip_prefix("description:").unwrap().trim();
            if !rest.is_empty() && rest != ">" && rest != "|" {
                parts.push(rest.to_string());
            }
            continue;
        }
        if in_description {
            if !trimmed.is_empty() && !trimmed.contains(':')
                || trimmed.starts_with(' ')
                || line.starts_with("  ")
            {
                parts.push(trimmed.to_string());
            } else {
                break;
            }
        }
    }

    let full = parts.join(" ");
    if full.len() > 72 {
        let mut end = 72;
        while end > 0 && !full.as_bytes()[end].is_ascii_whitespace() {
            end -= 1;
        }
        if end == 0 {
            end = 72;
        }
        format!("{}…", &full[..end].trim_end())
    } else {
        full
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_description_simple() {
        let content = "---\nname: test\ndescription: A short description\n---\nrest";
        let desc = extract_description(content);
        assert_eq!(desc, "A short description");
    }

    #[test]
    fn test_extract_description_multiline() {
        let content = "---\nname: test\ndescription: |\n  A longer description\n  spanning multiple lines\n---\nrest";
        let desc = extract_description(content);
        assert!(desc.contains("A longer description"));
        assert!(desc.contains("spanning multiple lines"));
    }

    #[test]
    fn test_extract_description_long() {
        let content = "---\nname: test\ndescription: This is a very long description that exceeds seventy two characters and should be truncated with an ellipsis at the end\n---\nrest";
        let desc = extract_description(content);
        assert!(desc.len() <= 75); // 72 chars + ellipsis character
        assert!(desc.contains("…"));
    }

    #[test]
    fn test_extract_description_no_frontmatter() {
        let content = "No frontmatter here\nJust plain text";
        let desc = extract_description(content);
        assert_eq!(desc, "");
    }

    #[test]
    fn test_extract_description_empty() {
        let content = "---\nname: test\n---\nrest";
        let desc = extract_description(content);
        assert_eq!(desc, "");
    }

    #[test]
    fn test_extract_description_empty_value() {
        let content = "---\nname: test\ndescription:\n---\nrest";
        let desc = extract_description(content);
        assert_eq!(desc, "");
    }

    #[test]
    fn test_extract_description_yaml_block() {
        let content = "---\nname: test\ndescription: >\n  This uses yaml block\n---\nrest";
        let desc = extract_description(content);
        // Block indicator should be skipped
        assert!(!desc.contains(">"));
    }
}
