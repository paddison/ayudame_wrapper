/// A debugging application to test out ayu_events functions

// Should be able to send any event to ayudame at any time,
// CLI app
// create task ids, function ids, etc with counters

use std::fmt::Display;
use std::{io, convert::TryFrom};

use ayudame_wrapper::{InputTypes, AppState};
use ayudame_wrapper::helper_macros::match_or_continue;

const PARSE_UNSIGNED_ERROR_MSG: &str = "Invalid input, must be positive numeric";

type Result<T> = std::result::Result<T, UserInputError>;

enum Command {
    AddTask,
    PrintState,
}

#[derive(Debug)]
enum UserInputError {
    TaskIdNotFound(u64),
    AlreadyInitialized(&'static str),
    InvalidFunctionName(String),
    SameTaskDependency,
}

impl Display for UserInputError {
    
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        type E = UserInputError;
        
        let msg = match self {
            E::AlreadyInitialized(init) => format!("{} should only be called once. Will not emit event.", init),
            E::TaskIdNotFound(id) => format!("Task with id: {} not found.", id),
            E::InvalidFunctionName(name) => format!("Invalid Name: {}. Can only contain ASCII characters", name.trim()),
            E::SameTaskDependency => "Parent and Child cannot be the same Task.".to_string(),
        };
        write!(f, "Error while reading input:\n\t{}", msg)
    }
}

impl std::error::Error for UserInputError { }

#[link(name = "ayudame", kind = "dylib")]
extern {
    fn ayu_event_preinit(rt: u64);
    fn ayu_event_init(n_threads: u64);
    fn ayu_event_addtask(task_id: u64, func_id: u64, priority: u64, scope_id: u64);
    fn ayu_event_registerfunction(func_id: u64, name: *mut std::os::raw::c_char);
    fn ayu_event_adddependency(to_id: u64, from_id: u64, memaddr: u64, orig_memaddr: u64);
    fn ayu_event_addtasktoqueue(task_id: u64, thread_id: u64);
    fn ayu_event_preruntask(task_id: u64, thread_id: u64);
    fn ayu_event_runtask(task_id: u64);
    fn ayu_event_postruntask(task_id: u64);
    fn ayu_event_removetask(task_id: u64);
    fn ayu_event_barrier();
    fn ayu_event_waiton(task_id: u64);
    fn ayu_event_finish();
}

fn main() {
    // create event loop
    let mut state = AppState::default();

    let _ = create_pre_init(&mut state);
    let _ = create_init(&mut state);
    
    loop {
        match ask_for_command() {
            Command::AddTask => {
                print_event_types();

                if let Err(e) = handle_user_input(&mut state) {
                    eprintln!("{}", e);
                }
            },
            Command::PrintState => println!("{}", state),
        }
    }
}

fn ask_for_command() -> Command {
    println!("Options:\n\t(a)dd new event\n\t(p)rint current state");
    loop {
        break match get_input().trim() {
            "a" => Command::AddTask,
            "p" => Command::PrintState,
            invalid => {
                eprintln!("Invalid Option: {}, try again", invalid);
                continue;
            },
        }   
    }
}

fn print_event_types() {
    let options_str = 
    "Event Types: 
        0.  PreInit
        1.  Init
        2.  AddTask
        3.  RegisterFunction
        4.  AddDependency
        5.  AddTaskToQueue
        6.  AddTask
        7.  PreRunTask
        8.  RunTask
        9.  PostRunTask
        10. RemoveTask
        11. Barrier
        12. WaitOn
        13. Finish
    ";
    println!("{options_str}");
}

pub fn get_event_type() -> InputTypes { 
    println!("Enter index of Event to send: ");
    
    loop {
        let n = get_numerical_input();

        break match_or_continue!(InputTypes::try_from(n), "Got Invalid Index, try again");
    } 
}

pub fn get_numerical_input() -> u64 {
    loop {
        let input = get_input();
        break match_or_continue!(input.trim().parse::<u64>(), "Got non numeric input, try again");
    }
}

pub fn get_input() -> String {
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("Unable to read user input, aborting...");
        std::process::exit(1);
    }

    input
}

fn handle_user_input(state: &mut AppState) -> Result<()> {
    match get_event_type() {
        InputTypes::PreInit => create_pre_init(state),
        InputTypes::Init => create_init(state),
        InputTypes::AddTask => create_add_task(state),
        InputTypes::RegisterFunction => create_register_function(state),
        InputTypes::AddDependency => create_add_dependency(state),
        InputTypes::AddTaskToQueue => create_add_task_to_queue(state),
        InputTypes::PreRunTask => create_pre_run_task(state),
        InputTypes::RunTask => create_run_task(state),
        InputTypes::PostRunTask => create_post_run_task(state),
        InputTypes::RemoveTask => create_remove_task(state),
        InputTypes::Barrier => create_barrier(state),
        InputTypes::WaitOn => create_wait_on(state),
        InputTypes::Finish => create_finish(state),
    }
}

