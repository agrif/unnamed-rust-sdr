use sdr::*;

fn main() -> std::io::Result<()> {
    let matches = clap::App::new("sdr fm")
        .about("listen to fm radio via rtl tcp")
        .arg(clap::Arg::with_name("FREQ")
             .required(true)
             .help("the frequency to tune to, in MHz")
             .index(1))
        .arg(clap::Arg::with_name("address")
             .help("the rtltcp address to connect to")
             .short("a")
             .long("address")
             .value_name("ADDRESS")
             .default_value("localhost:1234")
             .takes_value(true))
        .arg(clap::Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("FILE")
             .help("Output to a WAV file, not audio card.")
             .takes_value(true))
        .arg(clap::Arg::with_name("length")
             .short("l")
             .long("length")
             .value_name("SECONDS")
             .help("How long to record, if recording.")
             .takes_value(true)
             .default_value("10"))
        .get_matches();

    let rate = 1800000;
    use clap::value_t_or_exit;
    let rtl = rtltcp::RtlTcp::new()
        .address(matches.value_of("address").unwrap())
        .rate(rate)
        .gain(None)
        .rtlagc(true)
        .frequency((value_t_or_exit!(matches, "FREQ", f32) * 1000000.0) as u32);

    let pllf = filter::PllDesign::new(
        0.0, 0.035,
        filter::Biquadratic::LowPass(80000.0, 0.7),
        filter::Biquadratic::LowPass(20000.0, 0.7),
        filter::Biquadratic::LowPass(20000.0, 0.7),
    );
    let deemph = filter::Biquadratic::Lr(1.0 / (75.0 * 0.001 * 0.001));

    let fm = rtl.listen()?;
    let fm = fm.filter(pllf).map(|f| f.unwrap_or(0.0) / 75000.0);
    let fm = fm.resample_with(resample::ConverterType::Linear, 48000.0);
    let fm = fm.filter(deemph);

    if let Some(outfile) = matches.value_of("output") {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: fm.rate() as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut wr = hound::WavWriter::create(outfile, spec).unwrap();
        let recorded = fm.take(value_t_or_exit!(matches, "length", f32)).iter();
        for mut samp in recorded {
            samp *= std::i16::MAX as f32;
            wr.write_sample(samp as i16).unwrap();
        }
        wr.finalize().unwrap();
    } else {
        let device = rodio::default_output_device().unwrap();
        let source = fm.iter();
        let sink = rodio::Sink::new(&device);
        sink.set_volume(0.2); // inexplicably, rodio clips. so...
        sink.append(source);
        sink.sleep_until_end();
    }
    Ok(())
}
