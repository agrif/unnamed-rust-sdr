use sdr::*;

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

    let rate = 1800000 / 6;
    use clap::value_t_or_exit;
    let rtl = rtltcp::RtlTcp::new()
        .address(matches.value_of("ADDR").unwrap())
        .rate(rate)
        .gain(None)
        .rtlagc(true)
        .frequency((value_t_or_exit!(matches, "FREQ", f32) * 1000000.0) as u32);

    if false {
        let (fmraw, fm) = rtl.listen()?.tee();
        let (fmpll, fm) = fm.pll(200000.0).tee();
        let fmpll = fmpll.map(|r| r.output);
        let fm = fm.map(|r| r.frequency / 75000.0);
        let fm = fm.resample_with(resample::ConverterType::Linear, 48000.0);
        let fm = fm.filter(
            filter::Biquadratic::Lr(1.0 / (75.0 * 0.001 * 0.001))
        );

        let plt = plot::Plot::new();
        plt.plot(0, 0, fmraw.skip(1.0).take(0.005).enumerate());
        plt.plot(1, 0, fmpll.skip(1.0).take(0.005).enumerate());
        plt.plot(2, 0, fm.skip(1.0).take(1.0).enumerate());
        plt.show()?;
    } else {
        let fm = rtl.listen()?;
        let fm = fm.pll(200000.0).map(|r| r.frequency / 75000.0);
        let fm = fm.resample_with(resample::ConverterType::Linear, 48000.0);
        let fm = fm.filter(
            filter::Biquadratic::Lr(1.0 / (75.0 * 0.001 * 0.001))
        );
        if true {
            let device = rodio::default_output_device().unwrap();
            let source = fm.iter();
            let sink = rodio::Sink::new(&device);
            sink.set_volume(0.2); // inexplicably, rodio clips. so...
            sink.append(source);
            sink.sleep_until_end();
        } else {
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: fm.rate() as u32,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut wr = hound::WavWriter::create("example.wav", spec).unwrap();
            let example = fm.take(10.0).enumerate();
            for (_t, mut samp) in example {
                samp *= std::i16::MAX as f32;
                wr.write_sample(samp as i16).unwrap();
            }
            wr.finalize().unwrap();
        }
    }

    Ok(())
}
