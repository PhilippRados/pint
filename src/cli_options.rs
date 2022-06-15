use clap::{Arg, ArgMatches, Command};

fn validate_codel_size(size: String) -> Result<(), String> {
    let n = size.parse::<u32>().expect("Codel size must be number");
    if n > 0 {
        Ok(())
    } else {
        Err(String::from("Codel size must be number greater 0"))
    }
}
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
                .default_value("1")
                .long_help(
                    "Piet works by going through the pixels of an image.\n\
                    However, this makes piet images visually small when viewing them.\n\
                    Thus, piet allows interpreting images in codels which consist of larger pixels blocks.\n\
                    Setting codel-size to 2 would mean a codel is the size of 2x2 pixels.",
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
