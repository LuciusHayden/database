use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CLI {
    #[arg(short, long)]
    username: String,

    #[arg(short, long)]
    password: String,

    #[arg(short, long, default_value="./data")]
    dir: String,

}


impl CLI {
    pub fn get_args() -> (String, String, String) {
        let args = CLI::parse();
        (args.username, args.password, args.dir)
    }
}
