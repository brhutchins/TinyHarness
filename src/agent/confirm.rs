// ── Shared Tool Confirmation Logic ─────────────────────────────────────────
//
// The decision tree for whether a tool call is approved is identical in both
// CLI and TUI loops. The only difference is what happens at the "ask user"
// step — CLI uses an interactive terminal prompt, TUI sends a channel event.
//
// This module extracts the pure decision logic so both loops share the same
// branching, and each only needs to implement the I/O part.

use tinyharness_lib::config::AutoAcceptMode;
use tinyharness_lib::provider::ToolCall;

use super::safety::is_safe_command;

/// Decision about whether a tool call is allowed to proceed.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationDecision {
    /// The tool call is automatically approved (no user interaction needed).
    /// `auto_accepted` indicates whether this came from auto-accept mode
    /// (affects display: "Executing..." vs "(auto-accepted)").
    AutoApproved { auto_accepted: bool },

    /// The tool call needs explicit user confirmation.
    NeedsConfirmation,

    /// The tool call was denied (e.g., by a previous user response).
    /// This variant is not produced by `decide_tool_confirmation` directly,
    /// but is useful for callers that receive a "no" from the user.
    Denied,
}

/// Determine whether a tool call should be approved, needs user confirmation,
/// or should be denied.
///
/// This is pure logic — no I/O. The caller is responsible for implementing
/// the user interaction when `NeedsConfirmation` is returned.
///
/// The logic follows these rules:
/// 1. Read-only tools (no confirmation needed) → `AutoApproved { auto_accepted: false }`
/// 2. Auto-accept mode (Safe) → `AutoApproved { auto_accepted: true }` for most tools,
///    but `run` commands that aren't safe still need confirmation
/// 3. Auto-accept mode (All) → `AutoApproved { auto_accepted: true }` for everything
/// 4. Everything else → `NeedsConfirmation`
pub fn decide_tool_confirmation(
    call: &ToolCall,
    auto_accept_mode: AutoAcceptMode,
    safe_commands: &[String],
    denied_commands: &[String],
    needs_confirmation: bool,
) -> ConfirmationDecision {
    // Read-only tools: always approved, never "auto-accepted"
    if !needs_confirmation {
        return ConfirmationDecision::AutoApproved {
            auto_accepted: false,
        };
    }

    match auto_accept_mode {
        AutoAcceptMode::All => {
            // Auto-accept all mode: approve everything without prompting
            ConfirmationDecision::AutoApproved {
                auto_accepted: true,
            }
        }
        AutoAcceptMode::Safe => {
            // Auto-accept safe mode: approve most tools, but run commands need checks
            if call.function.name == "run" {
                if let Some(cmd_value) = call.function.arguments.get("command")
                    && let Some(cmd_str) = cmd_value.as_str()
                    && is_safe_command(cmd_str, safe_commands, denied_commands)
                {
                    return ConfirmationDecision::AutoApproved {
                        auto_accepted: true,
                    };
                }
                // Unsafe run command — still needs confirmation even in safe auto-accept mode
                ConfirmationDecision::NeedsConfirmation
            } else {
                // Other destructive tools can be auto-accepted
                ConfirmationDecision::AutoApproved {
                    auto_accepted: true,
                }
            }
        }
        AutoAcceptMode::Off => {
            // Auto-accept off: everything needs confirmation
            ConfirmationDecision::NeedsConfirmation
        }
    }
}
