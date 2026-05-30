use std::io::Write;

use tinyharness_lib::skill::{SkillRegistry, SkillSource};
use tinyharness_ui::output::Output;

use crate::commands::registry::{CommandContext, CommandResult};
use tinyharness_ui::style::*;

// ── Helper for /use and /skill use ──────────────────────────────────────────

pub fn handle_skill_use(name: &str, ctx: &mut CommandContext) -> Result<CommandResult, String> {
    // Validate that the skill exists and is user-invocable
    match ctx.skill_registry.get(name) {
        Some(skill) if !skill.user_invocable => {
            let _ = writeln!(
                ctx.output,
                "{ORANGE}Skill '{name}' is not user-invocable.{RESET} It can only be activated by the model.",
            );
            Ok(CommandResult::Ok)
        }
        Some(_) => {
            let name = name.to_string();
            Ok(CommandResult::SkillUse(name))
        }
        None => {
            let available = ctx
                .skill_registry
                .skills
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(
                ctx.output,
                "{RED}Skill '{name}' not found.{RESET} Use {BOLD}/skills{RESET} to list available skills.",
            );
            if !available.is_empty() {
                let _ = writeln!(
                    ctx.output,
                    "{GRAY}Available skills: {CYAN}{available}{RESET}",
                );
            }
            Ok(CommandResult::Ok)
        }
    }
}

// ── Display functions ────────────────────────────────────────────────────────

/// List all available skills, marking active ones.
pub fn execute_list(out: &mut Output, registry: &SkillRegistry, active_skills: &[String]) {
    if registry.skills.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "{ORANGE}No skills found.{RESET}");
        let _ = writeln!(
            out,
            "{GRAY}Create skills in ~/.tinyharness/skills/<name>/SKILL.md{RESET}",
        );
        let _ = writeln!(
            out,
            "{GRAY}or in .tinyharness/skills/<name>/SKILL.md (project-local){RESET}",
        );
        let _ = writeln!(out);
        return;
    }

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{BOLD}╭─ Available Skills ───────────────────────────╮{RESET}",
    );

    for skill in &registry.skills {
        let source_label = match skill.source {
            SkillSource::Personal => format!("{GRAY}~{RESET}"),
            SkillSource::Project => format!("{GRAY}.{RESET}"),
        };
        let auto_label = if skill.disable_model_invocation {
            format!("{GRAY}manual only{RESET}")
        } else {
            format!("{GREEN}auto{RESET}")
        };
        let active_marker = if active_skills
            .iter()
            .any(|s| s.eq_ignore_ascii_case(&skill.name))
        {
            format!("{GREEN}●{RESET}")
        } else {
            format!("{GRAY}○{RESET}")
        };

        let _ = writeln!(
            out,
            "{BOLD}│{RESET} {active_marker} {BOLD}{CYAN}{}{RESET} — {description}  {source_label}[{auto_label}]",
            skill.name,
            description = skill.description,
        );
    }

    let _ = writeln!(
        out,
        "{BOLD}╰──────────────────────────────────────────────╯{RESET}",
    );

    if !active_skills.is_empty() {
        let _ = writeln!(
            out,
            "  {GRAY}Active: {GREEN}{}{RESET}",
            active_skills.join(", "),
        );
    }

    let _ = writeln!(out);
}

/// Show details for a specific skill.
pub fn execute_show<W: Write>(
    registry: &SkillRegistry,
    name: &str,
    active_skills: &[String],
    stdout: &mut W,
) {
    let skill = match registry.get(name) {
        Some(s) => s,
        None => {
            writeln!(
                stdout,
                "{RED}Skill '{name}' not found.{RESET} Use {BOLD}/skills{RESET} to list available skills.",
            )
            .unwrap_or(());
            return;
        }
    };

    let source_label = match skill.source {
        SkillSource::Personal => "Personal (~/.tinyharness/skills/)",
        SkillSource::Project => "Project (.tinyharness/skills/)",
    };

    writeln!(stdout).unwrap_or(());
    writeln!(
        stdout,
        "{BOLD}╭─ Skill: {CYAN}{}{BOLD} ──────────────────────────╮{RESET}",
        skill.name,
    )
    .unwrap_or(());
    writeln!(
        stdout,
        "{BOLD}│{RESET}   {BOLD}Description:{RESET} {}",
        skill.description,
    )
    .unwrap_or(());
    writeln!(
        stdout,
        "{BOLD}│{RESET}   {BOLD}Source:{RESET} {source_label}",
    )
    .unwrap_or(());
    writeln!(
        stdout,
        "{BOLD}│{RESET}   {BOLD}Path:{RESET} {}",
        skill.path.display(),
    )
    .unwrap_or(());

    if let Some(hint) = &skill.argument_hint {
        writeln!(
            stdout,
            "{BOLD}│{RESET}   {BOLD}Argument hint:{RESET} {hint}",
        )
        .unwrap_or(());
    }

    if let Some(compat) = &skill.compatibility {
        writeln!(
            stdout,
            "{BOLD}│{RESET}   {BOLD}Compatibility:{RESET} {compat}",
        )
        .unwrap_or(());
    }

    if let Some(lic) = &skill.license {
        writeln!(stdout, "{BOLD}│{RESET}   {BOLD}License:{RESET} {lic}",).unwrap_or(());
    }

    let auto_label = if skill.disable_model_invocation {
        format!("{ORANGE}Manual invocation only (model cannot auto-invoke){RESET}")
    } else {
        format!("{GREEN}Model can auto-invoke this skill{RESET}")
    };
    writeln!(
        stdout,
        "{BOLD}│{RESET}   {BOLD}Auto-invoke:{RESET} {auto_label}",
    )
    .unwrap_or(());

    let active_label = if active_skills
        .iter()
        .any(|s| s.eq_ignore_ascii_case(&skill.name))
    {
        format!("{GREEN}● Active{RESET}")
    } else {
        format!("{GRAY}○ Inactive{RESET}")
    };
    writeln!(
        stdout,
        "{BOLD}│{RESET}   {BOLD}Status:{RESET} {active_label}",
    )
    .unwrap_or(());

    writeln!(
        stdout,
        "{BOLD}╰──────────────────────────────────────────────╯{RESET}",
    )
    .unwrap_or(());

    // Show the skill content
    if !skill.content.is_empty() {
        writeln!(stdout).unwrap_or(());
        writeln!(stdout, "{BOLD}Skill instructions:{RESET}").unwrap_or(());
        writeln!(stdout).unwrap_or(());
        for line in skill.content.lines() {
            writeln!(stdout, "  {line}").unwrap_or(());
        }
        writeln!(stdout).unwrap_or(());
    }
}

/// Print help for the /skill command.
pub fn execute_help(out: &mut Output) {
    let _ = writeln!(out);
    let _ = writeln!(out, "{BOLD}Skill management — subcommands:{RESET}");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {CYAN}{0:<16}{RESET} List all available skills",
        "list",
    );
    let _ = writeln!(
        out,
        "  {CYAN}{0:<16}{RESET} Show details and content of a skill",
        "<name>",
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{GRAY}Tip:{RESET} Use {BOLD}/use <name>{RESET} to activate a skill, {BOLD}/unload <name>{RESET} to deactivate it.",
    );
    let _ = writeln!(
        out,
        "      Skills are loaded from {BOLD}~/.tinyharness/skills/<name>/SKILL.md{RESET} (personal)",
    );
    let _ = writeln!(
        out,
        "      and {BOLD}.tinyharness/skills/<name>/SKILL.md{RESET} (project-local).",
    );
    let _ = writeln!(out);
}
