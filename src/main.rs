use sdr::*;

struct ArgDSignal<S> {
    signal: S,
    last: Option<f64>,
}

impl<S> ArgDSignal<S> {
    fn new(signal: S) -> ArgDSignal<S> {
        ArgDSignal {
            signal,
            last: None,
        }
    }
}

impl<S> Signal for ArgDSignal<S> where S: Signal<Sample=Complex<f64>> {
    type Sample = f64;
    fn next(&mut self) -> Option<Self::Sample> {
        use std::f64::consts::PI;

        // prime ourselves if this is the first run
        if let None = self.last {
            self.last = self.signal.next().map(|t| t.arg());
        }

        if let Some(last) = self.last {
            let opt_now = self.signal.next().map(|t| t.arg());
            if let Some(now) = opt_now {
                let mut dt = now - last;
                while dt < PI {
                    dt += 2.0 * PI;
                }
                while dt > PI {
                    dt -= 2.0 * PI;
                }
                self.last = opt_now;
                Some(dt * self.rate())
            } else {
                None
            }
        } else {
            None
        }
    }
    fn rate(&self) -> f64 {
        self.signal.rate()
    }
}

fn main() -> std::io::Result<()> {
    let matches = clap::App::new("sdr fm")
        .about("listen to fm radio via rtl tcp")
        .arg(clap::Arg::with_name("FREQ")
             .required(true)
             .help("the frequency to tune to, in MHz")
             .index(1))
        .arg(clap::Arg::with_name("ADDR")
             .help("the rtltcp address to connect to")
             .required(false)
             .default_value("localhost:1234")
             .index(2))
        .get_matches();

    let sig = signal::freq(44100.0, 440.0, 0.0);

    let rate = 1800000 / 6;
    use clap::value_t_or_exit;
    let rtl = rtltcp::RtlTcp::new()
        .address(matches.value_of("ADDR").unwrap())
        .rate(rate)
        .frequency((value_t_or_exit!(matches, "FREQ", f32) * 1000000.0) as u32);

    let fm = rtl.listen()?;
    let fm = ArgDSignal::new(fm);

    //let firsize = (fm.rate() / 44100.0).round() as usize;
    //let fir = vec![1.0 / firsize as f64; firsize];
    let firb = 44100.0 / 2.0;
    let firrate = fm.rate();
    let firsize = (4.0 * firrate / (2.0 * firb)).round() as isize;
    let fir = (-firsize..firsize+1).map(|i| {
        let t = (i as f64) / firrate;
        let filt = if t == 0.0 {
            2.0 * firb
        } else {
            let arg = 2.0 * std::f64::consts::PI * firb * t;
            2.0 * firb * arg.sin() / arg
        };
        filt / firrate
    });
    let fm = fm.filter(fir.collect::<Vec<f64>>());

    let fm = fm.map(|s| s / 1000000.0);

    if false {
        let plt = plot::Plot::new();
        plt.plot(0, 0, sig.take(0.1).enumerate());
        plt.plot(1, 0, fm.take(0.1).enumerate());
        plt.show()?;
    } else {
        if false {
            let device = rodio::default_output_device().unwrap();
            let source = fm.map(|s| s as f32).iter();
            let sink = rodio::Sink::new(&device);
            sink.append(source);
            sink.sleep_until_end();
        } else {
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut wr = hound::WavWriter::create("example.wav", spec).unwrap();
            let example = fm.take(10.0).enumerate();
            for (_t, mut samp) in example {
                samp *= std::i16::MAX as f64;
                wr.write_sample(samp as i16).unwrap();
            }
            wr.finalize().unwrap();
        }
    }

    Ok(())
}
