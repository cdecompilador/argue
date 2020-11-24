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

    /// Set the arg_output,
    pub fn arg_output(mut self, arg: Argument<'a>) -> ArgParserBuilder {
        self.arg_parser.arg_count += 1;
        self.arg_parser.arg_output = Some(arg);
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
        let mut recv_args = std::env::args().into_iter().peekable();
        let mut recv_args_parsed = Vec::with_capacity(recv_args.len());
        // TODO: Needs a lot of improvement
        for arg in self.arg_parser.args.unwrap().iter() {
            if let Some(a) = recv_args
                .find(|a_str| { arg.names.contains(&a_str.as_str()) }) 
            {
                let value = match arg.arg_type {
                    ArgumentType::Single(_) => None,
                    // TODO: Discern between Paired and Equaled
                    // FIXME: Possible error if invalid input
                    _ => recv_args.next(), 
                };
                recv_args_parsed.push(ReceivedArgument {
                    arg_type: arg.arg_type,
                    key: a,
                    value,
                });
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
    description: Option<&'a str>, // Description of the proj
    version: Option<&'a str>,     // Version of the proj, to be printed with -v
    app_name: Option<&'a str>,    // The name of the proj

    args: Option<&'a[Argument<'a>]>,  // The arguments
    arg_output: Option<Argument<'a>>, // The output

    arg_count: usize,

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
}

/// Defines the type of argument passed, a Single argument can be "--help", a 
/// paired one is "-j 5", and an equaled would be "--name=joshep".
/// The bool inside tells if the argument is mandatory, in case its not provided
/// It will print the usage and the missing argument 
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArgumentType {
    Single(bool), // like "--help"
    Paired(bool), // like "-j N"
    Equaled(bool), // like "--example example0"
}

/// The representation of an Argument passed to the argument parser
#[derive(Debug)]
pub struct Argument<'a> {
    arg_type: ArgumentType, // The type of argument
    names: &'a[&'a str],    // The possible names that matches the argument
    description: &'a str,   // The description of the argument
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