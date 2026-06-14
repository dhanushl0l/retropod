use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{error, path, thread};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use ringbuf::traits::{Producer, Split};
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

pub struct PlaybackSession {
    stream: cpal::Stream,
    stop_flag: Arc<AtomicBool>,
    decode_handle: Option<JoinHandle<()>>,
    progress: Arc<AtomicU8>,
}

impl PlaybackSession {
    pub fn start(path: &str, bins: Arc<Mutex<Vec<f32>>>) -> Result<Self, Box<dyn error::Error>> {
        let src = std::fs::File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path::Path::new(path).extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let mut format = symphonia::default::get_probe().probe(&hint, mss, fmt_opts, meta_opts)?;

        let track = format
            .default_track(TrackType::Audio)
            .ok_or("no audio track")?;

        let track_id = track.id;

        let codec_params = track
            .codec_params
            .as_ref()
            .ok_or("codec parameters missing")?
            .audio()
            .ok_or("not an audio codec")?
            .clone();

        let dec_opts: AudioDecoderOptions = Default::default();
        let mut decoder =
            symphonia::default::get_codecs().make_audio_decoder(&codec_params, &dec_opts)?;

        let sample_rate = codec_params.sample_rate.ok_or("unknown sample rate")?;
        let channels = codec_params
            .channels
            .as_ref()
            .map(|c| c.count())
            .ok_or("unknown channel count")? as u16;

        let ring_capacity = sample_rate as usize * channels as usize * 2; // ~2 sec
        let rb = HeapRb::<f32>::new(ring_capacity);
        let (mut producer, consumer) = rb.split();

        let stream = create_stream(sample_rate, channels, consumer)?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();
        let bins_clone = bins.clone();
        // let eq_gains_clone = eq_gains.clone();

        let total_frames = track.num_frames.unwrap_or(0);

        let progress = Arc::new(AtomicU8::new(0));
        let progress_clone = progress.clone();
        let mut frames_done: u64 = 0;
        let decode_handle = thread::spawn(move || {
            let mut samples: Vec<f32> = Vec::new();

            loop {
                if stop_flag_clone.load(Ordering::Relaxed) {
                    break;
                }

                let packet = match format.next_packet() {
                    Ok(Some(p)) => p,
                    Ok(None) => break,
                    Err(e) => {
                        eprintln!("{}", e);
                        break;
                    }
                };

                if packet.track_id != track_id {
                    continue;
                }

                let decoded = match decoder.decode(&packet) {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                decoded.copy_to_vec_interleaved(&mut samples);

                // apply_eq(&mut samples, &eq_gains_clone);
                update_bins(&samples, &bins_clone);

                if total_frames > 0 {
                    let decoded_frames = samples.len() as u64 / channels as u64;

                    frames_done += decoded_frames;

                    let pct = ((frames_done * 100) / total_frames).min(100) as u8;

                    progress_clone.store(pct, Ordering::Relaxed);
                }

                for &s in &samples {
                    loop {
                        if producer.try_push(s).is_ok() {
                            break;
                        }
                        if stop_flag_clone.load(Ordering::Relaxed) {
                            return;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            }
        });

        Ok(Self {
            stream,
            stop_flag,
            decode_handle: Some(decode_handle),
            progress,
        })
    }

    pub fn pause(&self) {
        let _ = self.stream.pause();
    }

    pub fn resume(&self) {
        let _ = self.stream.play();
    }
    pub fn progress(&self) -> u8 {
        self.progress.load(Ordering::Relaxed)
    }

    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        let _ = self.stream.pause();
        if let Some(h) = self.decode_handle.take() {
            let _ = h.join();
        }
    }
}

fn create_stream(
    sample_rate: u32,
    channels: u16,
    mut consumer: impl ringbuf::traits::Consumer<Item = f32> + Send + 'static,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device available")?;

    let stream_config = cpal::StreamConfig {
        channels,
        sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    let stream = device.build_output_stream(
        stream_config,
        move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                *sample = consumer.try_pop().unwrap_or(0.0);
            }
        },
        move |err| eprintln!("stream error: {err}"),
        None,
    )?;

    stream.play()?;

    Ok(stream)
}

fn apply_eq(_samples: &mut [f32], _eq_gains: &Arc<Mutex<Vec<f32>>>) {
    // TODO
}

fn update_bins(_samples: &[f32], _bins: &Arc<Mutex<Vec<f32>>>) {
    // TODO
}
