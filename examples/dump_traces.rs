// Simple example of showing how to use the rust API to
// print out stack traces from a python program
use failure::{Error, ResultExt};
use py_spy::{StackTrace, Frame};

extern crate py_spy;
extern crate remoteprocess;
extern crate failure;
#[macro_use]
extern crate log;
extern crate env_logger;

fn print_python_stacks(pid: remoteprocess::Pid) -> Result<(), failure::Error> {
    // Create a new PythonSpy object with the default config options
    let config = py_spy::Config::default();
    let mut process = py_spy::PythonSpy::new(pid, &config)?;

    // Create a stack unwind object, and use it to get the stack for each thread
    let unwinder = process.process.unwinder()?;

    for thread in process.process.threads()?.iter() {
        println!("______Thread {} - {}", thread.id()?, if thread.active()? { "running" } else { "idle" });

        // lock the thread to get a consistent snapshot (unwinding will fail otherwise)
        // Note: the thread will appear idle when locked, so we are calling
        // thread.active() before this
        let _lock = thread.lock()?;

        // Iterate over the callstack for the current thread
        for ip in unwinder.cursor(&thread)? {
            let ip = ip?;
            // Lookup the current stack frame containing a filename/function/linenumber etc
            // for the current address
            unwinder.symbolicate(ip, true, &mut |sf| {
                println!("\t{}", sf);
            })?;
        }
    }

    // get stack traces for each thread in the process
    let traces = process.get_stack_traces()?;

    // Print out the python stack for each thread
    for trace in traces {
        println!("Thread {:#X} ({})", trace.thread_id, trace.status_str());

        for frame in &trace.frames {
            let addr = {
                match &frame.frame_ptr {
                    Some(inner)=> format!("{}", inner),
                    None               => "None".to_string(),
                }
            };
            println!("\t {} {} ({}:{})", addr, frame.name, frame.filename, frame.line);
        }
    }
    Ok(())
}

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    let pid = if args.len() > 1 {
        args[1].parse().expect("invalid pid")
    } else {
        error!("you must specify a pid!");
        return;
    };

    if let Err(e) = print_python_stacks(pid) {
        error!("failed to print stack traces: {:?}", e);
    }
}
