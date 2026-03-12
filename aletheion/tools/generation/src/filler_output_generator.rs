use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageKind {
    Lua,
    JavaScript,
    Rust,
    Cpp,
    Kotlin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FillerReadiness {
    BlockedByResearch,
    PendingImplementation,
    ReadyForImplementation,
}

#[derive(Debug, Clone)]
pub struct ResearchDependency {
    pub research_gap_id: String,
    pub manifest_path: PathBuf,
    pub owner: String,
    pub deadline: String,
}

#[derive(Debug, Clone)]
pub struct TargetFileDescriptor {
    pub file_id: u32,
    pub relative_path: PathBuf,
    pub language: LanguageKind,
    pub research_dependency: Option<ResearchDependency>,
    pub readiness: FillerReadiness,
}

#[derive(Debug, Clone)]
pub struct TemplateDescriptor {
    pub language: LanguageKind,
    pub template_name: String,
    pub absolute_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FillerContext {
    pub repository_root: PathBuf,
    pub template_root: PathBuf,
    pub manifest_root: PathBuf,
    pub targets: Vec<TargetFileDescriptor>,
    pub templates: HashMap<(LanguageKind, String), TemplateDescriptor>,
}

impl LanguageKind {
    pub fn extension(&self) -> &'static str {
        match self {
            LanguageKind::Lua => "lua",
            LanguageKind::JavaScript => "js",
            LanguageKind::Rust => "rs",
            LanguageKind::Cpp => "cpp",
            LanguageKind::Kotlin => "kt",
        }
    }

    pub fn template_folder(&self) -> &'static str {
        match self {
            LanguageKind::Lua => "lua",
            LanguageKind::JavaScript => "js",
            LanguageKind::Rust => "rust",
            LanguageKind::Cpp => "cpp",
            LanguageKind::Kotlin => "kotlin",
        }
    }
}

impl FillerReadiness {
    pub fn label(&self) -> &'static str {
        match self {
            FillerReadiness::BlockedByResearch => "BLOCKED_BY_RESEARCH",
            FillerReadiness::PendingImplementation => "PENDING_IMPLEMENTATION",
            FillerReadiness::ReadyForImplementation => "READY_FOR_IMPLEMENTATION",
        }
    }
}

impl FillerContext {
    pub fn new(repository_root: PathBuf, template_root: PathBuf, manifest_root: PathBuf) -> Self {
        Self {
            repository_root,
            template_root,
            manifest_root,
            targets: Vec::new(),
            templates: HashMap::new(),
        }
    }

    pub fn register_template(
        &mut self,
        language: LanguageKind,
        template_name: impl Into<String>,
        relative_path: impl AsRef<Path>,
    ) {
        let template_name_string = template_name.into();
        let absolute_path = self.template_root.join(language.template_folder()).join(relative_path);
        let descriptor = TemplateDescriptor {
            language: language.clone(),
            template_name: template_name_string.clone(),
            absolute_path,
        };
        self.templates
            .insert((language, template_name_string), descriptor);
    }

    pub fn add_target(&mut self, descriptor: TargetFileDescriptor) {
        self.targets.push(descriptor);
    }

    pub fn generate_all(&self) -> Result<(), String> {
        for target in &self.targets {
            self.generate_for_target(target)?;
        }
        Ok(())
    }

    fn generate_for_target(&self, target: &TargetFileDescriptor) -> Result<(), String> {
        let template_key = self.resolve_template_key(target);
        let template_descriptor = self.templates.get(&template_key).ok_or_else(|| {
            format!(
                "Missing template for language {:?} and key {:?}",
                template_key.0, template_key.1
            )
        })?;

        let template_content =
            read_to_string(&template_descriptor.absolute_path).map_err(|err| {
                format!(
                    "Failed to read template {:?}: {}",
                    template_descriptor.absolute_path, err
                )
            })?;

        let rendered = self.render_template(&template_content, target)?;
        let absolute_target = self.repository_root.join(&target.relative_path);

        if let Some(parent) = absolute_target.parent() {
            if !parent.exists() {
                create_dir_all(parent).map_err(|err| {
                    format!("Failed to create parent directories {:?}: {}", parent, err)
                })?;
            }
        }

        write(&absolute_target, rendered).map_err(|err| {
            format!(
                "Failed to write filler output file {:?}: {}",
                absolute_target, err
            )
        })?;

        Ok(())
    }

