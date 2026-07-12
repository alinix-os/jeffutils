pub mod pipeline;

use crate::parser::{ExpandedPipeline, ListOp};
use crate::shell::ShellState;

/// Runs a fully parsed `CommandList` (a line possibly containing `;`, `&&`,
/// `||`, and multiple pipelines), applying short-circuit semantics based on
/// each pipeline's exit status. Builtins are dispatched in-process via
/// `handle_builtin` so `cd /tmp && ls` etc. behave correctly.
pub fn run_command_list(
    state: &mut ShellState,
    list: &crate::parser::CommandList,
    heredoc_bodies: &[Option<String>],
) {
    let mut prev_op: Option<ListOp> = None;

    for (i, (andor, op)) in list.items.iter().enumerate() {
        let should_run = match prev_op {
            None => true,
            Some(ListOp::Seq) => true,
            Some(ListOp::And) => state.last_exit_status == 0,
            Some(ListOp::Or) => state.last_exit_status != 0,
        };

        if should_run {
            let heredoc = heredoc_bodies.get(i).and_then(|o| o.as_deref());
            run_and_or(state, andor, heredoc);
        }

        prev_op = *op;
    }
}

fn run_and_or(state: &mut ShellState, andor: &crate::parser::AndOrList, heredoc: Option<&str>) {
    // Assignment-only pipeline, e.g. `FOO=bar` with no command: set the var
    // and don't spawn anything.
    if andor.pipeline.commands.len() == 1 {
        let cmd = &andor.pipeline.commands[0];
        if cmd.args.is_empty() && cmd.redirects.is_empty() {
            if let Some((name, value)) = ShellState::as_assignment(&cmd.program) {
                let expanded_value = state.expand_word_single(&crate::parser::Word::literal(value));
                state.set_var(&name, &expanded_value);
                state.last_exit_status = 0;
                return;
            }
        }
    }

    let expanded: ExpandedPipeline = state.expand_pipeline(&andor.pipeline, heredoc);
    if expanded.commands.is_empty() {
        return;
    }

    // Single command, no redirects/pipe: try builtins first (in-process).
    if expanded.commands.len() == 1 && expanded.commands[0].redirects.is_empty() {
        let cmd = &expanded.commands[0];
        let mut argv = vec![cmd.program.clone()];
        argv.extend(cmd.args.clone());
        if let Some(status) = crate::builtin::handle_builtin(&argv, state) {
            state.last_exit_status = status;
            return;
        }

        // User-defined shell functions win over external programs of the
        // same name (e.g. a `proj()` shortcut should shadow /usr/bin/proj).
        if state.functions.contains_key(&cmd.program) {
            state.last_exit_status = state.call_function(&cmd.program, &cmd.args);
            return;
        }

        // Not a builtin and not on PATH: if `.jshrc` sourced real bash
        // scripts (e.g. nvm.sh), retry the command through bash so
        // functions defined there (`nvm`, ...) still work.
        if !crate::builtin::is_executable(&cmd.program) {
            if let Some(status) = state.try_bash_fallback(&cmd.program, &cmd.args) {
                state.last_exit_status = status;
                return;
            }
        }
    }

    if andor.background {
        let pid = pipeline::spawn_detached(expanded);
        if let Some(pid) = pid {
            eprintln!("[bg] {}", pid);
        }
        state.last_exit_status = 0;
        return;
    }

    state.last_exit_status = pipeline::execute_with(expanded, state.quiet_errors);
}
