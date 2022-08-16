use clap::{Arg, ArgMatches, Command};

pub fn cli_options() -> ArgMatches {
    Command::new("piet interpreter")
        .author("Philipp Rados")
        .about("An interpreter for the piet programming language")
        .arg(
            Arg::new("file")
                .help("The image to execute. Currently only supports png.")
                .index(1)
                .required(true)
                .validator(|s| {
                    if !s.ends_with(".png") {Err(String::from("File must end with .png"))} else {Ok(())}
                }),
        )
        .arg(
            Arg::new("codel_size")
                .help("The size of a codel in pixels")
                .short('c')
                .long("codel-size")
                .long_help(
                    "Piet code takes the form of graphics made up of the recognised colours.\n
                    Individual pixels of colour are significant in the language, \n
                    so it is common for programs to be enlarged for viewing so that the details are easily visible.\n
                    In such enlarged programs, the term 'codel' is used to mean a block of colour equivalent to a single pixel of code,\n
                    to avoid confusion with the actual pixels of the enlarged graphic, of which many may make up one codel."
                )
                .takes_value(true)
                .required(false)
                .validator(|size|{
                    let n = size
                        .parse::<u32>()
                        .expect("Codel size must be number greater 0");
                    if n > 0 {
                        Ok(())
                    } else {
                        Err(String::from("Codel size must be number greater 0"))
                    }
                }
            ),
        )
    .get_matches()
}
