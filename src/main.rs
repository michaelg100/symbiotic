use std::fs;

use config_parse::ConfigParser;
use config_parse::Conf;

mod config_parse;

fn main() {
    let contents = fs::read_to_string("backend.yaml")
        .expect("Should have been able to read the file");

    let mystruct2: Conf = serde_yaml::from_str(&contents).unwrap();
    println!("{:#?}", mystruct2);

    let parser: ConfigParser = ConfigParser::new(mystruct2);

    parser.parse();

}
