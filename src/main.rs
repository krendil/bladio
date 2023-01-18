use clap::Parser;

mod tts;
mod json_file_source;
mod events;

#[derive(Parser, Debug)]
#[command(author="Krendil",version="0.0.1",about="Blaseball radio broadcast",long_about=None)]
struct Args {
    #[arg(short, long)]
    file_source: String,
}


fn main() {
    let args = Args::parse();

    let (tx, rx) = std::sync::mpsc::channel();

    // let _audio_thread
    let source_thread = json_file_source::new(&args.file_source, tx);
    // let state_thread = 

    source_thread.join();
}