fn create_pre_init(state: &mut AppState) -> Result<()> {
    if state.is_pre_init {
        return Err(UserInputError::AlreadyInitialized("PreInit"));
    }
    unsafe { 
        ayu_event_preinit(0); 
    }

    state.is_pre_init = true;
    Ok(())
}

fn create_init(state: &mut AppState) -> Result<()> {
    if state.is_init {
        return Err(UserInputError::AlreadyInitialized("Init"));
    }
    unsafe {
        ayu_event_init(2);
    }

    state.is_init = true;

    Ok(())
}

fn create_add_task(state: &mut AppState) -> Result<()>{

    // TODO: Return with error on wrong input
    println!("Specify Task to add: (leave empty for default values");

    println!("Is task critical (default is false)? (y/n)");
    let is_critical = loop {
        match get_input().trim() {
            "y" => break true,
            "n" => break false,
            "" => break false,
            invalid => eprintln!("Invalid option: {}", invalid),
        }
    };

    println!("Enter thread id: (default is 0)");
    let thread_id = loop {
        break match get_input().trim() {
            "" => 0,
            n => match_or_continue!(n.parse::<u64>(), PARSE_UNSIGNED_ERROR_MSG),
        };
    };

    println!("Choose a label for task: ");
    state.list_functions();
    let task = loop {
        let function_id = match get_input().trim() {
            "" => None,
            input => Some(match_or_continue!(input.parse::<u64>(), PARSE_UNSIGNED_ERROR_MSG)),
        };
        break match_or_continue!(state.create_task(is_critical, function_id, thread_id), "Function with provided id not found");
    };

    let (task_id, func_id, priority, scope_id) = task.into_raw_parts();

    unsafe {
        ayu_event_addtask(task_id, func_id, priority, scope_id);
    }

    Ok(())
}

// 
fn create_register_function(state: &mut AppState) -> Result<()> {
    println!("Enter a name for function (empty for default)");
    let name = get_input();
    let function = state.create_function(name.clone()).ok_or(UserInputError::InvalidFunctionName(name))?;

    let (id, name) = function.into_raw_parts();

    unsafe {
        ayu_event_registerfunction(id, name);
    }

    Ok(())
}

fn create_add_dependency(state: &mut AppState) -> Result<()> {
    state.list_tasks();

    println!("Enter parent, then child id");

    let parent_id = specify_task_id(state)?;
    let child_id = specify_task_id(state)?;

    if child_id == parent_id {
        return Err(UserInputError::SameTaskDependency);
    } 

    state.add_dependency(parent_id, child_id);

    unsafe {
        ayu_event_adddependency(parent_id, child_id, 0xffffeeee | parent_id, 0xffffeee | child_id);
    }
    Ok(())
}

fn create_add_task_to_queue(state: & AppState) -> Result<()> {
    state.list_tasks();
    
    let task_id = get_numerical_input();
    let (_, _, _, scope_id) = state.get_task(task_id).ok_or(UserInputError::TaskIdNotFound(task_id))?.into_raw_parts();

    unsafe {
        ayu_event_addtasktoqueue(task_id, scope_id);
    }

    Ok(())
}

fn create_pre_run_task(state: &AppState) -> Result<()> {
    state.list_tasks();
    let task_id = get_numerical_input();

    let (_, _, _, scope_id) = state.get_task(task_id).ok_or(UserInputError::TaskIdNotFound(task_id))?.into_raw_parts();

    unsafe {
        ayu_event_preruntask(task_id, scope_id);
    }

    Ok(())
}

fn create_run_task(state: &AppState) -> Result<()> {
    state.list_tasks();
    let task_id = specify_task_id(state)?;

    unsafe {
        ayu_event_runtask(task_id);
    }

    Ok(())
}

fn create_post_run_task(state: &AppState) -> Result<()> {
    state.list_tasks();
    let task_id = specify_task_id(state)?;

    unsafe {
        ayu_event_postruntask(task_id);
    }

    Ok(())
}

fn create_remove_task(state: &mut AppState) -> Result<()> {
    state.list_tasks();
    let task_id = specify_task_id(state)?;

    state.delete_task(task_id).ok_or(UserInputError::TaskIdNotFound(task_id))?;

    unsafe {
        ayu_event_removetask(task_id);
    }

    Ok(())
}

fn create_barrier(_state: &AppState) -> Result<()> {
    unsafe {
        ayu_event_barrier();
    }

    Ok(())
}

fn create_wait_on(_state: &AppState) -> Result<()> {
    unsafe {
        ayu_event_waiton(0);
    }

    Ok(())
}

fn create_finish(_state: &AppState) -> Result<()> {
    unsafe {
        ayu_event_finish();
    }

    Ok(())
}

fn specify_task_id(state: &AppState) -> Result<u64> {
    println!("Select Task: ");
    let id = get_numerical_input() as u64;
    if !state.does_task_exist(id) {
        Err(UserInputError::TaskIdNotFound(id))
    } else {
        Ok(id)
    }
}