    fn resolve_template_key(
        &self,
        target: &TargetFileDescriptor,
    ) -> (LanguageKind, String) {
        let base_name = target
            .relative_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let key = format!("{}.template", base_name);
        (target.language.clone(), key)
    }

    fn render_template(
        &self,
        template: &str,
        target: &TargetFileDescriptor,
    ) -> Result<String, String> {
        let mut output = String::new();
        let header = self.build_header_block(target);
        output.push_str(&header);
        output.push('\n');
        let mut body = template.to_owned();

        body = body.replace("{{FILE_ID}}", &target.file_id.to_string());
        body = body.replace("{{FILLER_READINESS}}", target.readiness.label());

        if let Some(dep) = &target.research_dependency {
            body = body.replace("{{RESEARCH_GAP_ID}}", &dep.research_gap_id);
            body = body.replace(
                "{{RESEARCH_MANIFEST_PATH}}",
                &self.manifest_root.join(&dep.manifest_path).to_string_lossy(),
            );
            body = body.replace("{{RESEARCH_OWNER}}", &dep.owner);
            body = body.replace("{{RESEARCH_DEADLINE}}", &dep.deadline);
        } else {
            body = body.replace("{{RESEARCH_GAP_ID}}", "NONE");
            body = body.replace("{{RESEARCH_MANIFEST_PATH}}", "NONE");
            body = body.replace("{{RESEARCH_OWNER}}", "NONE");
            body = body.replace("{{RESEARCH_DEADLINE}}", "NONE");
        }

        output.push_str(&body);
        Ok(output)
    }

    fn build_header_block(&self, target: &TargetFileDescriptor) -> String {
        let mut lines = Vec::new();

        let comment_prefix = match target.language {
            LanguageKind::Lua => "--",
            LanguageKind::JavaScript => "//",
            LanguageKind::Rust => "//",
            LanguageKind::Cpp => "//",
            LanguageKind::Kotlin => "//",
        };

        lines.push(format!(
            "{} ALETHEION-FILLER-START",
            comment_prefix
        ));
        lines.push(format!(
            "{} This file is a structural placeholder for File ID: {}.",
            comment_prefix, target.file_id
        ));
        lines.push(format!(
            "{} Filler readiness: {}.",
            comment_prefix,
            target.readiness.label()
        ));

        if let Some(dep) = &target.research_dependency {
            lines.push(format!(
                "{} It requires resolution of Research Gap: {}.",
                comment_prefix, dep.research_gap_id
            ));
            lines.push(format!(
                "{} Owner: {}, Deadline: {}.",
                comment_prefix, dep.owner, dep.deadline
            ));
            lines.push(format!(
                "{} See manifest: {}.",
                comment_prefix,
                self.manifest_root.join(&dep.manifest_path).to_string_lossy()
            ));
        } else {
            lines.push(format!(
                "{} This file currently has no registered external research dependency.",
                comment_prefix
            ));
        }

        lines.push(format!(
            "{} ALETHEION-FILLER-END",
            comment_prefix
        ));

        lines.join("\n")
    }
}

fn default_repository_root() -> PathBuf {
    PathBuf::from(".")
}

fn default_template_root() -> PathBuf {
    PathBuf::from("aletheion/tools/templates/tier2")
}

fn default_manifest_root() -> PathBuf {
    PathBuf::from("aletheion/tools/research/manifests")
}

