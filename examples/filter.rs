use sdr::*;

fn freq_sweep(rate: f32, low: f32, high: f32, df: f32)
              -> impl Signal<Sample=(f32, Complex<f32>)>
{
    let df2 = df.powi(2);
    let dt = 1.0 / rate;
    let endt = (high - low) / df2;

    let mut freq = low;
    let mut nphase = 0.0;
    signal::from_iter(rate, std::iter::from_fn(move || {
        use std::f32::consts::PI;
        freq += df2 * dt;
        nphase += freq * dt;
        let phase = 2.0 * PI * nphase.fract();
        Some((freq, Complex::from_polar(&1.0, &phase)))
    })).take(endt)
}

fn imp_response<F>(rate: f32, df: f32, filter: F) -> impl Signal<Sample=f32>
where
    F: IntoFilter<f32>,
{
    let impulse = signal::from_iter(
        rate,
        std::iter::once(1.0).chain(std::iter::repeat(0.0))
    );
    impulse.filter(filter).take(1.0 / df)
}

fn freq_response<F>(rate: f32, low: f32, high: f32, df: f32, usedb: bool,
                    filter: F)
                    -> (impl Iterator<Item=(f32, f32)>,
                        impl Iterator<Item=(f32, f32)>)
where
    F: IntoFilter<Complex<f32>>,
{
    let mut filter = filter.into_filter(rate);
    let sweep = freq_sweep(rate, low - df, high - df, df)
        .map(move |(f, v)| {
            (f, filter.apply(v) / v)
        }).skip(2.0 / df);
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

fn plot_filter<F>(rate: f32, range: (f32, f32), df: f32, usedb: bool, filter: F)
                  -> std::io::Result<()>
where
    F: Clone + IntoFilter<f32> + IntoFilter<Complex<f32>>,
{
    let plt = plot::Plot::new();
    let impulse = imp_response(rate, df, filter.clone()).enumerate();
    let (mag, phase) = freq_response(rate, range.0, range.1, df, usedb, filter);
    plt.plot(0, 0, impulse);
    plt.plot(1, 0, mag);
    plt.plot(2, 0, phase);
    plt.show()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    plot_filter(44100.0, (0.0, 20000.0), 100.0, true, filter::Biquadratic::Lr(13333.0))
}
