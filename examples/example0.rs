use argue::{app, Argument, ArgumentType};

/// HOW TO:
///
/// Execute the program this way cargo r --example example0 -- -j 12
/// Also you can try --help, -h, -v, --version that are enabled by default
fn main() {
    // The arguments must be set separeted from the passing to the `arguments`
    // function
    let arguments = &[
        Argument::new(
            // The false is to say than the argument is not mandatory
            ArgumentType::Paired(true), 
            &["-j", "--jthreads"], 
            "Set the number of threads"
        ),
        Argument::new(
            ArgumentType::Single(false),
            &["-M", "-Mayus"],
            "Print the message in Upper"
        )
    ];
    
    // Ez to understand i think, pretty close to clap
    let arg_parser = app("Example0")
        .description("An example of the arg parser")
        .version("0.0.1")
        .arguments(arguments)
        .build();
    
    // Get an argument equaled, also work even if -jthreads is passed through
    // the cli
    let n = arg_parser.get("-j").unwrap();
    if arg_parser.is_there("-M") {
        println!("NUMBER OF CORES: {}", n);
    } else {
        println!("Number of cores: {}", n);
    }
}