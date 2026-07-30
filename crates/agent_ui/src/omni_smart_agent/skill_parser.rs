use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents a typed I/O port on a Skill node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillPort {
    pub name: String,
    pub port_type: PortType,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PortType {
    Text,
    File,
    Json,
    Image,
    Any,
}

/// Parsed metadata from a SKILL.md file's YAML frontmatter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub source_path: PathBuf,
    pub input_ports: Vec<SkillPort>,
    pub output_ports: Vec<SkillPort>,
}

pub struct SkillParser;

impl SkillParser {
    /// Scans a directory for SKILL.md files and parses their metadata.
    pub fn scan_directory(skills_dir: &Path) -> Vec<SkillMetadata> {
        let mut results = Vec::new();

        let entries = match std::fs::read_dir(skills_dir) {
            Ok(entries) => entries,
            Err(_) => return results,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Ok(metadata) = Self::parse_skill_file(&skill_file) {
                        results.push(metadata);
                    }
                }
            }
        }

        results
    }

    /// Parses a single SKILL.md file extracting name/description from YAML frontmatter.
    fn parse_skill_file(path: &Path) -> Result<SkillMetadata> {
        let content = std::fs::read_to_string(path)?;

        let mut name = String::new();
        let mut description = String::new();

        if content.starts_with("---") {
            if let Some(end_idx) = content[3..].find("---") {
                let frontmatter = &content[3..3 + end_idx];
                for line in frontmatter.lines() {
                    let line = line.trim();
                    if let Some(value) = line.strip_prefix("name:") {
                        name = value.trim().trim_matches('"').trim_matches('\'').to_string();
                    } else if let Some(value) = line.strip_prefix("description:") {
                        description = value.trim().trim_matches('"').trim_matches('\'').to_string();
                    }
                }
            }
        }

        if name.is_empty() {
            name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
        }

        Ok(SkillMetadata {
            name,
            description,
            source_path: path.to_path_buf(),
            input_ports: vec![SkillPort {
                name: "input".to_string(),
                port_type: PortType::Text,
                description: "Default text input".to_string(),
            }],
            output_ports: vec![SkillPort {
                name: "output".to_string(),
                port_type: PortType::Text,
                description: "Default text output".to_string(),
            }],
        })
    }

    /// Scans all known skill directories on the system.
    pub fn scan_all_skill_sources() -> Vec<SkillMetadata> {
        let mut all_skills = Vec::new();

        if let Ok(home_str) = std::env::var("HOME") {
            let home = PathBuf::from(home_str);

            let gemini_skills = home.join(".gemini").join("config").join("skills");
            all_skills.extend(Self::scan_directory(&gemini_skills));

            let config_skills = home.join(".config").join("_skills_");
            all_skills.extend(Self::scan_directory(&config_skills));
        }

        all_skills
    }
}
