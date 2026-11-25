//! Execute the alpha install and tutorial journeys in a clean container.

use serde_json::Value;
use std::collections::{hash_map::Entry, HashMap};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const INSTALL_PAGE: &str = "src/content/docs/fsl/guides/set-up-your-environment.md";
const INSTALL_STEPS: &str = "install-test/steps.md";
const TUTORIAL_DIR: &str = "src/content/docs/fsl/tutorials";
const REFINEMENT_PAGE: &str = "src/content/docs/fsl/learn/refinement.md";
const MODEL_GENERATOR_PREFIX: &str = "node tools/model.mjs ";
const WRITE_A_MODEL_PAGE: &str = "src/content/docs/fsl/guides/write-a-model.md";

#[derive(Debug, PartialEq, Eq)]
enum OutputView {
    Summary,
    Fields(Vec<String>),
}

struct Container(String);

impl Drop for Container {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "-f", &self.0])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[derive(Debug)]
struct Example {
    page: PathBuf,
    line: usize,
    id: String,
    system: String,
    kind: ExampleKind,
    show: Vec<String>,
    view: OutputView,
}

#[derive(Debug, PartialEq, Eq)]
enum ExampleKind {
    Command,
    Output,
    Source,
}

#[derive(Debug)]
struct Case {
    rule: String,
    command: String,
    file: String,
    status: String,
    expected_exit: i32,
}

#[derive(Debug)]
struct LiteralCommand {
    page: PathBuf,
    system: String,
    command: String,
    expected: String,
    expected_exit: i32,
    line: usize,
}

#[derive(Debug)]
struct ParseIssue {
    page: PathBuf,
    line: usize,
    reason: String,
}

#[derive(Debug)]
struct AllowedSkip {
    page: PathBuf,
    line: usize,
    reason: String,
}

#[derive(Debug)]
struct FileExcerpt {
    page: PathBuf,
    line: usize,
    system: String,
    text: String,
}

fn main() {
    if env::args().any(|argument| argument == "--help" || argument == "-h") {
        println!(
            "alpha-journey-gate --docs PATH --examples PATH [--fsl-binary PATH] [--released-binary PATH] [--base-image IMAGE]\n\
             Acceptance defaults to ubuntu:26.04; pass --base-image ubuntu:24.04 for local candidates built on Ubuntu 24.04.\n\
             The install and tutorial routes are covered. The install page's source-build route is not covered."
        );
        return;
    }
    if let Err(error) = run() {
        eprintln!("alpha journey gate: {error}");
        std::process::exit(1);
    }
}

#[derive(Default)]
struct GateStats {
    checked: usize,
    tutorial_commands: usize,
    failures: usize,
    skips: usize,
}

fn run() -> Result<(), String> {
    let mut stats = GateStats::default();
    let result = run_inner(&mut stats);
    if let Err(error) = result {
        stats.failures += 1;
        fail_line(Path::new("gate"), "journey setup", "completed", &error);
    }
    if stats.tutorial_commands == 0 {
        stats.failures += 1;
        fail_line(
            Path::new("gate"),
            "tutorial coverage",
            "at least one tutorial command checked",
            "zero commands checked",
        );
    }
    println!(
        "journey gate: {} checked, {} failed, {} skipped",
        stats.checked, stats.failures, stats.skips
    );
    if stats.tutorial_commands == 0 {
        return Err("zero tutorial commands were checked".to_owned());
    }
    if stats.failures != 0 {
        return Err("one or more install/tutorial checks failed".to_owned());
    }
    Ok(())
}

