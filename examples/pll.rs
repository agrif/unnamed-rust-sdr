use sdr::*;
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let range = 200000.0;
    let df = range / 10.0;

    let freq = signal::freq_sweep(1800000.0, df, true, -range..range);
    let mut pllf = filter::PllDesign::new(
        0.0, 0.035,
        filter::BiquadD::LowPass(80000.0, 0.7),
        filter::BiquadD::LowPass(20000.0, 0.7),
        filter::BiquadD::LowPass(20000.0, 0.7),
    ).design(freq.rate());

    let pll = freq.clone().map(|(f, v)| {
        (f, pllf.apply(v).unwrap_or(0.0))
    });

    let matches = plot::cli::setup(clap::App::new("pll"))
        .get_matches();

    plot::cli::run(&matches, (640, 640), |root| {
        root.fill(&WHITE)?;
        let subs = root.split_evenly((2, 1));

        plot::Simple::on(&subs[0])
            .title("PLL Output")
            .xlabel("f")
            .add_line(pll.skip(1.0 / df).iter(), None)
            .draw()?;
        plot::Simple::on(&subs[1])
            .title("Input")
            .xlabel("f")
            .add_reim(freq.skip(1.0 / df).iter(), None)
            .draw()?;

        Ok(())
    })
}
