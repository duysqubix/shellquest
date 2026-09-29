//! Shell hook generation and rc-file installation.
//!
//! `sq hook --shell <sh>` prints the hook. `--install` writes a small loader into the
//! rc file that evaluates `sq hook` at shell startup, so hook fixes ship with the binary
//! instead of living on as stale text in every player's rc file. Installing also
//! upgrades, in place, the hooks that sq <= 1.28 appended verbatim.

/// Passed as `sq tick --hook=N` by the hooks below. Hooks from older versions pass
/// nothing (0) and re-tick the previous command on an empty Enter.
pub const VERSION: u32 = 2;

/// Opening line of the block this version writes for `shell` (also used to find it).
pub fn begin(shell: &str) -> String {
    format!("# >>> shellquest hook ({shell}) >>>")
}

/// Closing line of that block.
pub fn end(shell: &str) -> String {
    format!("# <<< shellquest hook ({shell}) <<<")
}

/// The hook itself: tick once for each command that actually ran, and keep `$?`.
pub fn code(shell: &str) -> Option<String> {
    let body = match shell {
        // preexec fires only for a command that is about to run, so an empty Enter,
        // Ctrl-C at the prompt or a new shell never ticks; add-zsh-hook ignores
        // duplicates. The hook sq <= 1.28 appended (`__sq_hook`) is retired when this
        // loads (the rc file was re-sourced after an upgrade) and at every prompt (a
        // copy in a file `--install` doesn't scan ran after this), so it can't tick too.
        "zsh" => {
            r#"typeset -g __sq_cmd=
__sq_preexec() { typeset -g __sq_cmd=$1; }
__sq_precmd() {
    local exit_code=$?
    emulate -L zsh
    if (( ${+functions[__sq_hook]} )); then unfunction __sq_hook; fi
    local cmd=${__sq_cmd-}
    __sq_cmd=
    # Skip: nothing ran, space-prefixed (kept out of history on purpose), comments, sq itself.
    if [[ -n $cmd && $cmd != [[:space:]]* && $cmd != \#* && ${cmd[(w)1]} != sq ]]; then
        sq tick --hook=2 --cmd="$cmd" --cwd="$PWD" --exit-code="$exit_code"
    fi
    return $exit_code
}
() {
    emulate -L zsh
    precmd_functions=(${precmd_functions:#__sq_hook})
    if (( ${+functions[__sq_hook]} )); then unfunction __sq_hook; fi
    autoload -Uz add-zsh-hook
    add-zsh-hook preexec __sq_preexec
    add-zsh-hook precmd __sq_precmd
}"#
        }
        // Bash can't see the text of a command that stays out of history, so it ticks
        // only when the newest history entry changes. An empty Enter, Ctrl-C at the
        // prompt, a new shell, or a command dropped by ignorespace/ignoredups/HISTIGNORE
        // leaves it as it was, so none of them is mistaken for the previous command (a
        // command repeated straight away under ignoredups counts once). Where PS0 works
        // (bash >= 4.4 with promptvars on), its arithmetic sets a marker, printing
        // nothing, when a command is about to run; requiring it too ignores entries
        // that another shell's history brought in. Needs shell history. The function
        // keeps the name sq <= 1.28 used, so re-sourcing an upgraded rc file replaces
        // the old hook instead of adding a second one.
        "bash" => {
            r#"__sq_read() {
    local entry num
    entry=$(HISTTIMEFORMAT= builtin history 1)
    # `history` prints "%5d%c %s": the number, '*' or ' ', one space, the command.
    # Split without =~ so the player's BASH_REMATCH survives the prompt.
    entry=${entry#"${entry%%[![:space:]]*}"}
    num=${entry%%[!0-9]*}
    __sq_cmd=
    if [[ -n $num ]]; then __sq_cmd=${entry:${#num}+2}; fi
    __sq_key="$num $__sq_cmd"
}
__sq_hook() {
    local exit_code=$? ran=${__sq_ran-} prev=${__sq_last-}
    __sq_ran=
    __sq_read
    __sq_last=$__sq_key
    # Skip: no new entry, space-prefixed (kept out of history on purpose), comments, sq itself.
    if [[ -n $prev && $__sq_key != "$prev" && -n $__sq_cmd ]] &&
       [[ $__sq_cmd != [[:space:]]* && $__sq_cmd != \#* && ${__sq_cmd%%[[:space:]]*} != sq ]]; then
        if [[ ${PS0-} != *__sq_ran* || -n $ran ]] || ! shopt -q promptvars; then
            sq tick --hook=2 --cmd="$__sq_cmd" --cwd="$PWD" --exit-code="$exit_code"
        fi
    fi
    return $exit_code
}
# Runs last: an entry that a later prompt command read in from another shell's
# history (history -n) becomes the baseline instead of looking like a new command.
# (HISTCMD can't tell whether that happened: bash 3.2 doesn't update it.)
__sq_mark() {
    local exit_code=$?
    __sq_read
    __sq_last=$__sq_key
    return $exit_code
}
if [[ $(declare -p PROMPT_COMMAND 2>/dev/null) == "declare -a"* ]]; then
    [[ " ${PROMPT_COMMAND[*]} " == *" __sq_hook "* ]] || PROMPT_COMMAND=(__sq_hook "${PROMPT_COMMAND[@]}")
    [[ " ${PROMPT_COMMAND[*]} " == *" __sq_mark "* ]] || PROMPT_COMMAND+=(__sq_mark)
else
    [[ ${PROMPT_COMMAND-} == *__sq_hook* ]] || PROMPT_COMMAND="__sq_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
    [[ $PROMPT_COMMAND == *__sq_mark* ]] || PROMPT_COMMAND+=$'\n__sq_mark'
fi
if (( BASH_VERSINFO[0] * 100 + BASH_VERSINFO[1] >= 404 )) && shopt -q promptvars &&
   [[ ${PS0-} != *__sq_ran* ]]; then
    PS0="${PS0-}"'${PS0:0:$((__sq_ran=1,0))}'
fi"#
        }
        // fish_postexec fires only for commands that ran; fish already keeps
        // space-prefixed commands out of history, so they are skipped here too.
        "fish" => {
            r#"function __sq_hook --on-event fish_postexec
    set -l exit_code $status
    set -l cmd $argv[1]
    # Skip: nothing ran, space-prefixed, comments, sq itself.
    string match -qr '^(\s|#|sq(\s|$)|$)' -- $cmd; and return
    sq tick --hook=2 --cmd=$cmd --cwd=$PWD --exit-code=$exit_code
end"#
        }
        _ => return None,
    };
    Some(format!(
        "{}\n# shellquest (sq): ticks once per command that ran; see `sq help hook`.\n{}\n{}\n",
        begin(shell),
        body,
        end(shell)
    ))
}

/// The rc-file block `--install` writes: evaluate the hook this binary prints.
pub fn loader(shell: &str) -> Option<String> {
    let line = match shell {
        "zsh" => "command -v sq >/dev/null 2>&1 && eval \"$(sq hook --shell zsh)\"",
        "bash" => "command -v sq >/dev/null 2>&1 && eval \"$(sq hook --shell bash)\"",
        "fish" => "status is-interactive; and command -q sq; and sq hook --shell fish | source",
        _ => return None,
    };
    Some(format!(
        "{}\n# shellquest (sq): the hook code ships with sq; `sq hook --shell {} --install` manages this block.\n{}\n{}\n",
        begin(shell),
        shell,
        line,
        end(shell)
    ))
}

/// Every hook sq <= 1.28 printed for `shell` (`sq hook --shell X >> rc`), line by
/// line. Only exact copies are upgraded; anything else is left for the player.
fn legacy_hooks(shell: &str) -> Vec<Vec<String>> {
    let (header_v1, opening, prologue, self_skip, closing, registration): (
        &str,
        &str,
        [&str; 2],
        [&str; 2],
        &str,
        Option<&str>,
    ) = match shell {
        "bash" => (
            "# sq shell hook — add to your ~/.bashrc",
            "__sq_hook() {",
            [
                "    local exit_code=$?",
                "    local cmd=$(HISTTIMEFORMAT= history 1 | sed 's/^ *[0-9]* *//')",
            ],
            [
                "    local first=$(printf '%s' \"$cmd\" | awk '{print $1}')",
                "    [ \"$first\" = \"sq\" ] && return",
            ],
            "}",
            Some("PROMPT_COMMAND=\"__sq_hook;$PROMPT_COMMAND\""),
        ),
        "zsh" => (
            "# sq shell hook — add to your ~/.zshrc",
            "__sq_hook() {",
            ["    local exit_code=$?", "    local cmd=$(fc -ln -1)"],
            [
                "    local first=${cmd[(w)1]}",
                "    [[ \"$first\" == \"sq\" ]] && return",
            ],
            "}",
            Some("precmd_functions+=(__sq_hook)"),
        ),
        "fish" => (
            "# sq shell hook — add to your ~/.config/fish/config.fish",
            "function __sq_hook --on-event fish_postexec",
            ["    set -l cmd $argv[1]", "    set -l exit_code $status"],
            [
                "    set -l first (string split -m1 ' ' $cmd)[1]",
                "    [ \"$first\" = \"sq\" ]; and return",
            ],
            "end",
            None,
        ),
        _ => return Vec::new(),
    };
    let header = "# shellquest (sq) — passive terminal RPG hook";
    let tick = "    sq tick --cmd \"$cmd\" --cwd \"$PWD\" --exit-code \"$exit_code\"";
    let assemble = |head: &str, body: &[&str]| {
        let mut lines = vec![head, opening];
        lines.extend(prologue);
        lines.extend(body);
        lines.push(closing);
        lines.extend(registration);
        lines.into_iter().map(str::to_string).collect::<Vec<_>>()
    };
    let backgrounded = format!("{tick} &");
    let silenced = format!("{tick} 2>/dev/null &");
    vec![
        // v1.28: synchronous, skips sq itself
        assemble(header, &[self_skip[0], self_skip[1], tick]),
        // synchronous
        assemble(header, &[tick]),
        // v1.1: backgrounded
        assemble(header, &[&backgrounded, "    disown 2>/dev/null"]),
        // v1.0: backgrounded, stderr discarded
        assemble(header_v1, &[&silenced, "    disown 2>/dev/null"]),
    ]
}

/// A hook block in an rc file, as a range of line indices.
struct Block {
    start: usize,
    end: usize, // exclusive
}

/// Recognized hook blocks for `shell`, or `Err` when hook text appears in a form
/// this version doesn't recognize (hand-edited, partial markers), in which case
/// nothing may be touched.
fn find_blocks(shell: &str, lines: &[&str]) -> Result<Vec<Block>, ()> {
    let bare = |l: &str| l.trim_end_matches(['\n', '\r']).trim_end().to_string();
    let legacy = legacy_hooks(shell);
    let mut ours = Vec::new();
    let mut covered = vec![false; lines.len()];
    let mut i = 0;
    while i < lines.len() {
        let line = bare(lines[i]);
        if let Some(owner) = line
            .strip_prefix("# >>> shellquest hook (")
            .and_then(|rest| rest.strip_suffix(") >>>"))
        {
            let close = format!("# <<< shellquest hook ({owner}) <<<");
            let stop = (i + 1..lines.len())
                .find(|&j| bare(lines[j]) == close)
                .ok_or(())?;
            // Another block opening inside this one means a marker went missing:
            // pairing across it could delete the player's own lines.
            if lines[i + 1..stop]
                .iter()
                .any(|l| bare(l).starts_with("# >>> shellquest hook ("))
            {
                return Err(());
            }
            covered[i..=stop].iter_mut().for_each(|c| *c = true);
            if owner == shell {
                ours.push(Block {
                    start: i,
                    end: stop + 1,
                });
            }
            i = stop + 1;
            continue;
        }
        if let Some(hook) = legacy.iter().find(|hook| {
            hook.len() <= lines.len() - i
                && hook
                    .iter()
                    .enumerate()
                    .all(|(k, l)| bare(lines[i + k]) == *l)
        }) {
            covered[i..i + hook.len()]
                .iter_mut()
                .for_each(|c| *c = true);
            ours.push(Block {
                start: i,
                end: i + hook.len(),
            });
            i += hook.len();
            continue;
        }
        i += 1;
    }
    let stray = lines
        .iter()
        .enumerate()
        .any(|(n, l)| !covered[n] && (l.contains("__sq_hook") || l.contains("shellquest hook (")));
    if stray {
        Err(())
    } else {
        Ok(ours)
    }
}

/// What installing into one rc file would do.
#[derive(Debug, PartialEq)]
pub enum RcPlan {
    /// No hook for this shell in the file.
    Absent,
    /// Hook blocks found. `current`: exactly the current loader and nothing else.
    /// `with_loader`: the file with them replaced by one loader, where the first one
    /// was; every other byte is unchanged.
    Present { current: bool, with_loader: String },
    /// Hook text this version doesn't recognize: leave the file alone.
    Unrecognized,
}

pub fn plan_rc(shell: &str, contents: &str) -> RcPlan {
    let Some(loader) = loader(shell) else {
        return RcPlan::Unrecognized;
    };
    let lines: Vec<&str> = contents.split_inclusive('\n').collect();
    let Ok(blocks) = find_blocks(shell, &lines) else {
        return RcPlan::Unrecognized;
    };
    if blocks.is_empty() {
        return RcPlan::Absent;
    }
    let current = blocks.len() == 1 && lines[blocks[0].start..blocks[0].end].concat() == loader;
    let mut with_loader = String::with_capacity(contents.len());
    for (n, line) in lines.iter().enumerate() {
        match blocks.iter().position(|b| (b.start..b.end).contains(&n)) {
            None => with_loader.push_str(line),
            Some(0) if n == blocks[0].start => with_loader.push_str(&loader),
            Some(_) => {}
        }
    }
    RcPlan::Present {
        current,
        with_loader,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_text(shell: &str, variant: usize) -> String {
        legacy_hooks(shell)[variant].join("\n") + "\n"
    }

    #[test]
    fn generated_hooks_are_marked_versioned_and_fork_free() {
        for shell in ["zsh", "bash", "fish"] {
            let code = code(shell).unwrap();
            assert!(
                code.starts_with(&begin(shell)) && code.trim_end().ends_with(&end(shell)),
                "{shell}"
            );
            assert!(
                code.contains("--hook=2") && code.contains("--cmd="),
                "{shell}"
            );
            assert!(
                !code.contains("sed ") && !code.contains("awk "),
                "{shell}: no forks per prompt"
            );
        }
        assert!(code("tcsh").is_none() && loader("tcsh").is_none());
        assert_eq!(VERSION, 2, "the hooks hard-code --hook=2");
    }

    #[test]
    fn zsh_hook_is_option_proof_and_retires_the_legacy_handler() {
        let zsh = code("zsh").unwrap();
        assert!(zsh.contains("emulate -L zsh"));
        assert!(zsh.contains("precmd_functions=(${precmd_functions:#__sq_hook})"));
        assert!(zsh.contains("return $exit_code"));
        assert!(code("bash").unwrap().contains("return $exit_code"));
    }

    #[test]
    fn there_are_four_historical_hooks_per_shell() {
        for shell in ["zsh", "bash", "fish"] {
            let hooks = legacy_hooks(shell);
            assert_eq!(hooks.len(), 4, "{shell}");
            assert!(hooks
                .iter()
                .all(|h| h.iter().any(|l| l.contains("sq tick"))));
        }
    }

    #[test]
    fn empty_and_unrelated_rc_files_have_no_hook() {
        assert_eq!(plan_rc("zsh", ""), RcPlan::Absent);
        assert_eq!(
            plan_rc("zsh", "export EDITOR=vim\nalias ll='ls -l'\n"),
            RcPlan::Absent
        );
    }

    #[test]
    fn the_current_loader_is_left_alone() {
        let rc = format!("export EDITOR=vim\n{}", loader("zsh").unwrap());
        assert_eq!(
            plan_rc("zsh", &rc),
            RcPlan::Present {
                current: true,
                with_loader: rc.clone()
            }
        );
    }

    #[test]
    fn every_historical_hook_is_upgraded_in_place() {
        for shell in ["zsh", "bash", "fish"] {
            for variant in 0..4 {
                let rc = format!("export A=1\n\n{}export B=2\n", legacy_text(shell, variant));
                let RcPlan::Present {
                    current: false,
                    with_loader,
                } = plan_rc(shell, &rc)
                else {
                    panic!("{shell} variant {variant} not recognized");
                };
                assert_eq!(
                    with_loader,
                    format!("export A=1\n\n{}export B=2\n", loader(shell).unwrap()),
                    "{shell} variant {variant}"
                );
                assert!(matches!(
                    plan_rc(shell, &with_loader),
                    RcPlan::Present { current: true, .. }
                ));
            }
        }
    }

    #[test]
    fn untouched_bytes_survive_an_upgrade() {
        let rc = format!("export A=1\r\n{}export B=2", legacy_text("zsh", 0));
        let RcPlan::Present { with_loader, .. } = plan_rc("zsh", &rc) else {
            panic!("not found");
        };
        assert!(
            with_loader.starts_with("export A=1\r\n"),
            "CRLF elsewhere is kept"
        );
        assert!(
            with_loader.ends_with("export B=2"),
            "no trailing newline is added"
        );
    }

    #[test]
    fn duplicate_hooks_in_one_file_collapse_to_one_loader() {
        let rc = format!(
            "{}{}\n{}",
            legacy_text("zsh", 0),
            legacy_text("zsh", 2),
            code("zsh").unwrap()
        );
        let RcPlan::Present { with_loader, .. } = plan_rc("zsh", &rc) else {
            panic!("not found");
        };
        assert_eq!(with_loader.matches(&begin("zsh")).count(), 1);
        assert!(!with_loader.contains("__sq_hook"));
    }

    #[test]
    fn modified_or_partial_hooks_are_not_touched() {
        let edited = legacy_text("zsh", 0).replace("fc -ln -1", "fc -ln -1 | tr -d x");
        assert_eq!(plan_rc("zsh", &edited), RcPlan::Unrecognized);
        let indented: String = legacy_text("zsh", 1)
            .lines()
            .map(|l| format!("  {l}\n"))
            .collect();
        assert_eq!(plan_rc("zsh", &indented), RcPlan::Unrecognized);
        assert_eq!(
            plan_rc("zsh", "# call __sq_hook myself\n"),
            RcPlan::Unrecognized
        );
        assert_eq!(
            plan_rc("zsh", &format!("{}\nstuff\n", begin("zsh"))),
            RcPlan::Unrecognized
        );
    }

    #[test]
    fn a_block_missing_its_closing_marker_is_not_touched() {
        let broken = format!(
            "{}alias keep=1\n{}",
            begin("zsh") + "\n",
            loader("zsh").unwrap()
        );
        assert_eq!(plan_rc("zsh", &broken), RcPlan::Unrecognized);
    }

    #[test]
    fn another_shells_block_is_not_ours() {
        let rc = format!("# shared\n{}", code("bash").unwrap());
        assert_eq!(plan_rc("zsh", &rc), RcPlan::Absent);
        assert!(matches!(
            plan_rc("bash", &rc),
            RcPlan::Present { current: false, .. }
        ));
    }
}