fn bootstrap_tier2_targets(context: &mut FillerContext) {
    context.add_target(TargetFileDescriptor {
        file_id: 151,
        relative_path: PathBuf::from("aletheion-agri/soil/water/soil_microbiome.rs"),
        language: LanguageKind::Rust,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-001".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Ag_Team".to_string(),
            deadline: "2026-04-10".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });

    context.add_target(TargetFileDescriptor {
        file_id: 152,
        relative_path: PathBuf::from("aletheion-agri/soil/water/soil_microbiome.lua"),
        language: LanguageKind::Lua,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-001".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Ag_Team".to_string(),
            deadline: "2026-04-10".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });

    context.add_target(TargetFileDescriptor {
        file_id: 153,
        relative_path: PathBuf::from("aletheion-env/monitoring/sensors/mof_water_harvesting.js"),
        language: LanguageKind::JavaScript,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-003".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Env_Team".to_string(),
            deadline: "2026-04-20".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });

    context.add_target(TargetFileDescriptor {
        file_id: 165,
        relative_path: PathBuf::from("aletheion-agri/crops/ecology/indigenous_agriculture.rs"),
        language: LanguageKind::Rust,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-002".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Treaty_Office".to_string(),
            deadline: "2026-05-01".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });

    context.add_target(TargetFileDescriptor {
        file_id: 192,
        relative_path: PathBuf::from("aletheion-logi/distribution/coldchain/coldchain_monitor.lua"),
        language: LanguageKind::Lua,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-004".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Logi_Team".to_string(),
            deadline: "2026-04-20".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });

    context.add_target(TargetFileDescriptor {
        file_id: 193,
        relative_path: PathBuf::from("aletheion-logi/distribution/coldchain/coldchain_monitor.js"),
        language: LanguageKind::JavaScript,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-004".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Logi_Team".to_string(),
            deadline: "2026-04-20".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });

    context.add_target(TargetFileDescriptor {
        file_id: 211,
        relative_path: PathBuf::from("aletheion-sec/agri/indigenous/pesticide_monitoring.rs"),
        language: LanguageKind::Rust,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-002".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Treaty_Office".to_string(),
            deadline: "2026-05-01".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });

    context.add_target(TargetFileDescriptor {
        file_id: 232,
        relative_path: PathBuf::from("aletheion-env/monitoring/sensors/sensor_calibration.lua"),
        language: LanguageKind::Lua,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-003".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Env_Team".to_string(),
            deadline: "2026-04-20".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });

    context.add_target(TargetFileDescriptor {
        file_id: 233,
        relative_path: PathBuf::from("aletheion-env/monitoring/sensors/sensor_calibration.js"),
        language: LanguageKind::JavaScript,
        research_dependency: Some(ResearchDependency {
            research_gap_id: "RG-003".to_string(),
            manifest_path: PathBuf::from("tier2_research_manifest.yml"),
            owner: "Env_Team".to_string(),
            deadline: "2026-04-20".to_string(),
        }),
        readiness: FillerReadiness::BlockedByResearch,
    });
}

fn bootstrap_templates(context: &mut FillerContext) {
    context.register_template(
        LanguageKind::Rust,
        "soil_microbiome.rs.template",
        "rust/soil/soil_microbiome.rs.template",
    );
    context.register_template(
        LanguageKind::Lua,
        "soil_microbiome.lua.template",
        "lua/soil/soil_microbiome.lua.template",
    );
    context.register_template(
        LanguageKind::JavaScript,
        "mof_water_harvesting.js.template",
        "js/env/mof_water_harvesting.js.template",
    );
    context.register_template(
        LanguageKind::Rust,
        "indigenous_agriculture.rs.template",
        "rust/crops/indigenous_agriculture.rs.template",
    );
    context.register_template(
        LanguageKind::Lua,
        "coldchain_monitor.lua.template",
        "lua/logi/coldchain_monitor.lua.template",
    );
    context.register_template(
        LanguageKind::JavaScript,
        "coldchain_monitor.js.template",
        "js/logi/coldchain_monitor.js.template",
    );
    context.register_template(
        LanguageKind::Rust,
        "pesticide_monitoring.rs.template",
        "rust/sec/pesticide_monitoring.rs.template",
    );
    context.register_template(
        LanguageKind::Lua,
        "sensor_calibration.lua.template",
        "lua/env/sensor_calibration.lua.template",
    );
    context.register_template(
        LanguageKind::JavaScript,
        "sensor_calibration.js.template",
        "js/env/sensor_calibration.js.template",
    );
}

fn main() {
    let repository_root = default_repository_root();
    let template_root = default_template_root();
    let manifest_root = default_manifest_root();

    let mut context = FillerContext::new(repository_root, template_root, manifest_root);

    bootstrap_templates(&mut context);
    bootstrap_tier2_targets(&mut context);

    if let Err(err) = context.generate_all() {
        eprintln!("FillerOutputGenerator error: {}", err);
        std::process::exit(1);
    }
}
