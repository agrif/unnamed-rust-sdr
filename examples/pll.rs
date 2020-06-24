use sdr::*;

fn main() -> std::io::Result<()> {
    let plt = plot::Plot::new();

    let range = 200000.0;
    let df = range / 10.0;

    let freq = signal::freq_sweep(1800000.0, df, true, -range..range);
    let mut pllf = filter::PllDesign::new(
        0.0, 0.035,
        filter::Biquadratic::LowPass(80000.0, 0.7),
        filter::Biquadratic::LowPass(20000.0, 0.7),
        filter::Biquadratic::LowPass(20000.0, 0.7),
    ).into_filter(freq.rate());

    let pll = freq.clone().map(|(f, v)| {
        (f, pllf.apply(v).unwrap_or(0.0))
    });
    plt.plot(0, 0, pll.skip(1.0 / df).iter());
    plt.plot(1, 0, freq.skip(1.0 / df).iter());
    plt.show()?;
    Ok(())
}
