
pub enum DebuggerCommand {
    Quit,
    Run(Vec<String>),
    Continue,
    Backtrace,
    Break(String),
}

impl DebuggerCommand {
    pub fn from_tokens(tokens: &Vec<&str>) -> Option<DebuggerCommand> {
        match tokens[0] {
            "q" | "quit" => Some(DebuggerCommand::Quit),
            "r" | "run" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Run(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            "c" | "continue" | "cont" => Some(DebuggerCommand::Continue),
            "bt"|"back"|"backtrace" => Some(DebuggerCommand::Backtrace),
            "break"|"b" => {
                Some(DebuggerCommand::Break(tokens[1].to_string()))
            },
            // Default case:
            _ => None,
        }
    }
}
