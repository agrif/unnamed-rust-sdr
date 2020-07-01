use super::dynbackend::DynDrawingBackend;

use piston_window::{EventLoop, PistonWindow, WindowSettings};
use plotters::prelude::*;
use plotters::coord::Shift;
use std::error::Error;

pub fn setup<'a, 'b>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.arg(clap::Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("FILE")
            .help("Write plot to a file.")
            .takes_value(true))
}

pub fn run<F>(matches: &clap::ArgMatches, size: (u32, u32), body: F)
              -> Result<(), Box<dyn Error>>
where
    F: FnOnce(DrawingArea<&mut dyn DynDrawingBackend, Shift>)
              -> Result<(), Box<dyn Error>>,
{
    if let Some(output) = matches.value_of("output") {
        let mut b = BitMapBackend::new(output, size);
        body((&mut b as &mut dyn DynDrawingBackend).into_drawing_area())
    } else {
        let mut window: PistonWindow = WindowSettings::new(
            "plot", [size.0, size.1]
        ).samples(4).build()?;
        window.set_max_fps(1);
        let mut optbody = Some(body);
        while let Some(_) = draw_piston_window(&mut window, |mut b| {
            if let Some(body) = optbody.take() {
                body((&mut b as &mut dyn DynDrawingBackend).into_drawing_area())
            } else {
                Ok(())
            }
        }) {}
        Ok(())
    }
}

pub fn setup_anim<'a, 'b>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app
}

pub fn run_anim<F>(_matches: &clap::ArgMatches, size: (u32, u32), fps: u64,
                   mut body: F) -> Result<(), Box<dyn Error>>
where
    F: FnMut(DrawingArea<&mut dyn DynDrawingBackend, Shift>)
             -> Result<(), Box<dyn Error>>,
{
    let mut window: PistonWindow = WindowSettings::new(
        "plot", [size.0, size.1]
    ).samples(4).build()?;
    window.set_max_fps(fps);
    while let Some(_) = draw_piston_window(&mut window, |mut b| {
        body((&mut b as &mut dyn DynDrawingBackend).into_drawing_area())
    }) {}
    Ok(())
}