fn run_inner(stats: &mut GateStats) -> Result<(), String> {
    let (docs, examples, candidate, released_binary, base_image) = arguments()?;
    let docs_commit = git_commit(&docs)?;
    let examples_commit = git_commit(&examples)?;
    println!("docs commit: {docs_commit}");
    println!("examples commit: {examples_commit}");
    let install_page_path = docs.join(INSTALL_PAGE);
    let install_page = read(&install_page_path)?;
    let install_source = read(&examples.join(INSTALL_STEPS))?;
    let install_steps = step_blocks(&install_source);
    let selected_steps = install_step_ids(&install_page, candidate.is_some());
    let install_script =
        assemble_install_script(&selected_steps, &install_steps, &install_page_path);
    let expected_version_output = expected_version_output(&install_page);
    let expected_version = expected_version_output
        .as_ref()
        .and_then(|lines| lines.first().cloned())
        .or_else(|| expected_version(&install_page))
        .ok_or_else(|| {
            format!(
                "{} does not state the expected fsl version",
                install_page_path.display()
            )
        })?;
    if candidate.is_some() {
        stats.skips += 1;
        println!("SKIP released-artifact check: candidate binary");
    } else if let Some(released_binary) = released_binary.as_deref() {
        stats.checked += 1;
        if let Err(error) = check_released_binary(
            &examples,
            released_binary,
            &install_page_path,
            &expected_version,
            expected_version_output.as_deref(),
        ) {
            record_failure(
                stats,
                &install_page_path,
                "released artifact startup on ubuntu:26.04 and GLIBC <= 2.43 without GLIBC_PRIVATE",
                &expected_version,
                &error,
            );
        }
    }

    let tutorial_pages = tutorial_pages(&docs, &docs.join(TUTORIAL_DIR))?;
    let mut examples_list = Vec::new();
    let mut literal_commands = Vec::new();
    let mut file_excerpts = Vec::new();
    let mut step_files = Vec::new();
    for page in &tutorial_pages {
        let content = match read(&docs.join(page)) {
            Ok(content) => content,
            Err(error) => {
                record_failure(
                    stats,
                    &docs.join(page),
                    "read tutorial page",
                    "readable page",
                    &error,
                );
                continue;
            }
        };
        let (page_examples, example_issues) = page_examples(page, &content);
        for issue in example_issues {
            record_parse_issue(stats, &docs, issue);
        }
        let literal_system = page_examples
            .first()
            .map(|example| example.system.as_str())
            .unwrap_or("");
        let (page_literals, skips, page_excerpts, literal_issues) =
            page_literal_commands(page, literal_system, &content);
        for skip in skips {
            stats.skips += 1;
            println!(
                "SKIP {}:{} {}",
                docs.join(&skip.page).display(),
                skip.line,
                skip.reason
            );
        }
        for issue in literal_issues {
            record_parse_issue(stats, &docs, issue);
        }
        for issue in unparsed_fences(page, &content) {
            record_parse_issue(stats, &docs, issue);
        }
        for (line, path) in page_step_files(&content) {
            step_files.push((page.clone(), line, path));
        }
        literal_commands.extend(page_literals);
        file_excerpts.extend(page_excerpts);
        examples_list.extend(page_examples);
    }
    for (page, line, path) in step_files {
        stats.checked += 1;
        let file = examples.join(&path);
        if fs::read_to_string(&file).is_ok_and(|text| !text.trim().is_empty()) {
            println!(
                "PASS {}:{} | steps file {} exists in the examples repository",
                docs.join(&page).display(),
                line,
                path
            );
        } else {
            record_failure(
                stats,
                &docs.join(&page),
                &format!("steps file fence at line {line}"),
                "a non-empty file in the examples repository",
                &format!("{} is missing or empty", file.display()),
            );
        }
    }
    for excerpt in file_excerpts {
        stats.checked += 1;
        match excerpt_matches(&examples, &excerpt.system, &excerpt.text) {
            Ok(true) => println!(
                "PASS {}:{} | file excerpt appears in examples/{}/**",
                docs.join(&excerpt.page).display(),
                excerpt.line,
                excerpt.system
            ),
            Ok(false) => record_failure(
                stats,
                &docs.join(&excerpt.page),
                &format!("text fence at line {}", excerpt.line),
                "excerpt lines occur contiguously in an examples file",
                "no matching file excerpt",
            ),
            Err(error) => record_failure(
                stats,
                &docs.join(&excerpt.page),
                &format!("text fence at line {}", excerpt.line),
                "searchable examples directory",
                &error,
            ),
        }
    }
    if examples_list.is_empty() && literal_commands.is_empty() {
        record_failure(
            stats,
            &docs.join(TUTORIAL_DIR),
            "tutorial coverage",
            "at least one command",
            "zero commands parsed",
        );
    }

    let mut cases_by_system = HashMap::new();
    for example in &examples_list {
        if let Entry::Vacant(entry) = cases_by_system.entry(example.system.clone()) {
            let manifest_path = examples
                .join("examples")
                .join(&example.system)
                .join("example.toml");
            match read(&manifest_path) {
                Ok(manifest) => {
                    entry.insert(manifest_cases(&manifest));
                }
                Err(error) => record_failure(
                    stats,
                    &example.page,
                    &format!("load {}", manifest_path.display()),
                    "readable manifest",
                    &error,
                ),
            }
        }
    }

    if selected_steps.is_empty() {
        record_failure(
            stats,
            &install_page_path,
            "install steps",
            "at least one selected step",
            "none selected",
        );
    }
    let install_script = match install_script {
        Ok(script) => Some(script),
        Err(error) => {
            record_failure(
                stats,
                &install_page_path,
                "assemble install route",
                "all referenced steps exist",
                &error,
            );
            None
        }
    };
    let container = start_container(&examples, candidate.as_deref(), &base_image)?;
    if let Some(script) = install_script {
        stats.checked += 1;
        match exec(&container, "bash", &["-e", "-c", &script]) {
            Ok(output) if output.status.success() => {
                println!("PASS {} | install route", install_page_path.display())
            }
            Ok(output) => {
                let text = output_text(&output);
                let step = failed_step(&text).unwrap_or_else(|| "install".to_owned());
                record_failure(
                    stats,
                    &install_page_path,
                    &step,
                    "install step exits 0",
                    &format!("exit {}: {}", status_code(&output), one_line(&text)),
                );
            }
            Err(error) => record_failure(
                stats,
                &install_page_path,
                "install route",
                "completed within timeout",
                &error,
            ),
        }
    }
    if let Some(candidate) = candidate.as_deref() {
        for (command, args) in [
            ("mkdir", vec!["-p", "/root/.local/bin"]),
            (
                "install",
                vec!["-m", "755", "/candidate-fsl", "/root/.local/bin/fsl"],
            ),
        ] {
            stats.checked += 1;
            match exec(&container, command, &args) {
                Ok(output) if output.status.success() => {}
                Ok(output) => record_failure(
                    stats,
                    &install_page_path,
                    &format!("install candidate {}", candidate.display()),
                    "exit 0",
                    &format!(
                        "exit {}: {}",
                        status_code(&output),
                        one_line(&output_text(&output))
                    ),
                ),
                Err(error) => record_failure(
                    stats,
                    &install_page_path,
                    "install candidate",
                    "completed within timeout",
                    &error,
                ),
            }
        }
    }

    stats.checked += 1;
    match exec(
        &container,
        "bash",
        &["-lc", "export PATH=/root/.local/bin:$PATH && fsl --version"],
    ) {
        Ok(version) => {
            let output = stdout_text(&version);
            let matches = matches_version_output(
                &output,
                &expected_version,
                expected_version_output.as_deref(),
            );
            if version.status.success() && matches {
                println!(
                    "PASS {} | fsl --version | {expected_version}",
                    install_page_path.display()
                );
            } else {
                record_failure(
                    stats,
                    &install_page_path,
                    "fsl --version",
                    &expected_version_output
                        .as_ref()
                        .map_or_else(|| expected_version.clone(), |lines| lines.join(" ")),
                    &format!(
                        "exit {}: {}{}",
                        status_code(&version),
                        one_line(&output),
                        stderr_suffix(&version)
                    ),
                );
            }
        }
        Err(error) => record_failure(
            stats,
            &install_page_path,
            "fsl --version",
            &expected_version,
            &error,
        ),
    }

    match disconnect_network(&container) {
        Ok(()) => println!("PASS gate | tutorial network disabled"),
        Err(error) => record_failure(
            stats,
            Path::new("gate"),
            "disable tutorial network",
            "network disconnected",
            &error,
        ),
    }

    for literal in literal_commands {
        stats.checked += 1;
        stats.tutorial_commands += 1;
        let expected_status = format!("exit {}", literal.expected_exit);
        let command_label = format!("line {}: {}", literal.line, literal.command);
        let shell_command = literal_shell_command(&literal);
        match exec(&container, "bash", &["-lc", &shell_command]) {
            Ok(actual) => {
                let actual_text = normalize_lines(&stdout_text(&actual));
                let expected_text = normalize_lines(&literal.expected);
                let version_command = literal.command.trim() == "fsl --version";
                let matches_output = if version_command {
                    actual_text.lines().next().map(str::trim)
                        == expected_text
                            .lines()
                            .find(|line| !line.trim().is_empty())
                            .map(str::trim)
                } else {
                    actual_text == expected_text
                };
                let expected_exit = if version_command {
                    0
                } else {
                    literal.expected_exit
                };
                if expected_exit == status_code(&actual) && matches_output {
                    println!(
                        "PASS {} | {}",
                        docs.join(&literal.page).display(),
                        literal.command
                    );
                } else {
                    record_failure(
                        stats,
                        &docs.join(&literal.page),
                        &command_label,
                        &format!("{} and {}", expected_status, one_line(&expected_text)),
                        &format!(
                            "exit {} and {}{}",
                            status_code(&actual),
                            one_line(&actual_text),
                            stderr_suffix(&actual)
                        ),
                    );
                }
            }
            Err(error) => record_failure(
                stats,
                &docs.join(&literal.page),
                &command_label,
                &expected_status,
                &error,
            ),
        }
    }
    for example in examples_list {
        stats.checked += 1;
        if example.kind != ExampleKind::Source {
            stats.tutorial_commands += 1;
        }
        let Some(case) = cases_by_system
            .get(&example.system)
            .and_then(|cases| cases.get(&example.id))
        else {
            record_failure(
                stats,
                &docs.join(&example.page),
                &format!("example {}/{}", example.system, example.id),
                "matching example.toml case",
                "case missing or manifest unreadable",
            );
            continue;
        };
        if case.status != "running" {
            record_failure(
                stats,
                &docs.join(&example.page),
                &format!("example {}/{}", example.system, example.id),
                "running case shown by docs renderer",
                &format!("manifest status is {:?}", case.status),
            );
            continue;
        }
        if let Err(error) = validate_example_parts(&examples, &example, case) {
            record_failure(
                stats,
                &docs.join(&example.page),
                &format!("render example {}/{}", example.system, example.id),
                "all shown source/data/request parts resolve",
                &error,
            );
        }
        if example.kind == ExampleKind::Source {
            let source_path = examples
                .join("examples")
                .join(&example.system)
                .join(&case.file);
            match read(&source_path) {
                Ok(source) if !source.is_empty() => println!(
                    "PASS {}:{} | source {}",
                    docs.join(&example.page).display(),
                    example.line,
                    source_path.display()
                ),
                Ok(_) => record_failure(
                    stats,
                    &docs.join(&example.page),
                    &format!("source {}/{}", example.system, example.id),
                    "non-empty displayed source",
                    "source file is empty",
                ),
                Err(error) => record_failure(
                    stats,
                    &docs.join(&example.page),
                    &format!("source {}/{}", example.system, example.id),
                    "readable source file",
                    &error,
                ),
            }
            continue;
        }
        let expected_path = examples
            .join("examples")
            .join(&example.system)
            .join("expected")
            .join(format!("{}.json", example.id));
        let expected: Value = match read(&expected_path).and_then(|text| {
            serde_json::from_str(&text)
                .map_err(|error| format!("{}: {error}", expected_path.display()))
        }) {
            Ok(expected) => expected,
            Err(error) => {
                record_failure(
                    stats,
                    &docs.join(&example.page),
                    &format!("expected output {}", example.id),
                    "valid expected JSON",
                    &error,
                );
                continue;
            }
        };
        let expected_output = match render_output(&expected, case.expected_exit, &example.view) {
            Ok(output) => output,
            Err(error) => {
                record_failure(
                    stats,
                    &docs.join(&example.page),
                    &format!("fsl {}", case.command),
                    "renderable page output",
                    &error,
                );
                continue;
            }
        };
        // The docs remark renderer displays `fsl <case.command> --format json`.
        let displayed_command = format!("fsl {} --format json", case.command);
        let shell_command = format!(
            "cd /candidate-examples/examples/{} && export PATH=/root/.local/bin:$PATH && {}",
            example.system, displayed_command
        );
        let executed_command = shell_command
            .rsplit_once(" && ")
            .map(|(_, command)| command);
        if executed_command != Some(displayed_command.as_str()) {
            record_failure(
                stats,
                &docs.join(&example.page),
                &displayed_command,
                "displayed command equals executed command",
                &shell_command,
            );
            continue;
        }
        match exec(&container, "bash", &["-lc", &shell_command]) {
            Ok(actual) => {
                let actual_output = stdout_text(&actual);
                let actual_json = serde_json::from_str::<Value>(&actual_output)
                    .ok()
                    .map(|value| project_recorded_outcome(&value));
                let actual_render = actual_json
                    .as_ref()
                    .map(|value| render_output(value, status_code(&actual), &example.view));
                if status_code(&actual) == case.expected_exit
                    && actual_render
                        .as_ref()
                        .and_then(|rendered| rendered.as_ref().ok())
                        == Some(&expected_output)
                {
                    println!(
                        "PASS {}:{} | {}",
                        docs.join(&example.page).display(),
                        example.line,
                        displayed_command
                    );
                } else {
                    let got = actual_render.map_or_else(
                        || {
                            format!(
                                "exit {} and invalid JSON: {}{}",
                                status_code(&actual),
                                one_line(&actual_output),
                                stderr_suffix(&actual)
                            )
                        },
                        |rendered| {
                            rendered.map_or_else(
                                |error| {
                                    format!(
                                        "exit {} and invalid output: {}{}",
                                        status_code(&actual),
                                        error,
                                        stderr_suffix(&actual)
                                    )
                                },
                                |value| {
                                    format!(
                                        "exit {} and {}{}",
                                        status_code(&actual),
                                        one_line(&value),
                                        stderr_suffix(&actual)
                                    )
                                },
                            )
                        },
                    );
                    record_failure(
                        stats,
                        &docs.join(&example.page),
                        &displayed_command,
                        &format!(
                            "exit {} and {}",
                            case.expected_exit,
                            one_line(&expected_output)
                        ),
                        &got,
                    );
                }
            }
            Err(error) => record_failure(
                stats,
                &docs.join(&example.page),
                &displayed_command,
                &format!("exit {}", case.expected_exit),
                &error,
            ),
        }
    }
    Ok(())
}

