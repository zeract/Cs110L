use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::process::Child;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::mem::size_of;
use std::collections::HashMap;
use crate::debugger::Breakpoint;
use crate::dwarf_data::DwarfData;

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}
fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}
pub struct Inferior {
    child: Child,
    pub breakpoint: HashMap<usize,Breakpoint>,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>,breakpoint:&Vec<usize>) -> Option<Inferior> {
        // TODO: implement me!
        let  mut cmd = Command::new(target);
        cmd.args(args);
        unsafe{
            cmd.pre_exec(child_traceme);
        }
        let mut child =  cmd.spawn().expect("fail to excute process");
        //println!(
        //    "Inferior::new not implemented! target={}, args={:?}",
        //    target, args
        //);
        
        let mut inferior = Inferior{child:child ,breakpoint:HashMap::new()};
        let result =  inferior.wait(None).ok()?;
        for address in breakpoint.iter(){
            match inferior.write_byte(*address, 0xcc){
                Ok(orig_byte) => {inferior.breakpoint.insert(*address, Breakpoint { addr: *address, orig_byte: orig_byte });}
                Err(_) => println!("Invalid breakpoint address {:#x}",address),
            }
            
        }
        Some(inferior)
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    pub fn inferior_continue(&mut self) -> Result<Status,nix::Error>{
        let mut regs = ptrace::getregs(self.pid())?;
        let rip = regs.rip as usize;
        if self.breakpoint.contains_key(&(rip-1)){
            //okknni println!("stopped at a breakpoint");
            let orig = self.breakpoint.get(&(rip-1)).unwrap().orig_byte;
            self.write_byte(rip-1, orig)?;

            regs.rip = (rip-1)  as u64;
            ptrace::setregs(self.pid(), regs).unwrap();

            ptrace::step(self.pid(), None).unwrap();
            
            match self.wait(None).unwrap(){
                Status::Exited(exit_code) => return Ok(Status::Exited(exit_code)), 
                Status::Signaled(signal) => return Ok(Status::Signaled(signal)),
                Status::Stopped(_, _) => {
                    // restore 0xcc in the breakpoint location
                    self.write_byte(rip - 1, 0xcc).unwrap();
                }
            }
        }
        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }
    pub fn kill(&mut self){
        let pid = self.pid();
        self.child.kill().expect("kill command wasn't running");
        waitpid(pid,None);
        println!("Killing running inferior (pid {})",self.pid());
    }

    pub fn print_backtrace(&self, data:&DwarfData) -> Result<(), nix::Error>{
        let regs =  ptrace::getregs(self.pid()).ok().unwrap();
        let mut instruction_ptr = regs.rip as usize;
        let mut base_ptr = regs.rbp as usize;
        while true{
            let file = data.get_line_from_addr(instruction_ptr);
            let func = data.get_function_from_addr(instruction_ptr);
            println!("{} ({}:{})",func.as_ref().unwrap(),file.as_ref().unwrap().file,file.as_ref().unwrap().number);
            if func.as_ref().unwrap()=="main"{
                break;
            }
            instruction_ptr = ptrace::read(self.pid(),(base_ptr+8) as ptrace::AddressType)? as usize ;
            base_ptr  = ptrace::read(self.pid(), base_ptr as ptrace::AddressType )? as usize;
        }
        
        Ok(())
    }

    pub fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }
}
