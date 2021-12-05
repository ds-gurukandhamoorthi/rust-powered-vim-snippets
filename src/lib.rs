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

    let starts_with_duration = Regex::new(r"^[0-9:]+;").unwrap();

    let grep = |contents: &str, pat: &str| -> Vec<String> {
        contents.lines().rev()
            .filter(|line| line.contains(&pat))
            .filter(|line| duration.is_empty() || starts_with_duration.is_match(line))
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    };

    let lines = files.into_iter()
        .filter_map(|file| fs::read_to_string(file).ok())
        .flat_map(|filecontent| grep(&filecontent, pattern))
        ;

    let one_relev_line = lines.take(1).next();

    let res = match one_relev_line {
        Some(line) => if duration.is_empty() {
                    line
                }else{
                    let regex = Regex::new(r"^[^;]+;").unwrap();
                    let line = regex.replace(&line, "");
                    format!("{};{}", generate_duration(duration), line)
                },
        _ => "".to_string(),
    };
    res
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

fn special_time_diff(timerange: &str) -> String {
    let mut iter = timerange.trim().splitn(2, ' ')
        .map(|x| x.parse::<u16>().unwrap())
        .map(|dur| ((dur / 100) * 60, dur % 100))
        .inspect(|&(_, mins)| assert!(mins < 60, "minutes in duration must be less than 60"))
        .map(|(hr, mn)| hr + mn);

    let start = iter.next().unwrap();
    let end = iter.next().unwrap();
    let diff = if start < end {
        end - start
    } else {
        24 * 60 - start + end
    };
    let (hour, min) = (diff / 60, diff % 60);
    let duration = format!("{}:{:02}", hour, min);
    let start = format!("{}:{:02}", start / 60, start % 60);
    let end = format!("{}:{:02}", end / 60, end % 60);
    format!("{}-{}={}", start, end, duration)
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

fn special_time_diff_py(_: Python, timerange: &str) -> PyResult<String> {
    Ok(special_time_diff(timerange))
}

py_module_initializer!(rustsnippetsutils, |py, m| {
    m.add(py, "__doc__", "Module written in rust for use in inline-python code snippets")?;
    m.add(py, "gen_init", py_fn!(py, gen_init_py(variables_str: &str)))?;
    m.add(py, "get_last_read_argument", py_fn!(py, get_last_read_argument_py(line: &str)))?;
    m.add(py, "get_recent_line_containing_pattern", py_fn!(py, get_recent_line_containing_pattern_py(direc: &str, pattern: &str, duration: &str)))?;
    m.add(py, "generate_duration", py_fn!(py, generate_duration_py(abbref: &str)))?;
    m.add(py, "special_time_diff", py_fn!(py, special_time_diff_py(timerange: &str)))?;
    Ok(())
});