fn git_commit(repository: &Path) -> Result<String, String> {
    let path = repository.display().to_string();
    let output = Command::new("git")
        .args(["-C", &path, "rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("git rev-parse {}: {error}", repository.display()))?;
    if !output.status.success() {
        return Err(format!(
            "git rev-parse {} failed: {}",
            repository.display(),
            one_line(&String::from_utf8_lossy(&output.stderr))
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn record_failure(stats: &mut GateStats, page: &Path, command: &str, expected: &str, got: &str) {
    stats.failures += 1;
    fail_line(page, command, expected, got);
}

fn record_parse_issue(stats: &mut GateStats, docs: &Path, issue: ParseIssue) {
    stats.checked += 1;
    stats.skips += 1;
    println!(
        "SKIP {}:{} {}",
        docs.join(&issue.page).display(),
        issue.line,
        issue.reason
    );
    record_failure(
        stats,
        &docs.join(issue.page),
        &format!("fence at line {}", issue.line),
        "checked or explicitly allowlisted fence",
        &issue.reason,
    );
}

fn arguments() -> Result<(PathBuf, PathBuf, Option<PathBuf>, Option<PathBuf>, String), String> {
    let mut args = env::args().skip(1);
    let mut docs = None;
    let mut examples = None;
    let mut candidate = None;
    let mut released_binary = None;
    let mut base_image = None;
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value after {arg}"))?;
        match arg.as_str() {
            "--docs" => docs = Some(PathBuf::from(value)),
            "--examples" => examples = Some(PathBuf::from(value)),
            "--fsl-binary" => candidate = Some(PathBuf::from(value)),
            "--released-binary" => released_binary = Some(PathBuf::from(value)),
            "--base-image" => base_image = Some(value),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    let docs = docs.ok_or_else(|| {
        "usage: alpha-journey-gate --docs PATH --examples PATH [--fsl-binary PATH] [--released-binary PATH] [--base-image IMAGE]".to_owned()
    })?;
    let examples = examples.ok_or_else(|| {
        "usage: alpha-journey-gate --docs PATH --examples PATH [--fsl-binary PATH] [--released-binary PATH] [--base-image IMAGE]".to_owned()
    })?;
    if !docs.is_dir() || !examples.is_dir() {
        return Err("--docs and --examples must name existing checkouts".to_owned());
    }
    let docs = docs
        .canonicalize()
        .map_err(|error| format!("{}: {error}", docs.display()))?;
    let examples = examples
        .canonicalize()
        .map_err(|error| format!("{}: {error}", examples.display()))?;
    let candidate = candidate
        .map(|path| {
            if !path.is_file() {
                return Err(format!("{} is not a file", path.display()));
            }
            if !is_linux_elf(&path) {
                return Err(format!(
                    "{} is not a Linux ELF binary; --fsl-binary requires Linux ELF",
                    path.display()
                ));
            }
            path.canonicalize()
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .transpose()?;
    let released_binary = released_binary
        .map(|path| {
            if !path.is_file() {
                return Err(format!("{} is not a file", path.display()));
            }
            if !is_linux_elf(&path) {
                return Err(format!(
                    "{} is not a Linux ELF binary; --released-binary requires Linux ELF",
                    path.display()
                ));
            }
            path.canonicalize()
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .transpose()?;
    let base_image = base_image.unwrap_or_else(|| {
        if candidate.is_some() {
            "ubuntu:26.04".to_owned()
        } else {
            "ubuntu:26.04".to_owned()
        }
    });
    Ok((docs, examples, candidate, released_binary, base_image))
}

fn is_linux_elf(path: &Path) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut magic = [0; 4];
    file.read_exact(&mut magic).is_ok() && magic == *b"\x7fELF"
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn install_step_ids(page: &str, candidate: bool) -> Vec<String> {
    let mut ids = Vec::new();
    let mut section = String::new();
    for line in page.lines() {
        if let Some(heading) = line.trim().strip_prefix("## ") {
            section = heading.to_owned();
        }
        let Some(reference) = line.trim().strip_prefix("```steps ") else {
            continue;
        };
        for id in reference.split_whitespace() {
            let selected = if candidate {
                id == "prerequisites" || (section == "Run the examples" && id == "clone")
            } else {
                section != "Build the driver from source"
            };
            if selected && !ids.iter().any(|existing| existing == id) {
                ids.push(id.to_owned());
            }
        }
    }
    ids
}

fn step_blocks(source: &str) -> HashMap<String, String> {
    let mut blocks = HashMap::new();
    let mut current_id = None;
    let mut contents = Vec::new();
    for line in source.lines() {
        if let Some(id) = line.trim().strip_prefix("```bash step=") {
            current_id = Some(id.trim().to_owned());
            contents.clear();
        } else if line.trim() == "```" {
            if let Some(id) = current_id.take() {
                blocks.insert(id, contents.join("\n"));
            }
        } else if current_id.is_some() {
            contents.push(line.to_owned());
        }
    }
    blocks
}

fn assemble_install_script(
    ids: &[String],
    blocks: &HashMap<String, String>,
    page: &Path,
) -> Result<String, String> {
    let mut script = String::from("set -eE\n");
    script.push_str(
        r#"trap 'status=$?; printf "__STEP_FAILED__%s\n" "$step_id"; exit "$status"' ERR"#,
    );
    script.push('\n');
    for id in ids {
        let block = blocks
            .get(id)
            .ok_or_else(|| format!("{} references missing install step {id}", page.display()))?;
        script.push_str(&format!("step_id={id}\nprintf '__STEP_START__{id}\\n'\n"));
        script.push_str(block);
        script.push('\n');
        script.push_str(&format!("printf '__STEP_OK__{id}\\n'\n"));
    }
    Ok(script)
}

fn expected_version(page: &str) -> Option<String> {
    for line in page.lines() {
        let line = line.trim();
        if let Some(version) = line.strip_prefix("FSL v") {
            if let Some(version) = version.split_whitespace().next() {
                return Some(version.trim_end_matches('.').to_owned());
            }
        }
        if let Some(version) = line.strip_prefix("The latest published driver release is v") {
            if let Some(version) = version.split_whitespace().next() {
                return Some(version.trim_end_matches('.').to_owned());
            }
        }
        if let Some((_, version_statement)) = line.split_once("Version ") {
            if let Some(version) = version_statement.split_whitespace().next() {
                let version = version.trim_end_matches('.');
                let is_numeric_version = version.contains('.')
                    && version
                        .split('.')
                        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
                if is_numeric_version {
                    return Some(version.to_owned());
                }
            }
        }
    }
    None
}

fn expected_version_output(page: &str) -> Option<Vec<String>> {
    let mut lines = page.lines();
    while let Some(line) = lines.next() {
        if !matches!(line.trim(), "$ fsl --version" | "fsl --version") {
            continue;
        }
        let mut output = Vec::new();
        for line in lines.by_ref() {
            if line.trim() == "```" {
                break;
            }
            if !line.trim().is_empty() {
                output.push(line.trim().to_owned());
            }
        }
        return (!output.is_empty()).then_some(output);
    }
    None
}

fn tutorial_pages(docs: &Path, directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pages = Vec::new();
    for entry in
        fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry.path().display()))?
            .is_file()
            && file_name.starts_with("tutorial-")
            && file_name.ends_with(".md")
        {
            pages.push(Path::new(TUTORIAL_DIR).join(file_name.as_ref()));
        }
    }
    pages.push(Path::new(REFINEMENT_PAGE).to_path_buf());
    pages.push(Path::new(WRITE_A_MODEL_PAGE).to_path_buf());
    let protocols = docs.join("src/content/docs/fsl/learn/protocols.md");
    if protocols.is_file() {
        pages.push(Path::new("src/content/docs/fsl/learn/protocols.md").to_path_buf());
    }
    pages.sort();
    if pages.is_empty() {
        return Err(format!(
            "{} has no tutorial-*.md pages",
            directory.display()
        ));
    }
    Ok(pages)
}

fn fence_open(line: &str) -> Option<(char, usize, usize, &str)> {
    let indent = line.len() - line.trim_start().len();
    let content = &line[indent..];
    let marker = content.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let length = content
        .chars()
        .take_while(|character| *character == marker)
        .count();
    (length >= 3).then_some((marker, length, indent, &content[length..]))
}

fn fence_close(line: &str, marker: char, minimum_length: usize) -> bool {
    let content = line.trim();
    let length = content
        .chars()
        .take_while(|character| *character == marker)
        .count();
    length >= minimum_length && content[length..].trim().is_empty()
}

fn page_examples(page: &Path, source: &str) -> (Vec<Example>, Vec<ParseIssue>) {
    let mut examples = Vec::new();
    let mut issues = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let Some((_, _, _, info)) = fence_open(line) else {
            continue;
        };
        let number = index + 1;
        let info = info.trim();
        let Some(reference) = info.strip_prefix("example") else {
            continue;
        };
        let Some(separator) = reference
            .chars()
            .next()
            .filter(|character| character.is_whitespace())
        else {
            issues.push(ParseIssue {
                page: page.to_path_buf(),
                line: number,
                reason: "example fence is missing its reference".to_owned(),
            });
            continue;
        };
        let reference = &reference[separator.len_utf8()..];
        let Some(key) = reference.split_whitespace().next() else {
            issues.push(ParseIssue {
                page: page.to_path_buf(),
                line: number,
                reason: "example fence is missing its reference".to_owned(),
            });
            continue;
        };
        let Some((system, id)) = key.split_once('/') else {
            issues.push(ParseIssue {
                page: page.to_path_buf(),
                line: number,
                reason: format!("example reference {key:?} must be system/id"),
            });
            continue;
        };
        let show = attribute(reference, "show").unwrap_or("source data command");
        let show = show
            .split(|character: char| character.is_whitespace() || character == ',')
            .filter(|part| !part.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if show.iter().any(|part| {
            !matches!(
                part.as_str(),
                "source" | "file" | "data" | "request" | "command" | "output"
            )
        }) {
            issues.push(ParseIssue {
                page: page.to_path_buf(),
                line: number,
                reason: format!("unsupported example part in show={show:?}"),
            });
            continue;
        }
        let kind = if show.iter().any(|part| part == "command") {
            ExampleKind::Command
        } else if show.iter().any(|part| part == "output") {
            ExampleKind::Output
        } else if show
            .iter()
            .any(|part| part == "source" || part == "file" || part == "data" || part == "request")
        {
            ExampleKind::Source
        } else {
            issues.push(ParseIssue {
                page: page.to_path_buf(),
                line: number,
                reason: "example block does not show a checked command, output, or file part"
                    .to_owned(),
            });
            continue;
        };
        let fields = attribute(reference, "fields");
        if fields.as_deref() == Some("") {
            issues.push(ParseIssue {
                page: page.to_path_buf(),
                line: number,
                reason: "fields= must name at least one output field".to_owned(),
            });
            continue;
        }
        let view = if reference.split_whitespace().any(|part| part == "summary") {
            OutputView::Summary
        } else if let Some(fields) = fields {
            let fields = fields
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if fields.is_empty() {
                issues.push(ParseIssue {
                    page: page.to_path_buf(),
                    line: number,
                    reason: "fields= must name at least one output field".to_owned(),
                });
                continue;
            }
            OutputView::Fields(fields)
        } else {
            OutputView::Summary
        };
        examples.push(Example {
            page: page.to_path_buf(),
            line: number,
            id: id.to_owned(),
            system: system.to_owned(),
            kind,
            show,
            view,
        });
    }
    (examples, issues)
}

fn attribute<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}=\"");
    let (_, value) = source.split_once(&prefix)?;
    value.split_once('"').map(|(value, _)| value)
}

fn page_literal_commands(
    page: &Path,
    system: &str,
    source: &str,
) -> (
    Vec<LiteralCommand>,
    Vec<AllowedSkip>,
    Vec<FileExcerpt>,
    Vec<ParseIssue>,
) {
    let mut commands = Vec::new();
    let mut skips = Vec::new();
    let mut excerpts = Vec::new();
    let mut issues = Vec::new();
    let mut in_text_block: Option<(char, usize)> = None;
    let mut lines: Vec<String> = Vec::new();
    let mut block_line = 0;
    let mut fence_indent = 0;
    let mut declared_exit: Option<i32> = None;
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if in_text_block.is_none()
            && fence_open(line).is_some_and(|(_, _, _, info)| {
                let info = info.trim();
                info == "text" || info.starts_with("text ")
            })
        {
            let (marker, length, _, _) = fence_open(line).expect("checked fence");
            in_text_block = Some((marker, length));
            lines.clear();
            block_line = index + 1;
            fence_indent = line.len() - line.trim_start().len();
            declared_exit = trimmed
                .split_whitespace()
                .find_map(|part| part.strip_prefix("exit="))
                .and_then(|value| value.parse().ok());
        } else if in_text_block.is_some_and(|(marker, length)| fence_close(line, marker, length)) {
            let content = lines.iter().map(|line| line.trim_end()).collect::<Vec<_>>();
            let first = content
                .iter()
                .position(|line| !line.trim().is_empty())
                .map(|index| content[index].trim());
            if let Some(command) = first.filter(|line| {
                ["fsl ", "cat ", "sed ", MODEL_GENERATOR_PREFIX]
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
            }) {
                let expected = lines
                    .iter()
                    .skip(
                        content
                            .iter()
                            .position(|line| !line.trim().is_empty())
                            .unwrap_or(0)
                            + 1,
                    )
                    .map(|line| line.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n");
                if declared_exit.is_some_and(|exit| exit != 0) {
                    issues.push(ParseIssue {
                        page: page.to_path_buf(),
                        line: block_line,
                        reason: format!("literal command {command:?} shows non-zero exit {}; use an example block backed by a case", declared_exit.unwrap_or_default()),
                    });
                }
                if let Some(exit) = shown_nonzero_exit(&expected) {
                    issues.push(ParseIssue {
                        page: page.to_path_buf(),
                        line: block_line,
                        reason: format!("literal command {command:?} shows exit {exit}; use an example block backed by a case"),
                    });
                }
                commands.push(LiteralCommand {
                    page: page.to_path_buf(),
                    system: system.to_owned(),
                    command: command.to_owned(),
                    expected,
                    expected_exit: 0,
                    line: block_line,
                });
            } else if let Some(command) =
                first.filter(|line| line.starts_with("git clone ") || line.starts_with("cd "))
            {
                skips.push(AllowedSkip {
                    page: page.to_path_buf(),
                    line: block_line,
                    reason: format!("allowlisted tutorial setup command: {command}"),
                });
            } else if !lines.iter().all(|line| line.trim().is_empty()) {
                excerpts.push(FileExcerpt {
                    page: page.to_path_buf(),
                    line: block_line,
                    system: system.to_owned(),
                    text: lines.join("\n"),
                });
            } else {
                issues.push(ParseIssue {
                    page: page.to_path_buf(),
                    line: block_line,
                    reason: "empty text fence".to_owned(),
                });
            }
            in_text_block = None;
        } else if in_text_block.is_some() {
            let dedented = line
                .get(fence_indent..)
                .filter(|_| line.len() >= fence_indent)
                .unwrap_or(line);
            lines.push(dedented.to_owned());
        }
    }
    if in_text_block.is_some() {
        issues.push(ParseIssue {
            page: page.to_path_buf(),
            line: block_line,
            reason: "unterminated text fence".to_owned(),
        });
    }
    (commands, skips, excerpts, issues)
}

/// The shell command that runs one literal tutorial command in the container.
///
/// A command runs in its system folder. The model generator is the one command that
/// writes a file, and the examples mount is read-only: it runs from the examples root,
/// writes to a temporary file, and the generated model must equal the written file.
fn literal_shell_command(literal: &LiteralCommand) -> String {
    let words: Vec<&str> = literal.command.split_whitespace().collect();
    if literal.command.starts_with(MODEL_GENERATOR_PREFIX) && words.len() == 4 {
        return format!(
            "cd /candidate-examples && export PATH=/root/.local/bin:$PATH && node tools/model.mjs {} /tmp/journey-model.json && cmp /tmp/journey-model.json {}",
            words[2], words[3]
        );
    }
    format!(
        "cd /candidate-examples/examples/{} && export PATH=/root/.local/bin:$PATH && {}",
        literal.system, literal.command
    )
}

fn shown_nonzero_exit(text: &str) -> Option<i32> {
    text.match_indices("exit ").find_map(|(index, _)| {
        text[index + 5..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse::<i32>()
            .ok()
            .filter(|exit| *exit != 0)
    })
}

/// The files that a `steps file=<path>` fence renders, with the line of the fence.
fn page_step_files(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let (_, _, _, info) = fence_open(line)?;
            let path = info.trim().strip_prefix("steps file=")?;
            Some((index + 1, path.trim().to_owned()))
        })
        .collect()
}

fn unparsed_fences(page: &Path, source: &str) -> Vec<ParseIssue> {
    let mut issues = Vec::new();
    let mut opening: Option<(usize, String, char, usize)> = None;
    for (index, line) in source.lines().enumerate() {
        if let Some((marker, length, _, info)) = fence_open(line) {
            if opening
                .as_ref()
                .is_some_and(|(_, _, open_marker, open_len)| {
                    fence_close(line, *open_marker, *open_len)
                })
            {
                opening = None;
                continue;
            }
            if opening.is_some() {
                issues.push(ParseIssue {
                    page: page.to_path_buf(),
                    line: index + 1,
                    reason: "nested fenced block before a closing fence".to_owned(),
                });
            }
            let kind = info.trim();
            opening = Some((index + 1, kind.to_owned(), marker, length));
            if !(kind == "example"
                || kind.starts_with("example ")
                || kind == "text"
                || kind.starts_with("text ")
                || kind.starts_with("steps file="))
            {
                issues.push(ParseIssue {
                    page: page.to_path_buf(),
                    line: index + 1,
                    reason: format!("unparsed fenced block {kind:?}"),
                });
            }
        }
    }
    if let Some((line, kind, _, _)) = opening {
        issues.push(ParseIssue {
            page: page.to_path_buf(),
            line,
            reason: format!("unterminated {kind:?} fence"),
        });
    }
    issues
}

fn manifest_cases(source: &str) -> HashMap<String, Case> {
    let mut cases = HashMap::new();
    for section in source.split("[[case]]").skip(1) {
        let id = quoted_field(section, "id");
        let rule = quoted_field(section, "rule");
        let command = quoted_field(section, "command");
        let file = quoted_field(section, "file");
        let status = quoted_field(section, "status");
        let expected_exit = integer_field(section, "expect_exit");
        if let (
            Some(id),
            Some(rule),
            Some(command),
            Some(file),
            Some(status),
            Some(expected_exit),
        ) = (id, rule, command, file, status, expected_exit)
        {
            cases.insert(
                id,
                Case {
                    rule,
                    command,
                    file,
                    status,
                    expected_exit,
                },
            );
        }
    }
    cases
}

fn quoted_field(section: &str, field: &str) -> Option<String> {
    section.lines().find_map(|line| {
        let value = line.trim().strip_prefix(&format!("{field} = \""))?;
        let mut chars = value.chars();
        let mut output = String::new();
        while let Some(ch) = chars.next() {
            match ch {
                '"' => return Some(output),
                '\\' => match chars.next()? {
                    '"' => output.push('"'),
                    '\\' => output.push('\\'),
                    other => {
                        output.push('\\');
                        output.push(other);
                    }
                },
                other => output.push(other),
            }
        }
        None
    })
}

fn integer_field(section: &str, field: &str) -> Option<i32> {
    section.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&format!("{field} = "))?
            .parse()
            .ok()
    })
}

fn trim_rule(source: &str, rule: &str) -> String {
    if rule.is_empty() {
        return source.trim_end().to_owned();
    }
    let lines = source.lines().collect::<Vec<_>>();
    let needle = rule;
    let Some(at) = lines
        .iter()
        .position(|line| !line.trim().starts_with("//") && line.contains(&needle))
    else {
        return source.trim_end().to_owned();
    };
    let mut start = at;
    while start > 0 && lines[start - 1].trim().starts_with("//") {
        start -= 1;
    }
    let mut depth = 0_i32;
    let mut end = at;
    for (index, line) in lines.iter().enumerate().skip(at) {
        depth += line.matches('{').count() as i32;
        depth -= line.matches('}').count() as i32;
        end = index;
        if depth <= 0 && (line.contains('}') || line.contains(';')) {
            break;
        }
    }
    lines[start..=end].join("\n")
}

fn validate_example_parts(examples: &Path, example: &Example, case: &Case) -> Result<(), String> {
    let system_dir = examples.join("examples").join(&example.system);
    let source_path = system_dir.join(&case.file);
    if example
        .show
        .iter()
        .any(|part| part == "source" || part == "file")
    {
        let source = read(&source_path)?;
        let rendered = if example.show.iter().any(|part| part == "file") {
            source
        } else {
            trim_rule(&source, &case.rule)
        };
        if rendered.trim().is_empty() {
            return Err(format!("{} renders no source text", source_path.display()));
        }
    }
    if example
        .show
        .iter()
        .any(|part| part == "request" || part == "data")
    {
        let request = case
            .command
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| format!("case {} has no request argument", example.id))?;
        let request_path = system_dir.join(request);
        let request_text = read(&request_path)?;
        if example.show.iter().any(|part| part == "data") {
            let request_json: Value = serde_json::from_str(&request_text)
                .map_err(|error| format!("{}: {error}", request_path.display()))?;
            if let Some(observations) = request_json
                .pointer("/request/clause/observations")
                .and_then(Value::as_object)
            {
                for relative in observations.values().filter_map(Value::as_str) {
                    let data_path = request_path.parent().unwrap_or(&system_dir).join(relative);
                    read(&data_path)?;
                }
            }
        }
    }
    Ok(())
}

