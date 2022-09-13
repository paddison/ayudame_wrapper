#[macro_use]
pub mod helper_macros;
       
use std::{
    fmt::Write,
    os::raw::c_char,
    rc::{Rc, Weak}, cell::{RefCell, RefMut}, fmt::Display,
};

#[derive(Debug)]
pub enum InputTypes {
    PreInit,
    Init,
    AddTask,
    RegisterFunction,
    AddDependency,
    AddTaskToQueue,
    PreRunTask,
    RunTask,
    PostRunTask,
    RemoveTask,
    Barrier,
    WaitOn,
    Finish,
}

impl TryFrom<u64> for InputTypes {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InputTypes::PreInit),
            1 => Ok(InputTypes::Init),
            2 => Ok(InputTypes::AddTask),
            3 => Ok(InputTypes::RegisterFunction),
            4 => Ok(InputTypes::AddDependency),
            5 => Ok(InputTypes::AddTaskToQueue),
            6 => Ok(InputTypes::AddTask),
            7 => Ok(InputTypes::PreRunTask),
            8 => Ok(InputTypes::RunTask),
            9 => Ok(InputTypes::PostRunTask),
            10 => Ok(InputTypes::RemoveTask),
            11 => Ok(InputTypes::Barrier),
            12 => Ok(InputTypes::WaitOn),
            13 => Ok(InputTypes::Finish),
            _ => Err("Invalid index"),
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub is_pre_init: bool,
    pub is_init: bool,
    tasks: Vec<Rc<Task>>,
    functions: Vec<Rc<Function>>,
    task_id_count: u64,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            is_pre_init: false,
            is_init: false,
            tasks: Vec::new(),
            functions: Vec::new(),
            task_id_count: 0,
        }
    }

    pub fn list_functions(&self) {
        for f in &self.functions {
            println!("{}", f)
        }
    }

    /// Creates a new function from a user provided name
    /// Retunrs None if the provided name contained non ASCII chars
    pub fn create_function(&mut self, name: String) -> Option<Rc<Function>> {
        // create a new id (this only works if we never delete a created label)
        let id = self.functions.len() as u64;

        match name.trim() {
            "" => {
                let f: Rc<Function> = Rc::new(id.into());
                self.functions.push(Rc::clone(&f));
                Some(f)
            },
            _ => match Function::new(id, name.trim().to_string()) {
                Ok(f) => { 
                    let f = Rc::new(f);
                    self.functions.push(Rc::clone(&f));
                    Some(f)
                },
                Err(_) => None,
            },
        }
    }

    pub fn list_tasks(&self) {
        for t in &self.tasks {
            println!("{}", t);
        }
    }

    pub fn does_task_exist(&self, id: u64) -> bool {
        self.tasks.iter().position(|t| t.id == id).is_some()
    }

    pub fn get_task(&self, id: u64) -> Option<&Rc<Task>> {
        self.tasks.iter()
            .position(|t| t.id == id)
            .and_then(|idx| self.tasks.get(idx))
    }

    fn get_dependencies(&self) -> Vec<(u64, u64)> {
        let mut dependencies = Vec::new();
        for parent in &self.tasks {
            for child_ptr in parent.children.borrow().iter() {
                if let Some(child) = child_ptr.upgrade() {
                    dependencies.push((parent.id, child.id))
                }
            }
        }

        dependencies
    }

    pub fn create_task(&mut self, is_critical: bool, function_id: Option<u64>, thread_id: u64) -> Result<Rc<Task>, &str> {
        
        // check if function for provided id exists
        let function = match function_id {
            Some(id) => {
                let id = self.functions.get(id as usize).ok_or("Provided id not in list.")?;
                Some(Rc::downgrade(id))
            },
            None => None,
        };
        
        // create new id for task, 
        let id = self.task_id_count;
        self.task_id_count += 1;

        let task = Rc::new(Task {
            id,
            thread_id,
            function,
            is_critical,
            parents: RefCell::new(Vec::new()),
            children: RefCell::new(Vec::new()),
        });

        self.tasks.push(Rc::clone(&task));
        Ok(task)
    }

    pub fn delete_task(&mut self, task_id: u64) -> Option<()> {

        self.tasks.iter()
            .position(|t| t.id == task_id)
            .and_then(|idx| { self.tasks.remove(idx); Some(()) })
    }

    pub fn add_dependency(&mut self, parent_id: u64, child_id: u64) -> Option<()> {
        let parent = self.get_task(parent_id)?;
        let child = self.get_task(child_id)?;

        {
            let mut children: RefMut<_> = parent.children.borrow_mut();
            children.push(Rc::downgrade(child));
        }

        {
            let mut parents: RefMut<_> = child.parents.borrow_mut();
            parents.push(Rc::downgrade(parent));
        }

        Some(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut task_string = String::new();
        for t in &self.tasks {
            let _ = write!(task_string, "\n\t\t{}", t);
        }

        let mut function_string = String::new();
        for f in &self.functions {
            let _ = write!(function_string, "\n\t\t{}", f);
        }

        let mut dependencies_string = String::new();
        for d in self.get_dependencies() {
            let _ = write!(dependencies_string, "\n\t\t(P: {}, C: {})", d.0, d.1);
        }

        write!(f, "Current State:\n\tPreInitialized: {}\n\tInitialized: {}\n\tTasks: {}\n\tFunctions/Labels: {}\n\tDependencies: {}", self.is_pre_init, self.is_init, task_string, function_string, dependencies_string)
    }
}

