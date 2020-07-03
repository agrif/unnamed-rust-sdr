use sdr::*;
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fps = 60;
    let matches = plot::cli::setup_anim(
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
        .rate(1800000 / 6)
        .gain(None)
        .rtlagc(true)
        .frequency((value_t_or_exit!(matches, "FREQ", f32) * 1000000.0) as u32);

    let sig = rtl.listen()?;
    let rate = sig.rate();
    let mut sig = sig.window(1000.0 / rate)
        .decimate(fps as f32)
        .map(|w| {
            let win = w.borrow();
            let winsig = signal::from_iter(rate, win.iter().cloned());
            let ts: Vec<(f32, num::Complex<f32>)> = winsig.clone()
                .enumerate().collect();
            let fs = fft::fft(winsig);
            (ts, fs)
        });

    plot::cli::run_anim(&matches, (640, 640), fps, |root| {
        let (ts, fs) = sig.next().unwrap();

        root.fill(&WHITE)?;
        let subs = root.split_evenly((2, 1));

        plot::Simple::on(&subs[0])
            .title("Signal")
            .xlabel("t")
            .ylabel("A")
            .add_reim(ts, None)
            .draw()?;
        plot::Simple::on(&subs[1])
            .title("Frequency")
            .xlabel("f")
            .ylabel("dB")
            .add_complex(fs, true, None)
            .draw()?;

        Ok(())
    })
}
