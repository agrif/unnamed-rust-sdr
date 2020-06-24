use sdr::*;

fn freq_response<F>(rate: f32, df: f32, range: std::ops::Range<f32>,
                    usedb: bool, filter: F)
                    -> (impl Iterator<Item=(f32, f32)>,
                        impl Iterator<Item=(f32, f32)>)
where
    F: IntoFilter<Complex<f32>>,
{
    let mut filter = filter.into_filter(rate);
    let sweep = signal::freq_sweep(rate, df, true, range)
        .map(move |(f, v)| {
            (f, filter.apply(v) / v)
        }).skip(1.0 / df);
    // fixme: average windows of samplerate / df points
    let (mag, phase) = sweep.tee();
    let mag = mag.map(move |(f, v)| {
        if usedb {
            (f, 20.0 * v.norm().log10())
        } else {
            (f, v.norm())
        }
    }).iter();
    let phase = phase.map(|(f, v)| (f, v.arg())).iter();
    (mag, phase)
}

fn plot_filter<F>(rate: f32, range: std::ops::Range<f32>, df: f32, usedb: bool, filter: F)
                  -> std::io::Result<()>
where
    F: Clone + IntoFilter<f32> + IntoFilter<Complex<f32>>,
{
    let plt = plot::Plot::new();
    let (mag, phase) = freq_response(rate, df, range, usedb, filter.clone());
    let impulse = signal::impulse::<f32>(rate).filter(filter).take(1.0 / df);
    plt.plot(0, 0, impulse.enumerate());
    plt.plot(1, 0, mag);
    plt.plot(2, 0, phase);
    plt.show()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    plot_filter(44100.0, 0.0..20000.0, 100.0, true, filter::Biquadratic::Lr(13333.0))
}
