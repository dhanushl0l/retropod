use chrono::Local;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use std::process::Command;

use retropod::{
    APP_TITLE,
    display::{header::render_header, init_display, render::render_display},
    web::home::render_home,
};

#[derive(Clone, Debug)]
struct Message {
    content: String,
}

fn play(file: &str) {
    Command::new("pw-play").arg(file).status().unwrap();
}

fn main() {
    let (broker_tx, broker_rx) = mpsc::channel::<Message>();
    let shared_receiver = Arc::new(Mutex::new(broker_rx));

    let tx_to_tcp = broker_tx.clone();

    let tcp_handler = thread::spawn(move || {
        start_tcp(tx_to_tcp);
    });
    let player_resiver: Arc<Mutex<mpsc::Receiver<Message>>> = Arc::clone(&shared_receiver);
    let player_handler = thread::spawn(|| {
        start_media(player_resiver);
    });

    let display = init_display::setup_display();

    let display_handler = thread::spawn(|| {
        render_display(display);
    });
    loop {}
}

fn start_media(rx: Arc<Mutex<mpsc::Receiver<Message>>>) {
    // Load songs from uploads/
    let mut songs: Vec<String> = vec![];

    if let Ok(entries) = fs::read_dir("uploads") {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name() {
                    songs.push(name.to_string_lossy().to_string());
                }
            }
        }
    }

    println!("Loaded {} songs", songs.len());

    loop {
        // Receive all pending songs without blocking playback
        {
            let receiver = rx.lock().unwrap();

            while let Ok(message) = receiver.try_recv() {
                println!("Added new song: {}", message.content);

                songs.push(message.content);
            }
        }

        // If no songs exist, sleep and continue
        if songs.is_empty() {
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        // Random song
        let index = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as usize)
            % songs.len();

        let song = songs[index].clone();

        let file_path = format!("uploads/{}", song);

        // Play in separate thread
        let play_thread = thread::spawn(move || {
            play(&file_path);
        });

        // Wait until song finishes
        match play_thread.join() {
            Ok(_) => {
                println!("Song finished");
            }

            Err(_) => {
                println!("Playback thread crashed");
            }
        }
    }
}

fn start_tcp(mut tx_resiver: mpsc::Sender<Message>) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server is running at http://0.0.0.0:8080");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, &mut tx_resiver);
    }
}

fn handle_connection(mut stream: TcpStream, tx_resiver: &mut mpsc::Sender<Message>) {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut file_name = String::from("unknown");

    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line).unwrap();
    let request_line = request_line.trim();

    let mut content_length = 0;

    loop {
        let mut header_line = String::new();
        buf_reader.read_line(&mut header_line).unwrap();

        if header_line == "\r\n" {
            break;
        }

        if header_line.to_lowercase().starts_with("filename:") {
            let parts: Vec<&str> = header_line.split(':').collect();
            if parts.len() == 2 {
                content_length = parts[1].trim().parse::<usize>().unwrap_or(0);
            }
            file_name = parts[1].trim().to_string();
        }
        if header_line.to_lowercase().starts_with("content-length:") {
            let parts: Vec<&str> = header_line.split(':').collect();
            if parts.len() == 2 {
                content_length = parts[1].trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    let (status_line, contents) = match request_line {
        "GET / HTTP/1.1" => render_home(),
        "OPTIONS /update_media HTTP/1.1" => ("HTTP/1.1 204 No Content", String::new()),
        "POST /update_media HTTP/1.1" => {
            get_media(&mut buf_reader, content_length, file_name, tx_resiver)
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "Oops, 404 Not Found.".to_string()),
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: text/html\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
}

fn detect_audio_type(data: &[u8]) -> Option<&'static str> {
    // MP3: ID3 header or MPEG frame sync
    if data.starts_with(b"ID3") || (data.len() > 1 && data[0] == 0xFF && (data[1] & 0xE0) == 0xE0) {
        return Some("mp3");
    }

    // FLAC
    if data.starts_with(b"fLaC") {
        return Some("flac");
    }

    // WAV
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
        return Some("wav");
    }

    None
}

fn get_media(
    buf_reader: &mut BufReader<&mut TcpStream>,
    content_length: usize,
    file_name: String,
    tx_resiver: &mut mpsc::Sender<Message>,
) -> (&'static str, String) {
    if content_length == 0 {
        return ("HTTP/1.1 400 BAD REQUEST", "No data sent!".to_string());
    }

    // Optional: limit upload size (example: 20 MB)
    const MAX_SIZE: usize = 100 * 1024 * 1024;

    if content_length > MAX_SIZE {
        return (
            "HTTP/1.1 413 PAYLOAD TOO LARGE",
            "File too large!".to_string(),
        );
    }

    // Read body safely
    let mut body = vec![0u8; content_length];

    if let Err(err) = buf_reader.read_exact(&mut body) {
        return (
            "HTTP/1.1 400 BAD REQUEST",
            format!("Failed to read upload: {}", err),
        );
    }

    // Detect file type
    let extension = match detect_audio_type(&body) {
        Some(ext) => ext,
        None => {
            return (
                "HTTP/1.1 415 UNSUPPORTED MEDIA TYPE",
                "Only MP3, FLAC, and WAV files are allowed.".to_string(),
            );
        }
    };

    // Save file
    let filename = format!("{}.{}", file_name, extension);
    let file_path = format!("uploads/{}", filename);

    // Ensure uploads dir exists
    if let Err(err) = std::fs::create_dir_all("uploads") {
        return (
            "HTTP/1.1 500 INTERNAL SERVER ERROR",
            format!("Failed to create upload directory: {}", err),
        );
    }

    match File::create(&file_path) {
        Ok(mut file) => {
            if let Err(err) = file.write_all(&body) {
                return (
                    "HTTP/1.1 500 INTERNAL SERVER ERROR",
                    format!("Failed to save file: {}", err),
                );
            }
        }
        Err(err) => {
            return (
                "HTTP/1.1 500 INTERNAL SERVER ERROR",
                format!("Could not create file: {}", err),
            );
        }
    }

    println!("Success! Saved {} bytes as {}", content_length, file_path);
    tx_resiver.send(Message { content: filename }).unwrap();

    (
        "HTTP/1.1 200 OK",
        format!("Uploaded {} file successfully!", extension),
    )
}
