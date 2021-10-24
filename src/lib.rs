use cpython::{Python, PyResult, py_module_initializer, py_fn};
use regex::Regex;
use std::fs;

fn gen_init(variables_str: &str) -> String {
    let tab = "    ";
    let variables = variables_str.split(',');
    let mut res = String::new();
    for var in variables {
        let var = var.trim();
        let line = format!("{}self._{} = {}\n{}", tab, var, var, tab);
        res.push_str(&line);
    }
    res
}

fn get_last_read_argument(line: &str) -> i32 {
    let regexp = Regex::new(r"argv\[(\d+)\]").unwrap();
    let mut caps = regexp.captures_iter(line);

    let num = caps.next();
    let nb_args_upto_this_line = match num {
        Some(i) => i[1].parse::<i32>().unwrap(),
        _ => 0,
    };
    nb_args_upto_this_line + 1
}

fn get_recent_line_containing_pattern(direc: &str, pattern: &str, duration: &str) -> String{
    let mut files: Vec<_> = fs::read_dir(direc).unwrap()
        .map(|file| file.unwrap().path())
        .collect();


    files.sort_by_key(|a| fs::metadata(a).unwrap()
        .modified().unwrap()
        .elapsed().unwrap()
        .as_secs() as i128
    );

    let contents_each_file = files.into_iter()
        .filter_map(|file| fs::read_to_string(file).ok());

    for content in contents_each_file{
        for line in content.lines().rev(){
            if line.contains(&pattern) {
                return if duration.is_empty() {
                    line.to_string()
                }else{
                    let regex = Regex::new(r"^[^;]+;").unwrap();
                    let line = regex.replace(line, "");
                    format!("{};{}", generate_duration(duration), line.to_string())
                };
            }
        }
    }

    "".to_string()
}

// 43 -> 43:00
fn generate_duration(abbrev: &str) -> String {
    let dur = abbrev.parse::<u16>();
    match dur {
        Ok(hour) if hour < 5 && hour > 0  => format!("{}:00:00", hour),
        Ok(min) if min < 60 && min > 7  => format!("{}:00", min),
        Ok(hourmin) if hourmin > 100 => format!("{}:{:02}:00", hourmin / 100, hourmin % 100),
        _ => "UNKNOWN".to_string(),
    }
}

fn get_last_read_argument_py(_: Python, line: &str) -> PyResult<i32> {
    Ok(get_last_read_argument(line))
}

fn gen_init_py(_:Python, variables_str: &str) -> PyResult<String> {
    Ok(gen_init(variables_str))
}

fn get_recent_line_containing_pattern_py(_: Python, direc: &str, pattern: &str, duration: &str) -> PyResult<String> {
    Ok(get_recent_line_containing_pattern(direc, pattern, duration))
}

fn generate_duration_py(_: Python, abbrev: &str) -> PyResult<String> {
    Ok(generate_duration(abbrev))
}

py_module_initializer!(rustsnippetsutils, |py, m| {
    m.add(py, "__doc__", "Module written in rust for use in inline-python code snippets")?;
    m.add(py, "gen_init", py_fn!(py, gen_init_py(variables_str: &str)))?;
    m.add(py, "get_last_read_argument", py_fn!(py, get_last_read_argument_py(line: &str)))?;
    m.add(py, "get_recent_line_containing_pattern", py_fn!(py, get_recent_line_containing_pattern_py(direc: &str, pattern: &str, duration: &str)))?;
    m.add(py, "generate_duration", py_fn!(py, generate_duration_py(abbref: &str)))?;
    Ok(())
});
