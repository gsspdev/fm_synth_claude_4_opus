use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Add these to your Cargo.toml:
// [dependencies]
// cpal = "0.15"
// anyhow = "1.0"

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// FM Synthesizer parameters
#[derive(Clone)]
struct FMParams {
    carrier_freq: f32,      // Carrier frequency in Hz
    modulator_freq: f32,    // Modulator frequency in Hz
    modulation_index: f32,  // Modulation depth
    amplitude: f32,         // Output amplitude (0.0 - 1.0)
}

impl Default for FMParams {
    fn default() -> Self {
        Self {
            carrier_freq: 440.0,
            modulator_freq: 220.0,
            modulation_index: 2.0,
            amplitude: 0.3,
        }
    }
}

/// FM Synthesizer oscillator
struct FMOscillator {
    sample_rate: f32,
    carrier_phase: f32,
    modulator_phase: f32,
    params: FMParams,
}

impl FMOscillator {
    fn new(sample_rate: f32, params: FMParams) -> Self {
        Self {
            sample_rate,
            carrier_phase: 0.0,
            modulator_phase: 0.0,
            params,
        }
    }

    /// Generate next sample using FM synthesis
    fn next_sample(&mut self) -> f32 {
        // Calculate modulator output
        let modulator = (2.0 * PI * self.modulator_phase).sin();
        
        // Apply modulation to carrier frequency
        let modulated_freq = self.params.carrier_freq * 
            (1.0 + self.params.modulation_index * modulator);
        
        // Generate carrier with modulated frequency
        let carrier = (2.0 * PI * self.carrier_phase).sin();
        
        // Update phases
        self.carrier_phase += modulated_freq / self.sample_rate;
        self.modulator_phase += self.params.modulator_freq / self.sample_rate;
        
        // Wrap phases to prevent overflow
        if self.carrier_phase >= 1.0 {
            self.carrier_phase -= 1.0;
        }
        if self.modulator_phase >= 1.0 {
            self.modulator_phase -= 1.0;
        }
        
        // Return amplitude-scaled output
        carrier * self.params.amplitude
    }

    fn set_params(&mut self, params: FMParams) {
        self.params = params;
    }
}

/// ADSR Envelope generator
struct Envelope {
    attack: f32,   // Attack time in seconds
    decay: f32,    // Decay time in seconds
    sustain: f32,  // Sustain level (0.0 - 1.0)
    release: f32,  // Release time in seconds
    
    sample_rate: f32,
    state: EnvelopeState,
    level: f32,
    time: f32,
}

#[derive(PartialEq)]
enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Envelope {
    fn new(sample_rate: f32) -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.5,
            sample_rate,
            state: EnvelopeState::Idle,
            level: 0.0,
            time: 0.0,
        }
    }

    fn trigger(&mut self) {
        self.state = EnvelopeState::Attack;
        self.time = 0.0;
    }

    fn release(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
            self.time = 0.0;
        }
    }

    fn process(&mut self) -> f32 {
        let dt = 1.0 / self.sample_rate;
        
        match self.state {
            EnvelopeState::Idle => {
                self.level = 0.0;
            }
            EnvelopeState::Attack => {
                self.level = self.time / self.attack;
                if self.time >= self.attack {
                    self.state = EnvelopeState::Decay;
                    self.time = 0.0;
                }
            }
            EnvelopeState::Decay => {
                self.level = 1.0 - ((1.0 - self.sustain) * (self.time / self.decay));
                if self.time >= self.decay {
                    self.state = EnvelopeState::Sustain;
                    self.time = 0.0;
                }
            }
            EnvelopeState::Sustain => {
                self.level = self.sustain;
            }
            EnvelopeState::Release => {
                self.level = self.sustain * (1.0 - (self.time / self.release));
                if self.time >= self.release {
                    self.state = EnvelopeState::Idle;
                    self.level = 0.0;
                }
            }
        }
        
        self.time += dt;
        self.level
    }
}

/// FM Synthesizer with envelope
struct FMSynth {
    oscillator: FMOscillator,
    envelope: Envelope,
}

impl FMSynth {
    fn new(sample_rate: f32, params: FMParams) -> Self {
        Self {
            oscillator: FMOscillator::new(sample_rate, params),
            envelope: Envelope::new(sample_rate),
        }
    }

    fn next_sample(&mut self) -> f32 {
        let osc_out = self.oscillator.next_sample();
        let env_out = self.envelope.process();
        osc_out * env_out
    }

    fn note_on(&mut self) {
        self.envelope.trigger();
    }

    fn note_off(&mut self) {
        self.envelope.release();
    }

    fn set_params(&mut self, params: FMParams) {
        self.oscillator.set_params(params);
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize audio
    let host = cpal::default_host();
    let device = host.default_output_device()
        .expect("No output device available");
    
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    
    // Create synth with default parameters
    let params = FMParams {
        carrier_freq: 440.0,      // A4
        modulator_freq: 880.0,    // A5
        modulation_index: 5.0,    // High modulation for bell-like sound
        amplitude: 0.3,
    };
    
    let synth = Arc::new(Mutex::new(FMSynth::new(sample_rate, params)));
    
    // Clone for audio callback
    let synth_clone = Arc::clone(&synth);
    
    // Build output stream
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut synth = synth_clone.lock().unwrap();
                for sample in data.iter_mut() {
                    *sample = synth.next_sample();
                }
            },
            |err| eprintln!("Error in audio stream: {}", err),
            None,
        )?,
        _ => panic!("Unsupported sample format"),
    };
    
    stream.play()?;
    
    println!("FM Synthesizer Demo");
    println!("==================");
    println!("Playing a sequence of FM tones...\n");
    
    // Play a simple melody
    let notes = vec![
        (440.0, 880.0, 2.0),   // A4 with 2:1 ratio
        (523.25, 1046.5, 3.0), // C5 with 2:1 ratio
        (659.25, 659.25, 5.0), // E5 with 1:1 ratio (bell-like)
        (440.0, 220.0, 8.0),   // A4 with 1:2 ratio (sub-harmonic)
    ];
    
    for (carrier, modulator, mod_index) in notes {
        println!("Playing: Carrier={:.1}Hz, Modulator={:.1}Hz, Index={:.1}", 
                 carrier, modulator, mod_index);
        
        // Update synth parameters
        {
            let mut synth = synth.lock().unwrap();
            synth.set_params(FMParams {
                carrier_freq: carrier,
                modulator_freq: modulator,
                modulation_index: mod_index,
                amplitude: 0.3,
            });
            synth.note_on();
        }
        
        // Play for 1 second
        std::thread::sleep(Duration::from_millis(800));
        
        // Note off
        {
            let mut synth = synth.lock().unwrap();
            synth.note_off();
        }
        
        // Wait for release
        std::thread::sleep(Duration::from_millis(700));
    }
    
    println!("\nDone!");
    Ok(())
}

// Example usage for creating different timbres:
#[allow(dead_code)]
fn example_presets() -> Vec<(&'static str, FMParams)> {
    vec![
        ("Bell", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 440.0,
            modulation_index: 7.0,
            amplitude: 0.3,
        }),
        ("Bass", FMParams {
            carrier_freq: 110.0,
            modulator_freq: 110.0,
            modulation_index: 1.5,
            amplitude: 0.5,
        }),
        ("Electric Piano", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 880.0,
            modulation_index: 3.0,
            amplitude: 0.4,
        }),
        ("Brass", FMParams {
            carrier_freq: 440.0,
            modulator_freq: 440.0,
            modulation_index: 2.5,
            amplitude: 0.4,
        }),
    ]
}
