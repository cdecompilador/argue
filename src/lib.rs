#![feature(str_split_once)]

/// Struct used to create a `ArgParser` with the builder pattern
pub struct ArgParserBuilder<'a> {
    arg_parser: ArgParser<'a>,
}

impl<'a> ArgParserBuilder<'a> {
    /// Not desired to be use to create the `ArgParserBuilder`
    pub fn new(app_name: &'a str) -> Self {
        ArgParserBuilder {
            arg_parser: ArgParser {
                app_name: Some(app_name),
                description: None,
                version: None,
                
                args: None,
                arg_output: None,

                received_arguments: vec![],
                input: None,
                expected_input: false,
                output: None,
                expected_output: false,

                arg_count: 0,
            }
        }
    }

    /// Set the description of the `ArgParser`
    pub fn description(mut self, h: &'a str) -> ArgParserBuilder {
        self.arg_parser.description = Some(h);
        self
    }

    /// Set the version of the `ArgParser`
    pub fn version(mut self, v: &'a str) -> ArgParserBuilder {
        self.arg_parser.version = Some(v);
        self
    }

    /// Set the arguments
    pub fn arguments(mut self, args: &'a[Argument<'a>]) -> ArgParserBuilder {
        self.arg_parser.arg_count += args.iter().count();
        self.arg_parser.args = Some(args);
        self
    }

    /// Set if there will be output arg
    /// TODO: Still not work
    pub fn output(mut self, is: bool) -> ArgParserBuilder<'a> {
        self.arg_parser.arg_count += 1;
        self.arg_parser.expected_output = is;
        self
    }

    /// Set if there will be input arg
    /// TODO: Still not work
    pub fn input(mut self, is: bool) -> ArgParserBuilder<'a> {
        self.arg_parser.arg_count += 1;
        self.arg_parser.expected_input= is;
        self
    }

    /// Build it!
    /// TODO: return a result with some defined errors
    pub fn build(mut self) -> ArgParser<'a> {
        // First of all check if --help or --version are requested
        if std::env::args().filter(|a| a == "-h" || a == "--help").next().is_some() {
            self.arg_parser.print_help();
            std::process::exit(0);
        }
        if std::env::args().filter(|a| a == "-v" || a == "--v").next().is_some() {
            self.arg_parser.print_version();
            std::process::exit(0);
        }
        // Get the received arguments from the env and save them into arg_parser
        let mut recv_args = std::env::args().into_iter();
        recv_args.next(); // Skip the program name
        let recv_args = recv_args.collect();
        let possible_args = self.parse_arguments(recv_args);

        // Check the possible arguments and push them into the received 
        // arguments if match with the expected ones, just bruteforce it!
        for possible_argument in possible_args.iter() {
            for expected_arg in self.arg_parser.args.unwrap() {
                match possible_argument {
                    PossibleArgument::Single(v) => {
                        if let ArgumentType::Single(r) = expected_arg.arg_type {
                            if expected_arg.names.iter().any(|n| n == v) {
                                let received_argument = ReceivedArgument {
                                    arg_type: ArgumentType::Single(r),
                                    key: v.clone(),
                                    value: None,
                                };
                                self.arg_parser
                                    .received_arguments.push(received_argument);
                            }
                        }
                    }
                    PossibleArgument::Paired((k, v)) => {
                        if let ArgumentType::Paired(r) = expected_arg.arg_type {
                            if expected_arg.names.iter().any(|n| n == k) {
                                let received_argument = ReceivedArgument {
                                    arg_type: ArgumentType::Paired(r),
                                    key: k.clone(),
                                    value: Some(v.clone()),
                                };
                                self.arg_parser
                                    .received_arguments.push(received_argument);
                            } 
                        }
                    }
                    PossibleArgument::Equaled((k, v)) => {
                        if let ArgumentType::Equaled(r) = expected_arg.arg_type {
                            if expected_arg.names.iter().any(|n| n == v) {
                                let received_argument = ReceivedArgument {
                                    arg_type: ArgumentType::Equaled(r),
                                    key: k.clone(),
                                    value: Some(v.clone()),
                                };
                                self.arg_parser
                                    .received_arguments.push(received_argument);
                            } 
                        }
                    }
                }
            }
        }

        // Check for the obligatory arguemnts
        if let Some(args) = self.arg_parser.args {
            let mut obligated_args = args.iter().filter(|&a| {
                match a.arg_type {
                    ArgumentType::Single(true) 
                    | ArgumentType::Paired(true) 
                    | ArgumentType::Equaled(true) => true,
                    _ => false,
                }
            });
            // Check if inside the obligated args all of them match at least 
            // with one of the received arguments
            if !obligated_args.all(|oblig_a| {
                self.arg_parser.received_arguments.iter().any(|a| {
                    oblig_a.names.iter().any(|&n| n == a.key)
                })
            }) {
                // TODO: Print exactly which was the error

                println!("Invalid arguments passed\n");
                self.arg_parser.print_help();
                std::process::exit(0);
            } 
        }

        // Return the arg_parser built
        self.arg_parser
    }

    fn parse_arguments(&self, recv_args: Vec<String>) 
        ->  Vec<PossibleArgument> {
        // Create the containers for the possible arguments
        // TODO: Maybe use crayon or futures for better efficiency but I don't
        // think this loops can be a problem
        // TODO: OMG TOO MANY STRING INSTANTIATIONS
        let mut single_arguments: Vec<String> = Vec::new();
        let mut paired_arguments: Vec<(String, String)> = Vec::new();
        let mut equaled_arguments: Vec<(String, String)> = Vec::new();

        // Single arguments loop, every pushed argument is reinstantiated
        for arg in &recv_args {
            if arg.starts_with("-") || arg.starts_with("--") {
                if arg.contains('=') {
                    continue;
                } else {
                    single_arguments.push(arg.clone());
                }
            }
        }

        // Paired arguments loop, every pushed argument is reinstantiated
        for (i, arg) in recv_args.iter().enumerate() {
            if arg.starts_with("-") || arg.starts_with("--") {
                if arg.contains('=') {
                    continue;
                } else {
                    if i != recv_args.len()-1 {
                        let second_one = recv_args[i + 1].clone();
                        if second_one.starts_with("-") 
                            || second_one.starts_with("--")
                            || second_one.contains("=") 
                        {
                            continue;
                        } else {
                            paired_arguments.push((arg.clone(), second_one));
                        }
                    }
                }
            }
        }

        // Equaled arguments loop, every pushed argument is reinstantiated
        for arg in &recv_args {
            if arg.starts_with("-") || arg.starts_with("--") {
                if arg.contains('=') {
                    arg.split_once('=').into_iter().for_each(|(x,y)| { 
                        equaled_arguments.push((x.to_owned(),y.to_owned())); 
                    });
                } else {
                    continue;
                }
            }
        }

        // Transform the Vec<String> to Vec<PossibleArgument>, reallocating
        // the strings >:c
        let single_arguments: Vec<PossibleArgument> = single_arguments
            .iter()
            .map(|arg_name| {
                PossibleArgument::Single(arg_name.clone())
        }).collect();
        let paired_arguments: Vec<PossibleArgument> = paired_arguments
            .iter()
            .map(|(key, value)| {
                PossibleArgument::Paired((key.clone(), value.clone()))
        }).collect();
        let equaled_arguments: Vec<PossibleArgument> = equaled_arguments
            .iter()
            .map(|(key, value)| {
                PossibleArgument::Equaled((key.clone(), value.clone()))
        }).collect();

        // Well I think here there is no reallocation
        let mut possible_args = Vec::with_capacity(10);
        possible_args.extend(single_arguments);
        possible_args.extend(paired_arguments);
        possible_args.extend(equaled_arguments);
        possible_args
    }
}

