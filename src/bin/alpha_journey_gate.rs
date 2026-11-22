use serde_json::Value;
use std::collections::{hash_map::Entry, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const INSTALL_PAGE: &str = "src/content/docs/fsl/guides/set-up-your-environment.md";
const FIRST_RULE_PAGE: &str = "src/content/docs/fsl/learn/first-rule.md";
const INSTALL_STEPS: &str = "install-test/steps.md";

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
    id: String,
    system: String,
    fields: Vec<String>,
}

#[derive(Debug)]
struct Case {
    command: String,
    expected_exit: i32,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("alpha journey gate: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let (docs, examples) = arguments()?;
    let install_page_path = docs.join(INSTALL_PAGE);
    let tutorial_page_path = docs.join(FIRST_RULE_PAGE);
    let install_page = read(&install_page_path)?;
    let tutorial_page = read(&tutorial_page_path)?;
    let install_source = read(&examples.join(INSTALL_STEPS))?;
    let install_steps = step_blocks(&install_source);
    let selected_steps = install_step_ids(&install_page);

    if selected_steps.is_empty() {
        return Err(format!(
            "{} has no install step references",
            install_page_path.display()
        ));
    }
    let install_script =
        assemble_install_script(&selected_steps, &install_steps, &install_page_path)?;
    let expected_version = expected_version(&install_page).ok_or_else(|| {
        format!(
            "{} does not state the expected fsl version",
            install_page_path.display()
        )
    })?;

    let examples_list = page_examples(&tutorial_page);
    if examples_list.is_empty() {
        return Err(format!(
            "{} has no rendered command examples",
            tutorial_page_path.display()
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

    let container = start_container(&examples)?;
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
    let version_output = output_text(&install);
    if !version_output
        .lines()
        .any(|line| line.trim() == expected_version)
    {
        fail_line(
            &install_page_path,
            "fsl --version",
            &format!("version {expected_version}"),
            &one_line(&version_output),
        );
        return Err("installed version differs from the install page".to_owned());
    }
    println!(
        "PASS {} | fsl --version | {expected_version}",
        install_page_path.display()
    );

    let mut tutorial_failed = false;
    for example in examples_list {
        let cases = cases_by_system
            .get(&example.system)
            .ok_or_else(|| format!("no cases loaded for {}", example.system))?;
        let case = cases.get(&example.id).ok_or_else(|| {
            format!(
                "{} references {}/{} but its example.toml has no matching case",
                tutorial_page_path.display(),
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
        let expected_output = projected_outcome(&expected, &example.fields);
        let actual_json = serde_json::from_str::<Value>(actual_output.trim()).ok();
        let actual_projection = actual_json
            .as_ref()
            .map(|value| projected_outcome(value, &example.fields));

        if status_code(&actual) != case.expected_exit
            || actual_projection.as_ref() != Some(&expected_output)
        {
            tutorial_failed = true;
            fail_line(
                &tutorial_page_path,
                &command_text,
                &format!("exit {} and {}", case.expected_exit, expected_output),
                &format!(
                    "exit {} and {}",
                    status_code(&actual),
                    actual_projection
                        .map_or_else(|| one_line(&actual_output), |value| value.to_string())
                ),
            );
            continue;
        }
        println!("PASS {} | {}", tutorial_page_path.display(), command_text);
    }
    if tutorial_failed {
        return Err(
            "one or more tutorial command outputs differ from the page examples".to_owned(),
        );
    }
    Ok(())
}

fn arguments() -> Result<(PathBuf, PathBuf), String> {
    let mut args = env::args().skip(1);
    let mut docs = None;
    let mut examples = None;
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value after {arg}"))?;
        match arg.as_str() {
            "--docs" => docs = Some(PathBuf::from(value)),
            "--examples" => examples = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    let docs =
        docs.ok_or_else(|| "usage: alpha-journey-gate --docs PATH --examples PATH".to_owned())?;
    let examples = examples
        .ok_or_else(|| "usage: alpha-journey-gate --docs PATH --examples PATH".to_owned())?;
    if !docs.is_dir() || !examples.is_dir() {
        return Err("--docs and --examples must name existing checkouts".to_owned());
    }
    let docs = docs
        .canonicalize()
        .map_err(|error| format!("{}: {error}", docs.display()))?;
    let examples = examples
        .canonicalize()
        .map_err(|error| format!("{}: {error}", examples.display()))?;
    Ok((docs, examples))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn install_step_ids(page: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for line in page.lines() {
        let Some(reference) = line.trim().strip_prefix("```steps ") else {
            continue;
        };
        for id in reference.split_whitespace() {
            if !matches!(
                id,
                "rust" | "driver-source" | "check-release" | "check-source"
            ) && !ids.iter().any(|existing| existing == id)
            {
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

fn page_examples(page: &str) -> Vec<Example> {
    let mut examples = Vec::new();
    for line in page.lines() {
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
        let fields = reference
            .split_once("fields=\"")
            .and_then(|(_, value)| value.split_once('"').map(|(fields, _)| fields))
            .unwrap_or("clause result category")
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        examples.push(Example {
            id: id.to_owned(),
            system: system.to_owned(),
            fields,
        });
    }
    examples
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

fn projected_outcome(value: &Value, fields: &[String]) -> Value {
    let Some(item) = value
        .get("items")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
    else {
        return Value::Null;
    };
    let mut projected = serde_json::Map::new();
    if let Some(axes) = item.get("axes") {
        projected.insert("axes".to_owned(), axes.clone());
    }
    for field in fields {
        let found = match field.as_str() {
            "category" => value.get("category"),
            "clause" | "result" => item.get(field),
            _ => item.get("axes").and_then(|axes| axes.get(field)),
        };
        if let Some(found) = found {
            projected.insert(field.clone(), found.clone());
        }
    }
    Value::Object(projected)
}

fn start_container(examples: &Path) -> Result<Container, String> {
    let mount = format!(
        "type=bind,src={},dst=/candidate-examples,readonly",
        examples.display()
    );
    let output = Command::new("docker")
        .args([
            "run",
            "-d",
            "--rm",
            "--mount",
            &mount,
            "ubuntu:22.04",
            "sleep",
            "infinity",
        ])
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
