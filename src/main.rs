use rodio::{Decoder, OutputStream, Sink};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::thread::sleep;
use std::time::{self, Duration, Instant};
use udev::MonitorBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let mut last_alert = Instant::now();
    let mut last_alert_type = "none".to_owned();

    // Set up the udev monitor
    let monitor = MonitorBuilder::new()
        .unwrap()
        .match_subsystem("usb")
        .unwrap();

    // Start monitoring
    let socket = monitor.listen().unwrap();

    let mut iter = socket.iter();

    println!("udev monitor socket opened");

    loop {
        match iter.next() {
            Some(event) => {
                println!("new event");
                let action = event.action().ok_or("fuck")?.to_str().unwrap();
                let this_alert = Instant::now();

                // Only care about these 2 events and if the event type is same,
                // use a timeout as there are multiple events with the same action
                // on device connect and disconnect
                if (action == "add" || action == "remove")
                    && (this_alert.duration_since(last_alert) > Duration::from_millis(1200)
                        && last_alert_type == action.to_owned()
                        || last_alert_type != action.to_owned())
                {
                    last_alert = this_alert;
                    last_alert_type = action.to_owned();

                    if action == "add" {
                        play_sound("connect.wav")
                    } else if action == "remove" {
                        play_sound("disconnect.wav")
                    }
                }
            }
            None => {
                println!("no event");
            }
        }
    }
}

fn play_sound(sound_name: &str) {
    // Get the default sink
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Open the WAV file
    let mut full_path = env::current_dir().unwrap().as_os_str().to_owned();
    full_path.push(format!("/{}", sound_name));
    let file = BufReader::new(File::open(full_path).unwrap());
    let source = Decoder::new(file).unwrap();

    // Play the sound directly on the device
    sink.append(source);
    sink.sleep_until_end();
}