fn render_output(value: &Value, exit: i32, view: &OutputView) -> Result<String, String> {
    match view {
        OutputView::Summary => {
            let items = value
                .get("items")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if items.is_empty() {
                let category = value
                    .get("category")
                    .ok_or_else(|| "summary has no category".to_owned())?;
                return Ok(format!("{}, exit {exit}", show_value(category)));
            }
            items
                .iter()
                .map(|item| {
                    let name = item
                        .get("clause")
                        .or_else(|| item.get("claim_name"))
                        .or_else(|| value.get("category"))
                        .ok_or_else(|| "summary item has no clause or category".to_owned())?;
                    let result = item
                        .get("result")
                        .or_else(|| item.get("record").and_then(|record| record.get("result")))
                        .ok_or_else(|| "summary item has no result".to_owned())?;
                    Ok(format!(
                        "{}: {}, exit {exit}",
                        show_value(name),
                        show_value(result)
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
                .map(|lines| lines.join("\n"))
        }
        OutputView::Fields(fields) => {
            let items = value.get("items").and_then(Value::as_array);
            let diagnostics = value.get("diagnostics").and_then(Value::as_array);
            let records = items
                .filter(|records| !records.is_empty())
                .or_else(|| diagnostics.filter(|records| !records.is_empty()))
                .map(|records| records.iter().collect::<Vec<_>>())
                .unwrap_or_else(|| vec![value]);
            records
                .iter()
                .enumerate()
                .map(|(index, record)| {
                    let mut lines = Vec::new();
                    for field in fields {
                        let path = field.split('.').collect::<Vec<_>>();
                        let values = path_values(record, &path);
                        let values = if values.is_empty() {
                            path_values(value, &path)
                        } else {
                            values
                        };
                        if values.is_empty() {
                            return Err(format!(
                                "field {field} is absent from record {}",
                                index + 1
                            ));
                        }
                        for selected in values {
                            lines.push(format!("{field}: {}", show_value(selected)));
                        }
                    }
                    Ok(lines.join("\n"))
                })
                .collect::<Result<Vec<_>, String>>()
                .map(|records| records.join("\n\n"))
        }
    }
}

fn project_recorded_outcome(raw: &Value) -> Value {
    if raw.get("format").and_then(Value::as_str) != Some("filament-outcome/1") {
        return serde_json::json!({ "outcome": null });
    }
    let field = |name: &str| raw.get(name).cloned().unwrap_or(Value::Null);
    let diagnostics = raw
        .get("diagnostics")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let first_diagnostic = diagnostics.first().map_or(Value::Null, |diagnostic| {
        serde_json::json!({
            "code": diagnostic.get("code").cloned().unwrap_or(Value::Null),
            "cause": diagnostic.get("cause").cloned().unwrap_or(Value::Null),
        })
    });
    let items = raw
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    serde_json::json!({
        "category": field("category"),
        "last_stage": field("last_stage"),
        "result": field("result"),
        "first_diagnostic": first_diagnostic,
        "items": items,
    })
}

fn matches_version_output(
    actual: &str,
    expected: &str,
    expected_output: Option<&[String]>,
) -> bool {
    let expected_first = expected_output
        .and_then(|lines| lines.first())
        .map(String::as_str)
        .unwrap_or(expected)
        .trim();
    actual.lines().next().map(str::trim) == Some(expected_first)
}

fn path_values<'a>(value: &'a Value, path: &[&str]) -> Vec<&'a Value> {
    let Some((segment, rest)) = path.split_first() else {
        return vec![value];
    };
    if *segment == "*" {
        return value
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|item| path_values(item, rest))
            .collect();
    }
    value
        .get(*segment)
        .map(|next| path_values(next, rest))
        .unwrap_or_default()
}

fn show_value(value: &Value) -> String {
    match value {
        Value::String(string) => string.clone(),
        _ => value.to_string(),
    }
}

fn check_released_binary(
    examples: &Path,
    binary: &Path,
    install_page: &Path,
    expected_version: &str,
    expected_output: Option<&[String]>,
) -> Result<(), String> {
    check_glibc_requirements(binary)?;
    let container = start_container(examples, Some(binary), "ubuntu:26.04")?;
    let setup = exec(
        &container,
        "bash",
        &[
            "-e",
            "-c",
            "mkdir -p /root/.local/bin && install -m 755 /candidate-fsl /root/.local/bin/fsl",
        ],
    )?;
    if !setup.status.success() {
        return Err(format!(
            "exit {}: {}",
            status_code(&setup),
            one_line(&output_text(&setup))
        ));
    }
    let version = exec(
        &container,
        "bash",
        &["-lc", "export PATH=/root/.local/bin:$PATH && fsl --version"],
    )?;
    let output = stdout_text(&version);
    let matches = matches_version_output(&output, expected_version, expected_output);
    if !version.status.success() || !matches {
        return Err(format!(
            "exit {}: {}",
            status_code(&version),
            one_line(&output)
        ));
    }
    println!(
        "PASS {} | released artifact starts on ubuntu:26.04 and needs GLIBC <= 2.43 without GLIBC_PRIVATE | {expected_version}",
        install_page.display()
    );
    Ok(())
}

fn check_glibc_requirements(binary: &Path) -> Result<(), String> {
    let output = Command::new("readelf")
        .args(["--version-info", "--wide"])
        .arg(binary)
        .output()
        .map_err(|error| format!("could not run readelf for GLIBC check: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "readelf failed: {}",
            one_line(&String::from_utf8_lossy(&output.stderr))
        ));
    }
    validate_glibc_version_info(&String::from_utf8_lossy(&output.stdout))?;
    Ok(())
}

