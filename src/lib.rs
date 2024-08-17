use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Seek, SeekFrom, Write};
use std::path::Path;

const FILENAME: &str = "/tmp/tasks.txt";
const SEPARATOR: char = '|';

pub enum Command<'a> {
    Add(&'a str),
    List,
    Complete(usize),
    Delete(usize),
}

impl<'a> Command<'a> {
    pub fn build(args: &'a [String]) -> Result<Command<'a>, Box<dyn Error>> {
        let mut args = args.iter();

        args.next().unwrap();  // Discard program name

        let command = args
            .next()
            .map(|s| s.to_lowercase())
            .ok_or("Missing command")?;

        match command.as_str() {
            "add" => {
                let task = args.next().ok_or("Missing task")?;
                Ok(Command::Add(task))
            },
            "list" => Ok(Command::List),
            "complete" | "delete" => {
                let number = args.next()
                    .ok_or("Missing task number")?
                    .parse::<usize>()
                    .map_err(|_| "Non-integer number")?;

                if command == "complete" {
                    Ok(Command::Complete(number))
                } else {
                    Ok(Command::Delete(number))
                }
            },
            _ => Err("Unsupported command".into()),
        }
    }
}

struct Task {
    name: String,
    completed: bool,
}

impl Task {
    fn new(name: String) -> Self {
        Self {
            name,
            completed: false,
        }
    }

    fn complete(&mut self) {
        self.completed = true;
    }

    fn from_string(content: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = content.split('|').collect();

        if !parts.len() == 2 {
            return Err("Failed to parse Task".into());
        }

        let name = parts[1].to_string();
        let completed = parts[0].parse::<usize>().expect("Not a number") == 1;

        Ok(Self { name, completed })
    }

    fn to_string(&self) -> String {
        let completed = if self.completed { 1 } else { 0 };
        format!(
            "{}{}{}",
            completed,
            SEPARATOR,
            self.name,
        )
    }
}

struct TaskList {
    db_file: File,
    tasks: Vec<Task>,
}

impl TaskList {
    fn load(db_file: File) -> Result<Self, Box<dyn Error>> {
        let reader = io::BufReader::new(&db_file);

        let mut tasks: Vec<Task> = Vec::new();

        for line in reader.lines() {
            let content = line?;
            let task = Task::from_string(&content)?;

            tasks.push(task);
        }

        Ok(Self { db_file, tasks })
    }

    fn save(&mut self) -> Result<(), Box<dyn Error>> {
        self.db_file.set_len(0)?;
        self.db_file.seek(SeekFrom::Start(0))?;

        for task in self.tasks.iter() {
            writeln!(self.db_file, "{}", task.to_string())?;
        }

        self.db_file.flush()?;

        Ok(())
    }
}

pub fn run(command: Command) -> Result<(), Box<dyn Error>> {
    let db_path = Path::new(FILENAME);
    let db_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&db_path)?;

    let mut task_list = TaskList::load(db_file)?;

    match command {
        Command::Add(content) => add_task(&mut task_list, &content),
        Command::List => list_tasks(&task_list),
        Command::Complete(number) => complete_task(&mut task_list, number),
        Command::Delete(number) => delete_task(&mut task_list, number),
    }
}

fn add_task(task_list: &mut TaskList, content: &str) -> Result<(), Box<dyn Error>> {
    println!("Adding task: {}", content);

    let task = Task::new(content.to_string());

    task_list.tasks.push(task);
    task_list.save()?;

    Ok(())
}

fn list_tasks(task_list: &TaskList) -> Result<(), Box<dyn Error>> {
    println!("#{SEPARATOR}C{SEPARATOR}Task");

    for (index, task) in task_list.tasks.iter().enumerate() {
        println!("{}{}{}", index, SEPARATOR, task.to_string());
    }

    Ok(())
}

fn complete_task(task_list: &mut TaskList, number: usize) -> Result<(), Box<dyn Error>> {
    println!("Completing task: {}", number);

    task_list.tasks.get_mut(number).ok_or_else(|| "Missing task")?.complete();
    task_list.save()?;

    Ok(())
}

fn delete_task(task_list: &mut TaskList, number: usize) -> Result<(), Box<dyn Error>> {
    println!("Deleting task: {}", number);

    if task_list.tasks.len() < number {
        return Err("Missing task".into());
    }

    task_list.tasks.remove(number);
    task_list.save()?;

    Ok(())
}