/// Like the ArgParserBuilder::new but more accesible
pub fn app(app_name: &str) -> ArgParserBuilder {
    ArgParserBuilder::new(app_name)
}

/// Contains all the data and arguments of the project
#[derive(Debug)]
pub struct ArgParser<'a> {
    /// Description of the proj
    description: Option<&'a str>, 
    /// Version of the proj, to be printed with -v
    version: Option<&'a str>,     
    /// The name of the proj
    app_name: Option<&'a str>,    
    /// The arguments
    args: Option<&'a[Argument<'a>]>,  
    /// The output
    arg_output: Option<Argument<'a>>, 

    arg_count: usize,
    /// The first non matchig arg
    input: Option<String>,  
    expected_input: bool,
    /// The second    ... 
    output: Option<String>, 
    expected_output: bool,

    received_arguments: Vec<ReceivedArgument>,
}

impl<'a> ArgParser<'a> {
    /// Prints the help message
    pub fn print_help(&self) -> Option<()> {
        // Push the header "names: proj description"
        let header = format!("{}: {}\n", self.app_name?, self.description?);
        // Push the arguments names and description
        let mut args = String::new();
        for arg in self.args? {
            let formatted_arg = format!("    {}\n        {}\n",
                arg.names.join(", "), arg.description);
            args.push_str(&formatted_arg);
        }
        println!("{}{}", header, args); // Print it
        Some(())
    }