fn validate_glibc_version_info(version_info: &str) -> Result<(u32, u32, u32), String> {
    let mut maximum = None;
    for token in glibc_tokens(version_info) {
        if token == "GLIBC_PRIVATE" {
            return Err("artifact requires rejected GLIBC_PRIVATE symbol version".to_owned());
        }
        if let Some(version) = parse_glibc_version(token) {
            maximum = Some(maximum.map_or(version, |known: (u32, u32, u32)| known.max(version)));
        }
    }
    let maximum = maximum.ok_or_else(|| "readelf reported no GLIBC symbol versions".to_owned())?;
    if maximum > (2, 43, 0) {
        return Err(format!(
            "artifact requires GLIBC_{}.{}.{}, above GLIBC_2.43",
            maximum.0, maximum.1, maximum.2
        ));
    }
    Ok(maximum)
}

fn glibc_tokens(source: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut rest = source;
    while let Some(index) = rest.find("GLIBC_") {
        let candidate = &rest[index..];
        let length = candidate
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || *character == '_' || *character == '.'
            })
            .map(char::len_utf8)
            .sum::<usize>();
        if length > 6 {
            tokens.push(&candidate[..length]);
        }
        rest = &candidate[length.max(6)..];
    }
    tokens
}

fn parse_glibc_version(token: &str) -> Option<(u32, u32, u32)> {
    let version = token.strip_prefix("GLIBC_")?;
    let mut parts = version.split('.');
    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next().map(str::parse).transpose().ok()?.unwrap_or(0),
    ))
}

