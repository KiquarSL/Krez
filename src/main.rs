use clap::Parser;
use krez::compiler::KrezCompiler;
use krez::report::std::Verbose;

#[derive(Debug, clap::ValueEnum, Clone)]
#[clap(rename_all = "lowercase")]
enum BackendType {
    Qbe,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, num_args = 1..)]
    pub src: Vec<String>,
    #[arg(short, long, default_value = "qbe")]
    pub backend: BackendType,
    #[arg(short, long)]
    pub target: String,
    #[arg(short, long, default_value = "normal")]
    pub verbose: Verbose,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut krezc = KrezCompiler::default(args.target, args.verbose);
    krezc.compile(args.src)
}
