use crate::datapack::Function;

pub struct Runtime {
    pub init: String,
    pub functions: Vec<Function>,
}

pub fn setup_runtime() -> Runtime {
    Runtime {
        init: setup_init(),
        functions: setup_functions(),
    }
}

fn setup_init() -> String {
    let mut init = String::new();

    // Registry
    init.push_str(&format!("scoreboard objectives add registry dummy\n"));
    // Caller saved registers
    for i in 1..=8 {
        init.push_str(&format!("scoreboard players set r{i} registry 0\n"));
    }
    // Callee saved registers
    for i in 1..=8 {
        init.push_str(&format!("scoreboard players set e{i} registry 0\n"));
    }
    // Argument passing registers
    for i in 1..=8 {
        init.push_str(&format!("scoreboard players set a{i} registry 0\n"));
    }

    // Stack
    init.push_str(&format!("scoreboard objectives add stack dummy\n"));
    init.push_str(&format!("scoreboard players set ptr stack 0\n"));
    init.push_str(&format!("scoreboard players set offset stack 0\n"));
    init.push_str(&format!("scoreboard players set item stack 0\n"));
    // Stack values
    for i in 0..32 {
        init.push_str(&format!("scoreboard players set {i} stack 0\n"));
    }
    init
}

fn setup_functions() -> Vec<Function> {
    let mut funcs = Vec::new();
    funcs.push(setup_push());
    funcs.push(setup_pop());

    funcs
}

fn setup_push() -> Function {
    let namespace = "mctest".to_owned();
    let name = "push".to_owned();

    let mut content = String::new();
    content.push_str(&format!(
        "scoreboard players operation tmp stack = ptr stack\n"
    ));
    content.push_str(&format!(
        "scoreboard players operation tmp stack -= offset stack\n"
    ));

    for i in 0..32 {
        content.push_str(&format!("execute if score tmp stack matches {i} run scoreboard players operation {i} stack = item stack\n"));
    }

    Function {
        namespace,
        name,
        content,
    }
}

fn setup_pop() -> Function {
    let namespace = "mctest".to_owned();
    let name = "pop".to_owned();

    let mut content = String::new();
    content.push_str(&format!(
        "scoreboard players operation tmp stack = ptr stack\n"
    ));
    content.push_str(&format!(
        "scoreboard players operation tmp stack -= offset stack\n"
    ));

    for i in 0..32 {
        content.push_str(&format!("execute if score tmp stack matches {i} run scoreboard players operation item stack = {i} stack\n"));
    }

    Function {
        namespace,
        name,
        content,
    }
}