fn start_container(
    examples: &Path,
    candidate: Option<&Path>,
    base_image: &str,
) -> Result<Container, String> {
    let mount = format!(
        "type=bind,src={},dst=/candidate-examples,readonly",
        examples.display()
    );
    let mut command = Command::new("docker");
    command.args(["run", "-d", "--rm", "--mount", &mount]);
    if let Some(candidate) = candidate {
        let candidate_mount = format!(
            "type=bind,src={},dst=/candidate-fsl,readonly",
            candidate.display()
        );
        command.arg("--mount").arg(candidate_mount);
    }
    let output = command
        .args([base_image, "sleep", "infinity"])
        .output()
        .map_err(|error| format!("could not start docker: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "docker run failed: {}",
            one_line(&String::from_utf8_lossy(&output.stderr))
        ));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if id.is_empty() {
        return Err("docker run returned an empty container id".to_owned());
    }
    Ok(Container(id))
}

fn exec(container: &Container, program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new("docker")
        .args(["exec", &container.0, "timeout", "300s", program])
        .args(args)
        .output()
        .map_err(|error| format!("docker exec {program}: {error}"))
}

fn disconnect_network(container: &Container) -> Result<(), String> {
    let output = Command::new("docker")
        .args(["network", "disconnect", "bridge", &container.0])
        .output()
        .map_err(|error| format!("docker network disconnect: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "docker network disconnect failed: {}",
            one_line(&String::from_utf8_lossy(&output.stderr))
        ))
    }
}