    /// Print the version message
    pub fn print_version(&self) -> Option<()> {
        let header = format!("{}: {}\n", self.app_name?, self.description?);
        println!("{}Version: {}", header, self.version?);
        Some(())
    }

    /// Function that given a `key` returns the corresponding value if the 
    /// argument is found and has a value asociated, returns it
    pub fn get(&self, key: &str) -> Option<String> {
        // Check that the key is inside the possible keys, tel that uknown 
        // argument requested, and get the possible names
        let keys = self.args?.iter()
            .filter(|a| a.arg_type != ArgumentType::Single(false) || 
                        a.arg_type != ArgumentType::Single(true))
                        // Maybe a match instead??
            .find(|a| a.names.contains(&key))?.names;
        // Try get it, i don't know how to return an &'a str
        self.received_arguments.iter()
            .find(|&a| keys.contains(&a.key.as_str()))?.value.clone()
    }

    /// Retrieve the `input` argument if avaible
    pub fn get_input(&self) -> Option<String> {
        self.input.clone()
    }

    /// Retrieve the `input` argument if avaible
    pub fn get_output(&self) -> Option<String> {
        self.output.clone()
    }

    /// Check if the name argument its inside the received arguments
    pub fn is_there(&self, name: &str) -> bool {
        if let Some(args) = self.args {
            if let Some(names) = args.iter()
                .find(|a| a.names.iter().any(|&n| n == name)) {
                let names = names.names;
                if self.received_arguments.iter().any(|a| {
                    let recv_name = a.key.as_str();
                    names.contains(&recv_name) 
                }) {
                    return true;
                } 
            } 
        }
        false
    }
}

/// Defines the type of argument passed, a Single argument can be "--help", a 
/// paired one is "-j 5", and an equaled would be "--name=joshep".
/// The bool inside tells if the argument is mandatory, in case its not provided
/// It will print the usage and the missing argument 
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArgumentType {
    /// like "--help"
    Single(bool), 
    /// like "-j N"
    Paired(bool), 
    /// like "--example example0"
    Equaled(bool), 
}

/// The representation of an Argument passed to the argument parser
#[derive(Debug)]
pub struct Argument<'a> {
    /// The type of argument
    arg_type: ArgumentType, 
    /// The possible names that matches the argument
    names: &'a[&'a str],    
    /// The description of the argument
    description: &'a str,  
}

/// Received argument
#[derive(Debug, Clone)]
pub struct ReceivedArgument {
    arg_type: ArgumentType,
    key: String,
    value: Option<String>
}

impl<'a> Argument<'a> {
    /// Function that creates the argument
    pub fn new(arg_type: ArgumentType, names: &'a [&'a str], description: &'a str) -> Self {
        Argument {
            arg_type, names, description
        }
    }
}

/// The PossibleArgument is used as part of the brute force to match arguments
/// FIXME: Maybe would be better to use &String instead of owned ones
#[derive(Debug, Clone)]
enum PossibleArgument {
    Single(String),
    Paired((String, String)),
    Equaled((String, String)),
}