#[derive(Debug)]
pub struct Task {
    id: u64,
    thread_id: u64,
    function: Option<Weak<Function>>,
    is_critical: bool,
    parents: RefCell<Vec<Weak<Task>>>,
    children: RefCell<Vec<Weak<Task>>>,
}

impl Task {
    pub fn into_raw_parts(&self) -> (u64, u64, u64, u64) {
        let function_id = self.function
                            .as_ref()
                            .and_then(|f| f.upgrade())
                            .map_or(self.id, |f| f.id);

        (self.id, function_id, if self.is_critical { 1 } else { 0 }, self.thread_id)
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let f_label = self.function
                            .as_ref()
                            .and_then(|f| f.upgrade())
                            .map_or("None".to_string(), |f| f.name.clone());

        let string = format!("{}: label = {}, is_critical = {}, thread_id = {}", self.id, f_label, self.is_critical, self.thread_id);
        write!(f, "{}", string)
    }
}

// impl Eq for Task {}

impl From<u64> for Task {
    fn from(id: u64) -> Self {
        Task {
            id,
            thread_id: 0,
            function: Some(Rc::downgrade(&Rc::new(0.into()))),
            is_critical: false,
            parents: RefCell::new(Vec::new()),
            children: RefCell::new(Vec::new()),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub id: u64,
    pub name: String,
}

impl Function {
    pub fn new(id: u64, mut name: String) -> Result<Self, &'static str> {
        // make sure string is valid ascii
        // let mut name = name.to_owned();
        if !name.is_ascii() {
            return Err("string contains non ascii characters");
        }
        
        // add null byte for c string
        name += "\0";

        Ok(Self { id, name })
    }

    pub fn into_raw_parts(&self) -> (u64, *mut c_char) {
        (self.id, self.name.as_ptr() as *mut c_char)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.id, self.name)
    }
}

impl From<u64> for Function {
    fn from(id: u64) -> Self {
        Function {
            id,
            name: format!("default_function_{id}\0"),
        }
    }
}

// implementations so we can store them in hashsets
// impl_partial_eq!(Task; Function);
// impl_partial_ord!(Task; Function);
// impl_hash!(Task; Function);
// impl_ord!(Task; Function);

#[cfg(test)]
mod tests {
    use crate::{AppState, Function};

    #[test]
    fn function_new_is_ok() {
        let f = Function::new(0, String::from("function"));
        assert!(f.is_ok());
    }

    #[test]
    fn function_new_is_err() {
        let f = Function::new(0, String::from("功能"));
        assert!(f.is_err());
    }

    #[test]
    fn function_prepare_for_sending() {
        let f = Function::new(0, String::from("function0")).unwrap();
        let (_, name) = f.into_raw_parts();

        unsafe {
            assert_eq!(*name, 102); // f
            assert_eq!(*name.add(1), 117); // u
            assert_eq!(*name.add(2), 110); // n
            assert_eq!(*name.add(3), 99); // c
            assert_eq!(*name.add(4), 116); // t
            assert_eq!(*name.add(5), 105); // i
            assert_eq!(*name.add(6), 111); // o
            assert_eq!(*name.add(7), 110); // n
            assert_eq!(*name.add(8), 48); // 0
            assert_eq!(*name.add(9), 0); // \0
        }
    }

    #[test]
    fn app_state_create_function() {
        let mut state = AppState::new();
        assert_eq!(state.functions.len(), 0);

        state.create_function("functino".to_string());
        assert_eq!(state.functions.len(), 1);

        let result = state.functions.get(0);
        assert!(result.is_some());

        let f = result.unwrap();
        assert_eq!(f.name, "functino\0".to_string());

        state.create_function("funco".to_string());
        assert_eq!(state.functions.len(), 2);

        let result = state.functions.get(1);
        assert!(result.is_some());

        let f = result.unwrap();
        assert_eq!(f.name, "funco\0".to_string());
    }

    #[test]
    fn app_state_create_task() {
        let mut state = AppState::new();

        assert!(state.create_task(false, Some(0), 1).is_err());

        let _ = state.create_function("f1".to_string());

        assert!(state.create_task(false, Some(0), 0).is_ok());
    }
}
