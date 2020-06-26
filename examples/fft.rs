use sdr::*;

fn main() -> std::io::Result<()> {
    let rtl = rtltcp::RtlTcp::new()
        .address("localhost:1234")
        .rate(1800000)
        .gain(None)
        .rtlagc(true)
        .frequency((97.9 * 1000000.0) as u32);

    let pllf = filter::PllDesign::new(
        0.0, 0.035,
        filter::Biquadratic::LowPass(80000.0, 0.7),
        filter::Identity,
        filter::Biquadratic::LowPass(20000.0, 0.7),
    );

    let pilotf = 19000.0;
    let pllpilot = filter::PllDesign::new(
        pilotf, 0.0002,
        filter::Biquadratic::LowPass(200.0, 0.7),
        filter::Biquadratic::LowPass(20.0, 0.7),
        filter::Biquadratic::LowPass(20.0, 0.7),
    );

    let deemph = filter::Biquadratic::Lr(1.0 / (75.0 * 0.001 * 0.001));

    let fm = rtl.listen()?;
    let fm = fm.filter(pllf).map(|f| f.unwrap_or(0.0) / 75000.0);
    let fm = fm.resample_with(resample::ConverterType::Linear, 48000.0 * 3.0);

    let mut monod = deemph.clone().into_filter(fm.rate());
    let mut diffd = deemph.clone().into_filter(fm.rate());
    let mut pilot = pllpilot.into_filter(fm.rate());
    let fm = fm.map(|f| {
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

    let fm = fm.skip(2.0).take(2.0);
    let (fmmono, fm) = fm.tee();
    let fmmono = fmmono.map(|v| v.1);
    let (fmpilot, fm) = fm.tee();
    let fmpilot = fmpilot.map(|v| v.2);
    let (fmdiff, fm) = fm.tee();
    let fmdiff = fmdiff.map(|v| v.3);
    let fm = fm.map(|v| v.0);

    let plt = plot::Plot::new();
    plt.plot(0, 0, fft::rfft(fm).into_iter().map(|(f, v)| (f, v.norm())));
    plt.plot(1, 0, fft::rfft(fmmono).into_iter().map(|(f, v)| (f, v.norm())));
    plt.plot(2, 0, fmpilot.enumerate());
    plt.plot(3, 0, fft::rfft(fmdiff).into_iter().map(|(f, v)| (f, v.norm())));
    plt.show()?;
    Ok(())
}
