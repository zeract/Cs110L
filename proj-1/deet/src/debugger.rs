use core::panic;
use nix::sys::ptrace;
use crate::debugger_command::DebuggerCommand;
use crate::inferior::Inferior;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::inferior::Status;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data:DwarfData,
    breakpoint:Vec<usize>,
}
#[derive(Clone)]
pub struct Breakpoint {
    pub addr: usize,
    pub orig_byte: u8,
}
impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        debug_data.print();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data:debug_data,
            breakpoint:Vec::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if self.inferior.is_some(){
                        self.inferior.as_mut().unwrap().kill();
                    }
                    if let Some(inferior) = Inferior::new(&self.target, &args,&self.breakpoint) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        let status =  self.inferior.as_mut().unwrap().inferior_continue().ok().unwrap();
                        match status{
                            Status::Stopped(signal,pointer) => 
                            {   println!("Child stopped by signal {}",signal);
                                let file = self.debug_data.get_line_from_addr(pointer).unwrap();
                                println!("Stopped at {}:{}",file.file,file.number);
                            },
                            Status::Exited(exit_code) => {println!("Child exited (status {})",exit_code);
                                                        self.inferior = None;},
                            Status::Signaled(signal) => {println!("Child killed by signal {}",signal);
                                                        self.inferior = None;},
                            other => panic!("continue return wrong!"),
                        }
                        
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    self.inferior.as_mut().unwrap().kill();
                    return;
                }
                DebuggerCommand::Continue =>{
                    
                    if self.inferior.is_none(){
                        panic!("The process is not run");
                    }
                    let status =  self.inferior.as_mut().unwrap().inferior_continue().ok().unwrap();
                    match status{
                        Status::Exited(exit_code) => {
                            println!("Child exited (status {})", exit_code);
                            self.inferior = None;
                        }
                        Status::Signaled(signal) => {
                            println!("Child exited due to signal {}", signal);
                            self.inferior = None;
                        }
                        Status::Stopped(signal, rip) => {
                            println!("Child stopped (signal {})", signal);
                            let _line = self.debug_data.get_line_from_addr(rip);
                            let _func = self.debug_data.get_function_from_addr(rip);
                            if _line.is_some() && _func.is_some() {
                                println!("Stopped at {} ({})", _func.unwrap(), _line.unwrap());
                            }
                        }
                    }
                }
                DebuggerCommand::Backtrace =>{
                    if self.inferior.is_some(){              
                        self.inferior.as_ref().unwrap().print_backtrace(&self.debug_data);
                    }else{
                        panic!("doesn't have a running process");
                    }
                }
                DebuggerCommand::Break(arg) =>{
                    if arg.starts_with('*'){
                        let point = &arg[1..];
                        let mut address = Self::parse_address(point);
                        println!("Set breakpoint {} at {:#x}",self.breakpoint.len(),address.unwrap());
                        self.breakpoint.push(address.unwrap());
                        if self.inferior.is_some(){
                            match self.inferior.as_mut().unwrap().write_byte(address.unwrap(), 0xcc){
                                Ok(orig_byte) => { self.inferior.as_mut().unwrap().breakpoint.insert(address.unwrap(), Breakpoint { addr: address.unwrap(), orig_byte: orig_byte });}
                                Err(_) => println!("Invalid breakpoint address {:#x}",address.unwrap()),
                            }
                            
                        }
                    }else{
                        
                        let line = Self::parse_address(arg.as_str());
                        match line{
                            Some(number) => {
                                
                                let address = self.debug_data.get_addr_for_line(None, arg.parse::<usize>().ok().unwrap());
                                println!("Set breakpoint {} at {:#x}",self.breakpoint.len(),address.unwrap());
                                self.breakpoint.push(address.unwrap());
                                if self.inferior.is_some(){
                                    match self.inferior.as_mut().unwrap().write_byte(address.unwrap(), 0xcc){
                                        Ok(orig_byte) => { self.inferior.as_mut().unwrap().breakpoint.insert(address.unwrap(), Breakpoint { addr: address.unwrap(), orig_byte: orig_byte });}
                                        Err(_) => println!("Invalid breakpoint address {:#x}",address.unwrap()),
                                    }
                                    
                                }
                            },
                            None => {
                                let address = self.debug_data.get_addr_for_function(None, &arg);
                                println!("Set breakpoint {} at {:#x}",self.breakpoint.len(),address.unwrap());
                                self.breakpoint.push(address.unwrap());
                                if self.inferior.is_some(){
                                    match self.inferior.as_mut().unwrap().write_byte(address.unwrap(), 0xcc){
                                        Ok(orig_byte) => { self.inferior.as_mut().unwrap().breakpoint.insert(address.unwrap(), Breakpoint { addr: address.unwrap(), orig_byte: orig_byte });}
                                        Err(_) => println!("Invalid breakpoint address {:#x}",address.unwrap()),
                                    }
                                    
                                }
                            },
                        }
                        
                        
                    }
                    
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }

    fn parse_address(addr: &str) -> Option<usize> {
        let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
            &addr[2..]
        } else {
            &addr
        };
        usize::from_str_radix(addr_without_0x, 16).ok()
    }
}
