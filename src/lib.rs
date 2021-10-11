use cpython::{Python, PyResult, py_module_initializer, py_fn};
use regex::Regex;

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

fn get_last_read_argument_py(_: Python, line: &str) -> PyResult<i32> {
    Ok(get_last_read_argument(line))
}

fn gen_init_py(_:Python, variables_str: &str) -> PyResult<String> {
    Ok(gen_init(variables_str))
}

py_module_initializer!(rustsnippetsutils, |py, m| {
    m.add(py, "__doc__", "Module written in rust for use in inline-python code snippets")?;
    m.add(py, "gen_init", py_fn!(py, gen_init_py(variables_str: &str)))?;
    m.add(py, "get_last_read_argument", py_fn!(py, get_last_read_argument_py(line: &str)))?;
    Ok(())
});
