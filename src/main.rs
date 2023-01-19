use announce_channel::AnnounceChannel;
use clap::Parser;
use vorbis_output::output_to_vorbis;

mod tts;
mod json_file_source;
mod events;
mod game_state;
mod announce_channel;
mod types;
mod vorbis_output;

#[derive(Parser, Debug)]
#[command(author="Krendil",version="0.0.1",about="Blaseball radio broadcast",long_about=None)]
struct Args {
    #[arg(short, long)]
    file_source: String,
}


fn main() {
    let args = Args::parse();

    let (game_tx, game_rx) = std::sync::mpsc::channel();
    let (announce_tx, announce_rx) = std::sync::mpsc::channel();

    // let _audio_thread
    let source_thread = json_file_source::new(&args.file_source, game_tx);
    let game_thread = game_state::spawn_game_thread(game_rx, announce_tx); 
    let mut announcer = AnnounceChannel::new(announce_rx);
    
    let mut outstream = std::io::stdout();
    let _encoder = output_to_vorbis(move |buf| {
        announcer.next(buf)
    }, &mut outstream).unwrap();

    source_thread.join().unwrap();
    game_thread.join().unwrap();
}
