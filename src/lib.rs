use cpython::{Python, PyResult, py_module_initializer, py_fn};

fn gen_init(variables_str: &str) -> String {
    let tab = "    ";
    let variables = variables_str.split(',');
    let mut res = String::new();
    for var in variables {
        let var = var.trim();
        let line = format!("{}self._{} = {}\n", tab, var, var);
        res.push_str(&line);
    }
    res
}

fn gen_init_py(_:Python, variables_str: &str) -> PyResult<String> {
    Ok(gen_init(variables_str))
}

py_module_initializer!(rustsnippetsutils, |py, m| {
    m.add(py, "__doc__", "Module written in rust for use in inline-python code snippets")?;
    m.add(py, "gen_init", py_fn!(py, gen_init_py(variables_str: &str)))?;
    Ok(())
});
