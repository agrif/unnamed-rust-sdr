use sdr::*;
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = plot::cli::setup(
        clap::App::new("fft")
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
    ).get_matches();

    use clap::value_t_or_exit;
    let rtl = rtltcp::RtlTcp::new()
        .address(matches.value_of("address").unwrap())
        .rate(1800000)
        .gain(None)
        .rtlagc(true)
        .frequency((value_t_or_exit!(matches, "FREQ", f32) * 1000000.0) as u32);

    let pllf = filter::PllDesign::new(
        0.0, 0.035,
        filter::BiquadD::LowPass(80000.0, 0.7),
        filter::Identity,
        filter::BiquadD::LowPass(20000.0, 0.7),
    );

    let pilotf = 19000.0;
    let pllpilot = filter::PllDesign::new(
        pilotf, 0.0002,
        filter::BiquadD::LowPass(200.0, 0.7),
        filter::BiquadD::LowPass(20.0, 0.7),
        filter::BiquadD::LowPass(20.0, 0.7),
    );

    let deemph = filter::BiquadD::Lr(1.0 / (75.0 * 0.001 * 0.001));

    let fm = rtl.listen()?;
    let fm = fm.filter(pllf).map(|f| f.unwrap_or(0.0) / 75000.0);
    let fm = fm.resample_with(resample::ConverterType::Linear, 48000.0 * 3.0);

    let mut monod = deemph.clone().design(fm.rate());
    let mut diffd = deemph.clone().design(fm.rate());
    let mut pilot = pllpilot.design(fm.rate());
    let fm = fm.map(move |f| {
        let mono = monod.apply(f);
        let pilottune = pilot.apply(num::Complex::new(f, 0.0));
        let diff = if let Some(_) = pilottune {
            let diffc = f / pilot.value.powi(2);
            diffd.apply(diffc.re)
        } else {
            0.0
        };
        (f, mono, pilottune.unwrap_or(0.0), diff)
    });

    let fm = fm.skip(2.0).take(0.1).block(0.1);
    let fmmono = fm.clone().map(|v| v.1);
    let fmpilot = fm.clone().map(|v| v.2);
    let fmdiff = fm.clone().map(|v| v.3);
    let fm = fm.map(|v| v.0);

    plot::cli::run(&matches, (640, 200 * 4), |root| {
        root.fill(&WHITE)?;
        let subs = root.split_evenly((4, 1));

        plot::Simple::on(&subs[0])
            .title("Raw Demodulated FM")
            .xlabel("f")
            .ylabel("dB")
            .add_complex(fft::rfft(fm), true, None)
            .draw()?;
        plot::Simple::on(&subs[1])
            .title("L + R")
            .xlabel("f")
            .ylabel("dB")
            .add_complex(fft::rfft(fmmono), true, None)
            .draw()?;
        plot::Simple::on(&subs[2])
            .title("Pilot Tune Deviation")
            .xlabel("t")
            .ylabel("df")
            .add_line(fmpilot.enumerate(), None)
            .draw()?;
        plot::Simple::on(&subs[3])
            .title("L - R")
            .xlabel("f")
            .ylabel("dB")
            .add_complex(fft::rfft(fmdiff), true, None)
            .draw()?;

        Ok(())
     })
}
