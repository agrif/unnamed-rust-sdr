use sdr::*;
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rate = 44100.0;
    let range = -20000.0..20000.0;
    let df = 100.0;
    let filter = filter::Biquadratic::Lr(13333.0);

    let mut sweepfilter = filter.into_filter(rate);
    let sweep = signal::freq_sweep(rate, df, true, range)
        .map(move |(f, v)| {
            (f, sweepfilter.apply(v) / v)
        }).skip(1.0 / df).iter();

    let impulse = signal::impulse::<f32>(rate).filter(filter)
        .take(1.0 / df).enumerate();

    let matches = plot::cli::setup(clap::App::new("filter"))
        .get_matches();

    plot::cli::run(&matches, (640, 640), |root| {
        root.fill(&WHITE)?;
        let subs = root.split_evenly((2, 1));

        plot::Simple::on(&subs[0])
            .title("Impulse Response")
            .xlabel("t")
            .ylabel("amplitude")
            .add_line(impulse, None)
            .draw()?;
        plot::Simple::on(&subs[1])
            .title("Frequency Response")
            .xlabel("f")
            .ylabel("dB")
            .add_complex_hue(sweep, true, Some("phase"))
            .draw()?;
        Ok(())
    })
}
