//! Execute the alpha install and tutorial journeys in a clean container.

use serde_json::Value;
use std::collections::{hash_map::Entry, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const INSTALL_PAGE: &str = "src/content/docs/fsl/guides/set-up-your-environment.md";
const INSTALL_STEPS: &str = "install-test/steps.md";
const TUTORIAL_DIR: &str = "src/content/docs/fsl/guides";

#[derive(Debug)]
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
    id: String,
    system: String,
    view: OutputView,
}

#[derive(Debug)]
struct Case {
    command: String,
    expected_exit: i32,
}

#[derive(Debug)]
struct LiteralCommand {
    page: PathBuf,
    system: String,
    command: String,
    expected: String,
    expected_exit: i32,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("alpha journey gate: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let (docs, examples, candidate, base_image) = arguments()?;
    let install_page_path = docs.join(INSTALL_PAGE);
    let install_page = read(&install_page_path)?;
    let install_source = read(&examples.join(INSTALL_STEPS))?;
    let install_steps = step_blocks(&install_source);
    let selected_steps = install_step_ids(&install_page, candidate.is_some());

    if selected_steps.is_empty() {
        return Err(format!(
            "{} has no install step references",
            install_page_path.display()
        ));
    }
    let install_script =
        assemble_install_script(&selected_steps, &install_steps, &install_page_path)?;
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
    let tutorial_pages = tutorial_pages(&docs, &docs.join(TUTORIAL_DIR))?;
    let mut examples_list = Vec::new();
    let mut literal_commands = Vec::new();
    for page in &tutorial_pages {
        let content = read(&docs.join(page))?;
        let page_examples = page_examples(page, &content);
        if page_examples.is_empty() {
            return Err(format!(
                "{} has no checked command example blocks",
                docs.join(page).display()
            ));
        }
        if let Some(system) = page_examples.first().map(|example| example.system.clone()) {
            literal_commands.extend(page_literal_commands(page, &system, &content));
        }
        examples_list.extend(page_examples);
    }
    if examples_list.is_empty() {
        return Err(format!(
            "{} contains no tutorial command examples",
            docs.join(TUTORIAL_DIR).display()
        ));
    }
    let mut cases_by_system = HashMap::new();
    for example in &examples_list {
        if let Entry::Vacant(entry) = cases_by_system.entry(example.system.clone()) {
            let manifest = read(
                &examples
                    .join("examples")
                    .join(&example.system)
                    .join("example.toml"),
            )?;
            entry.insert(manifest_cases(&manifest));
        }
    }

    let container = start_container(&examples, candidate.as_deref(), &base_image)?;
    let install = exec(&container, "bash", &["-e", "-c", &install_script]);
    if !install.status.success() {
        let output = output_text(&install);
        let failed_step = failed_step(&output).unwrap_or("install".to_owned());
        fail_line(
            &install_page_path,
            &failed_step,
            "install step exits 0",
            &format!("exit {}: {}", status_code(&install), one_line(&output)),
        );
        return Err("installation journey failed".to_owned());
    }
    if let Some(candidate) = candidate.as_deref() {
        let install_dir = exec(&container, "mkdir", &["-p", "/root/.local/bin"]);
        if !install_dir.status.success() {
            fail_line(
                &install_page_path,
                "install candidate binary",
                "exit 0",
                &one_line(&output_text(&install_dir)),
            );
            return Err("could not create fsl install directory".to_owned());
        }
        let install_binary = exec(
            &container,
            "install",
            &["-m", "755", "/candidate-fsl", "/root/.local/bin/fsl"],
        );
        if !install_binary.status.success() {
            fail_line(
                &install_page_path,
                &candidate.display().to_string(),
                "install as fsl",
                &one_line(&output_text(&install_binary)),
            );
            return Err("could not install candidate fsl binary".to_owned());
        }
    }
    let version = exec(
        &container,
        "bash",
        &["-lc", "export PATH=/root/.local/bin:$PATH && fsl --version"],
    );
    let version_output = output_text(&version);
    let actual_version_lines = version_output.lines().map(str::trim).collect::<Vec<_>>();
    let version_matches = match &expected_version_output {
        Some(expected) => {
            actual_version_lines == expected.iter().map(String::as_str).collect::<Vec<_>>()
        }
        None => actual_version_lines
            .iter()
            .any(|line| *line == expected_version),
    };
    if !version.status.success() || !version_matches {
        fail_line(
            &install_page_path,
            "fsl --version",
            &expected_version_output
                .as_ref()
                .map_or_else(|| expected_version.clone(), |lines| lines.join(" ")),
            &format!(
                "exit {}: {}",
                status_code(&version),
                one_line(&version_output)
            ),
        );
        return Err("installed fsl version differs from the install page".to_owned());
    }
    println!(
        "PASS {} | fsl --version | {expected_version}",
        install_page_path.display()
    );

    let mut tutorial_failed = false;
    for literal in literal_commands {
        let shell_command = format!(
            "cd /candidate-examples/examples/{} && export PATH=/root/.local/bin:$PATH && {}",
            literal.system, literal.command
        );
        let actual = exec(&container, "bash", &["-lc", &shell_command]);
        let actual_text = String::from_utf8_lossy(&actual.stdout).trim().to_owned();
        if status_code(&actual) != literal.expected_exit || actual_text != literal.expected {
            tutorial_failed = true;
            fail_line(
                &docs.join(&literal.page),
                &literal.command,
                &format!(
                    "exit {} and {}",
                    literal.expected_exit,
                    one_line(&literal.expected)
                ),
                &format!(
                    "exit {} and {}",
                    status_code(&actual),
                    one_line(&actual_text)
                ),
            );
        } else {
            println!(
                "PASS {} | {}",
                docs.join(&literal.page).display(),
                literal.command
            );
        }
    }
    for example in examples_list {
        let cases = cases_by_system
            .get(&example.system)
            .ok_or_else(|| format!("no cases loaded for {}", example.system))?;
        let case = cases.get(&example.id).ok_or_else(|| {
            format!(
                "{} references {}/{} but its example.toml has no matching case",
                docs.join(&example.page).display(),
                example.system,
                example.id
            )
        })?;
        let expected_path = examples
            .join("examples")
            .join(&example.system)
            .join("expected")
            .join(format!("{}.json", example.id));
        let expected: Value = serde_json::from_str(&read(&expected_path)?)
            .map_err(|error| format!("{}: {error}", expected_path.display()))?;
        let command_text = format!("fsl {} --format json", case.command);
        let shell_command = format!(
            "cd /candidate-examples/examples/{} && export PATH=/root/.local/bin:$PATH && {}",
            example.system, command_text
        );
        let actual = exec(&container, "bash", &["-lc", &shell_command]);
        let actual_output = output_text(&actual);
        let expected_output = render_output(&expected, case.expected_exit, &example.view)
            .map_err(|error| format!("{}: {}: {error}", example.page.display(), command_text))?;
        let actual_json = serde_json::from_str::<Value>(actual_output.trim()).ok();
        let actual_render = actual_json
            .as_ref()
            .map(|value| render_output(value, status_code(&actual), &example.view));

        if status_code(&actual) != case.expected_exit
            || actual_render
                .as_ref()
                .and_then(|rendered| rendered.as_ref().ok())
                != Some(&expected_output)
        {
            tutorial_failed = true;
            fail_line(
                &docs.join(&example.page),
                &command_text,
                &format!(
                    "exit {} and {}",
                    case.expected_exit,
                    one_line(&expected_output)
                ),
                &format!(
                    "exit {} and {}",
                    status_code(&actual),
                    actual_render.map_or_else(
                        || one_line(&actual_output),
                        |rendered| rendered.map_or_else(
                            |error| format!("invalid output: {error}"),
                            |value| one_line(&value)
                        ),
                    )
                ),
            );
            continue;
        }
        println!(
            "PASS {} | {}",
            docs.join(&example.page).display(),
            command_text
        );
    }
    if tutorial_failed {
        return Err(
            "one or more tutorial command outputs differ from the page examples".to_owned(),
        );
    }
    Ok(())
}

fn arguments() -> Result<(PathBuf, PathBuf, Option<PathBuf>, String), String> {
    let mut args = env::args().skip(1);
    let mut docs = None;
    let mut examples = None;
    let mut candidate = None;
    let mut base_image = "ubuntu:22.04".to_owned();
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value after {arg}"))?;
        match arg.as_str() {
            "--docs" => docs = Some(PathBuf::from(value)),
            "--examples" => examples = Some(PathBuf::from(value)),
            "--fsl-binary" => candidate = Some(PathBuf::from(value)),
            "--base-image" => base_image = value,
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    let docs = docs.ok_or_else(|| {
        "usage: alpha-journey-gate --docs PATH --examples PATH [--fsl-binary PATH] [--base-image IMAGE]".to_owned()
    })?;
    let examples = examples.ok_or_else(|| {
        "usage: alpha-journey-gate --docs PATH --examples PATH [--fsl-binary PATH] [--base-image IMAGE]".to_owned()
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
            path.canonicalize()
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .transpose()?;
    Ok((docs, examples, candidate, base_image))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn install_step_ids(page: &str, candidate: bool) -> Vec<String> {
    let mut ids = Vec::new();
    for line in page.lines() {
        let Some(reference) = line.trim().strip_prefix("```steps ") else {
            continue;
        };
        for id in reference.split_whitespace() {
            if !matches!(
                id,
                "rust" | "driver-source" | "check" | "check-release" | "check-source"
            ) && !ids.iter().any(|existing| existing == id)
            {
                if candidate
                    && matches!(
                        id,
                        "node"
                            | "build-tools"
                            | "driver-target"
                            | "driver-download"
                            | "driver-verify"
                            | "driver-install"
                    )
                {
                    continue;
                }
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
                return Some(version.to_owned());
            }
        }
        if let Some(version) = line.strip_prefix("The latest published driver release is v") {
            if let Some(version) = version.split_whitespace().next() {
                return Some(version.to_owned());
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

fn page_examples(page: &Path, source: &str) -> Vec<Example> {
    let mut examples = Vec::new();
    for line in source.lines() {
        let Some(reference) = line.trim().strip_prefix("```example ") else {
            continue;
        };
        let mut parts = reference.split_whitespace();
        let Some(key) = parts.next() else { continue };
        if !reference.contains("show=\"command\"") {
            continue;
        }
        let Some((system, id)) = key.split_once('/') else {
            continue;
        };
        let view = if reference.split_whitespace().any(|part| part == "summary") {
            OutputView::Summary
        } else if let Some(fields) = reference
            .split_once("fields=\"")
            .and_then(|(_, value)| value.split_once('"').map(|(fields, _)| fields))
        {
            OutputView::Fields(fields.split_whitespace().map(str::to_owned).collect())
        } else {
            OutputView::Summary
        };
        examples.push(Example {
            page: page.to_path_buf(),
            id: id.to_owned(),
            system: system.to_owned(),
            view,
        });
    }
    examples
}

fn page_literal_commands(page: &Path, system: &str, source: &str) -> Vec<LiteralCommand> {
    let mut commands = Vec::new();
    let mut in_text_block = false;
    let mut lines: Vec<&str> = Vec::new();
    for line in source.lines() {
        if line.trim() == "```text" {
            in_text_block = true;
            lines.clear();
        } else if line.trim() == "```" && in_text_block {
            if let Some(command) = lines.first().map(|line| line.trim()).filter(|line| {
                ["fsl ", "cat ", "sed "]
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
            }) {
                let expected = lines
                    .iter()
                    .skip(1)
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                if !expected.is_empty() {
                    let expected_exit = expected
                        .split("exit ")
                        .nth(1)
                        .and_then(|tail| tail.split(|ch: char| !ch.is_ascii_digit()).next())
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(0);
                    commands.push(LiteralCommand {
                        page: page.to_path_buf(),
                        system: system.to_owned(),
                        command: command.to_owned(),
                        expected,
                        expected_exit,
                    });
                }
            }
            in_text_block = false;
        } else if in_text_block {
            lines.push(line);
        }
    }
    commands
}

fn manifest_cases(source: &str) -> HashMap<String, Case> {
    let mut cases = HashMap::new();
    for section in source.split("[[case]]").skip(1) {
        let id = quoted_field(section, "id");
        let command = quoted_field(section, "command");
        let expected_exit = integer_field(section, "expect_exit");
        if let (Some(id), Some(command), Some(expected_exit)) = (id, command, expected_exit) {
            cases.insert(
                id,
                Case {
                    command,
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
        value.strip_suffix('"').map(str::to_owned)
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

fn exec(container: &Container, program: &str, args: &[&str]) -> Output {
    Command::new("docker")
        .arg("exec")
        .arg(&container.0)
        .arg(program)
        .args(args)
        .output()
        .expect("docker exec should start")
}

fn status_code(output: &Output) -> i32 {
    output.status.code().unwrap_or(128)
}

fn output_text(output: &Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text
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