fn status_code(output: &Output) -> i32 {
    output.status.code().unwrap_or(128)
}

fn output_text(output: &Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_suffix(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.trim().is_empty() {
        String::new()
    } else {
        format!("; stderr: {}", one_line(&stderr))
    }
}

fn normalize_lines(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

fn excerpt_matches(examples: &Path, system: &str, excerpt: &str) -> Result<bool, String> {
    let root = examples.join("examples").join(system);
    if !root.is_dir() {
        return Err(format!("{} is not a system directory", root.display()));
    }
    let expected = normalize_lines(excerpt);
    if expected.is_empty() {
        return Ok(false);
    }
    excerpt_matches_in(&root, &expected)
}

fn excerpt_matches_in(directory: &Path, expected: &str) -> Result<bool, String> {
    for entry in
        fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == ".git") {
                continue;
            }
            if excerpt_matches_in(&path, expected)? {
                return Ok(true);
            }
        } else if path.is_file() {
            if let Ok(contents) = fs::read_to_string(&path) {
                if contains_contiguous_lines(&contents, expected) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn contains_contiguous_lines(file: &str, excerpt: &str) -> bool {
    let file = normalize_lines(file)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let excerpt = normalize_lines(excerpt);
    let excerpt = excerpt.lines().collect::<Vec<_>>();
    !excerpt.is_empty()
        && file.windows(excerpt.len()).any(|window| {
            window
                .iter()
                .map(String::as_str)
                .eq(excerpt.iter().copied())
        })
}

fn failed_step(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| line.strip_prefix("__STEP_FAILED__").map(str::to_owned))
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fail_line(page: &Path, command: &str, expected: &str, got: &str) {
    eprintln!(
        "FAIL {} | {} | expected {} | got {}",
        page.display(),
        command,
        expected,
        got
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_examples_reads_summary_and_output_views_and_rejects_empty_fields() {
        let page = Path::new("tutorial.md");
        let source = concat!(
            "```example shop/default\n```\n",
            "```example shop/run show=\"command\" summary\n```\n",
            "```example shop/run show=\"output\" fields=\"claim_name record.result\"\n```\n",
            "```example shop/run show=\"output\" fields=\"\"\n```\n"
        );
        let (examples, issues) = page_examples(page, source);
        assert_eq!(examples.len(), 3);
        assert_eq!(examples[0].kind, ExampleKind::Command);
        assert_eq!(examples[0].show, vec!["source", "data", "command"]);
        assert_eq!(examples[1].view, OutputView::Summary);
        assert_eq!(examples[1].kind, ExampleKind::Command);
        assert_eq!(examples[2].kind, ExampleKind::Output);
        assert_eq!(
            issues
                .iter()
                .map(|issue| issue.reason.as_str())
                .collect::<Vec<_>>(),
            vec!["fields= must name at least one output field"]
        );
    }

    #[test]
    fn page_examples_reports_unparseable_example_fence() {
        let (examples, issues) = page_examples(Path::new("tutorial.md"), "```example bad\n```\n");
        assert!(examples.is_empty());
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn tutorial_pages_includes_refinement_write_a_model_and_optional_probability_page() {
        let root = std::env::temp_dir().join(format!(
            "alpha-journey-tutorial-pages-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the Unix epoch")
                .as_nanos()
        ));
        let tutorials = root.join(TUTORIAL_DIR);
        let learn = root.join("src/content/docs/fsl/learn");
        fs::create_dir_all(&tutorials).expect("create tutorial fixture directory");
        fs::create_dir_all(&learn).expect("create learn fixture directory");
        fs::write(learn.join("refinement.md"), "refinement page")
            .expect("write refinement fixture");
        for index in 0..6 {
            fs::write(
                tutorials.join(format!("tutorial-existing-{index}.md")),
                "tutorial page",
            )
            .expect("write existing tutorial fixture");
        }

        let without_probability =
            tutorial_pages(&root, &tutorials).expect("pages without probability");
        assert_eq!(without_probability.len(), 8);
        assert!(without_probability.contains(&Path::new(REFINEMENT_PAGE).to_path_buf()));
        assert!(without_probability.contains(&Path::new(WRITE_A_MODEL_PAGE).to_path_buf()));
        assert!(!without_probability.iter().any(|page| {
            page.file_name()
                .is_some_and(|name| name == "tutorial-probability.md")
        }));

        fs::write(
            tutorials.join("tutorial-probability.md"),
            "probability page",
        )
        .expect("write probability fixture");
        let with_probability = tutorial_pages(&root, &tutorials).expect("pages with probability");
        assert_eq!(with_probability.len(), 9);
        assert!(with_probability.iter().any(|page| {
            page.file_name()
                .is_some_and(|name| name == "tutorial-probability.md")
        }));

        fs::remove_dir_all(&root).expect("remove tutorial fixtures");
    }

    #[test]
    fn unparsed_fences_catches_misspelled_example_info() {
        let issues = unparsed_fences(Path::new("tutorial.md"), "```exmaple shop/run\n```\n");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line, 1);
    }

    #[test]
    fn page_literal_commands_rejects_nonzero_examples_and_defaults_to_exit_zero() {
        let source = concat!(
            "```text exit=10\nfsl check bad.fsl\nviolation\n```\n",
            "```text exit=20\nfsl check refused.fsl\nrefused\n```\n",
            "```text\nfsl --version\n0.2.0\n```\n"
        );
        let (commands, _, excerpts, issues) =
            page_literal_commands(Path::new("tutorial.md"), "shop", source);
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].expected_exit, 0);
        assert_eq!(commands[1].expected_exit, 0);
        assert_eq!(commands[2].expected_exit, 0);
        assert!(excerpts.is_empty());
        assert_eq!(issues.len(), 2);
        assert!(issues
            .iter()
            .all(|issue| issue.reason.contains("use an example block")));
    }

    #[test]
    fn page_literal_commands_allows_only_clone_and_cd_skips() {
        let source = "```text\ngit clone examples\ncd examples/shop\n```\n";
        let (commands, skips, excerpts, issues) =
            page_literal_commands(Path::new("tutorial.md"), "shop", source);
        assert!(commands.is_empty());
        assert_eq!(skips.len(), 1);
        assert!(excerpts.is_empty());
        assert!(issues.is_empty());
    }

    #[test]
    fn model_generator_command_runs_from_the_examples_root_and_compares_the_model() {
        let source = "```text\nnode tools/model.mjs tools/models/library.mjs examples/library/model/library.semantic-ir.json\n```\n";
        let (commands, skips, excerpts, issues) =
            page_literal_commands(Path::new("guide.md"), "library", source);
        assert_eq!(commands.len(), 1);
        assert!(skips.is_empty() && excerpts.is_empty() && issues.is_empty());
        let shell = literal_shell_command(&commands[0]);
        assert!(shell.starts_with("cd /candidate-examples && "));
        assert!(shell.contains(
            "node tools/model.mjs tools/models/library.mjs /tmp/journey-model.json && cmp /tmp/journey-model.json examples/library/model/library.semantic-ir.json"
        ));
    }

    #[test]
    fn other_literal_commands_run_from_their_system_folder() {
        let source = "```text\nfsl --version\n```\n";
        let (commands, _, _, _) = page_literal_commands(Path::new("guide.md"), "shop", source);
        assert_eq!(
            literal_shell_command(&commands[0]),
            "cd /candidate-examples/examples/shop && export PATH=/root/.local/bin:$PATH && fsl --version"
        );
    }

    #[test]
    fn steps_file_fences_are_listed_and_not_reported_as_unparsed() {
        let source = "1. Describe it.\n\n   ```steps file=tools/models/library.mjs\n   ```\n";
        assert_eq!(
            page_step_files(source),
            vec![(3, "tools/models/library.mjs".to_owned())]
        );
        assert!(unparsed_fences(Path::new("guide.md"), source).is_empty());
        assert_eq!(
            unparsed_fences(Path::new("guide.md"), "```steps other\n```\n").len(),
            1
        );
    }

    #[test]
    fn page_literal_commands_treats_other_text_as_a_file_excerpt() {
        let source = "```text\ninvariant R { true }\n```\n";
        let (commands, skips, excerpts, issues) =
            page_literal_commands(Path::new("tutorial.md"), "shop", source);
        assert!(commands.is_empty());
        assert!(skips.is_empty());
        assert_eq!(excerpts.len(), 1);
        assert_eq!(excerpts[0].text, "invariant R { true }");
        assert!(issues.is_empty());
    }

    #[test]
    fn manifest_cases_unescapes_quoted_commands() {
        let cases = manifest_cases(
            r#"[[case]]
id = "quoted"
rule = "R"
file = "rules/a.fsl"
command = "run data/\"quoted\".json"
status = "running"
expect_exit = 10
"#,
        );
        assert_eq!(cases["quoted"].command, "run data/\"quoted\".json");
        assert_eq!(cases["quoted"].expected_exit, 10);
    }

    #[test]
    fn render_output_supports_summary_and_output_fields() {
        let value: Value = serde_json::from_str(
            r#"{"items":[{"claim_name":"C","record":{"result":"refuted"},"counterexample":{"kind":"formula"}}]}"#,
        )
        .unwrap();
        assert_eq!(
            render_output(&value, 10, &OutputView::Summary).unwrap(),
            "C: refuted, exit 10"
        );
        assert_eq!(
            render_output(&value, 20, &OutputView::Summary).unwrap(),
            "C: refuted, exit 20"
        );
        assert_eq!(
            render_output(
                &value,
                10,
                &OutputView::Fields(vec![
                    "claim_name".to_owned(),
                    "counterexample.kind".to_owned()
                ])
            )
            .unwrap(),
            "claim_name: C\ncounterexample.kind: formula"
        );
    }

    #[test]
    fn install_step_ids_selects_candidate_and_release_routes() {
        let page = concat!(
            "## Download the prebuilt driver\n```steps prerequisites node\n",
            "## Install the FSL driver from a release\n```steps driver-target driver-download\n",
            "## Build the driver from source\n```steps build-tools rust driver-source\n",
            "## Run the examples\n```steps clone check\n"
        );
        assert_eq!(install_step_ids(page, true), vec!["prerequisites", "clone"]);
        assert_eq!(
            install_step_ids(page, false),
            vec![
                "prerequisites",
                "node",
                "driver-target",
                "driver-download",
                "clone",
                "check"
            ]
        );
    }

    /// Provenance: QSL-3161.
    #[test]
    fn expected_version_reads_current_install_page_wording_and_keeps_old_forms() {
        let current =
            "The v0.1.4 release prints `0.1.4`. Version 0.2.0 also prints the pinned FSL revision.";
        assert_eq!(expected_version(current).as_deref(), Some("0.2.0"));
        assert_eq!(
            expected_version("FSL v0.2.0 release").as_deref(),
            Some("0.2.0")
        );
        assert_eq!(
            expected_version("The latest published driver release is v0.2.0.").as_deref(),
            Some("0.2.0")
        );
    }

    /// Provenance: QSL-3161.
    #[test]
    fn expected_version_requires_a_documented_version() {
        assert_eq!(expected_version("The FSL release is ready."), None);
        assert_eq!(expected_version("Version current is pending."), None);
    }

    #[test]
    fn released_glibc_policy_accepts_up_to_243_and_rejects_private_or_newer() {
        assert_eq!(
            validate_glibc_version_info("Version needs: GLIBC_2.3 GLIBC_2.43"),
            Ok((2, 43, 0))
        );
        assert!(validate_glibc_version_info("GLIBC_2.44").is_err());
        assert!(validate_glibc_version_info("GLIBC_2.40 GLIBC_PRIVATE")
            .unwrap_err()
            .contains("GLIBC_PRIVATE"));
        assert!(validate_glibc_version_info("no symbol versions").is_err());
    }

    #[test]
    fn recorded_outcome_projection_matches_examples_runner_shape() {
        let raw = serde_json::json!({
            "format": "filament-outcome/1",
            "category": "refusal",
            "last_stage": "S2",
            "result": null,
            "diagnostics": [{
                "code": "wrong_snapshot",
                "cause": "wrong-observation",
                "message": "message text is not projected"
            }],
            "items": []
        });
        assert_eq!(
            project_recorded_outcome(&raw),
            serde_json::json!({
                "category": "refusal",
                "last_stage": "S2",
                "result": null,
                "first_diagnostic": {
                    "code": "wrong_snapshot",
                    "cause": "wrong-observation"
                },
                "items": []
            })
        );
        assert_eq!(
            project_recorded_outcome(&serde_json::json!({ "format": "other" })),
            serde_json::json!({ "outcome": null })
        );
    }

    #[test]
    fn version_output_compares_first_line_only() {
        let actual = "0.2.0\nFSL revision: build-specific\n";
        assert!(matches_version_output(actual, "0.2.0", None));
        assert!(matches_version_output(
            actual,
            "0.2.0",
            Some(&["0.2.0".to_owned()])
        ));
        assert!(!matches_version_output(actual, "0.2.1", None));
        assert!(!matches_version_output("FSL revision: xyz", "0.2.0", None));
    }

    #[test]
    fn step_blocks_reads_named_bash_blocks() {
        let blocks =
            step_blocks("```bash step=one\necho one\n```\n```bash step=two\necho two\n```\n");
        assert_eq!(blocks["one"], "echo one");
        assert_eq!(blocks["two"], "echo two");
    }

    #[test]
    fn literal_output_normalization_preserves_indentation() {
        assert_eq!(normalize_lines("  first  \nsecond\n"), "  first\nsecond");
    }

    #[test]
    fn file_excerpt_match_requires_contiguous_lines_and_trims_only_trailing_space() {
        let file = "before\n  first  \nsecond\nafter\n";
        assert!(contains_contiguous_lines(file, "  first\nsecond  "));
        assert!(!contains_contiguous_lines(file, "first\nsecond"));
        assert!(!contains_contiguous_lines(file, "  first\nafter"));
    }

    #[test]
    fn trim_rule_matches_the_docs_source_part() {
        let source = "// explanation\nimport shop\ninvariant R on Shop::Order { true }\ninvariant Other { false }\n";
        assert_eq!(
            trim_rule(source, "R"),
            "invariant R on Shop::Order { true }"
        );
    }
}
