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
    pub fn output(mut self, is: bool) -> ArgParserBuilder<'a> {
        self.arg_parser.arg_count += 1;
        self.arg_parser.expected_output = is;
        self
    }

    /// Set if there will be input arg
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
        let mut recv_args_parsed = Vec::with_capacity(recv_args.len());
        for expected_arg in self.arg_parser.args.unwrap_or_default().iter() {
            while let Some(received_arg) = recv_args.next() {
                // Case that the received argument is into the argument list
                if expected_arg.names.contains(&received_arg.as_str()){
                    let value = match expected_arg.arg_type {
                        ArgumentType::Single(_) => None,
                        // TODO: Discern between Paired and Equaled
                        // FIXME: Possible error if invalid input
                        _ => recv_args.next(), 
                    };
                    recv_args_parsed.push(ReceivedArgument {
                        arg_type: expected_arg.arg_type,
                        key: received_arg,
                        value,
                    });
                // Case not, check if its input, output or none of them
                } else {
                    if self.arg_parser.expected_input 
                        && self.arg_parser.input.is_none() {
                        self.arg_parser.input = Some(received_arg.clone());
                    } else if self.arg_parser.expected_output 
                        && self.arg_parser.output.is_none() {
                        self.arg_parser.output = Some(received_arg.clone());
                    } else {
                        // TODO: handle error
                    }
                }
            }
        }
        // Case no arguments but yes input/output
        if self.arg_parser.args.iter().count() == 0 {
            if self.arg_parser.expected_input {
                self.arg_parser.input = recv_args.nth(0);
            }
            if self.arg_parser.expected_output { 
                self.arg_parser.output = recv_args.nth(1);
            }
        }
        self.arg_parser.received_arguments = recv_args_parsed;
        self.arg_parser
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