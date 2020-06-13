use std::cell::RefCell;
use std::collections::HashMap;
use rustplotlib::{Axes2D, Line2D, Figure, Backend};

pub struct Plot<'a> {
    // use a cell here so we can borrow axes mutably while allocator is
    // borrowed immutably
    axes: RefCell<HashMap<(u32, u32), Axes2D<'a>>>,
    allocator: bumpalo::Bump,
}

pub trait Plottable: Sized {
    fn plot<'a>(plot: &'a Plot<'a>, data: &[(f64, Self)]) -> Axes2D<'a>;
}

impl<'a> Plot<'a> {
    pub fn new() -> Self {
        Plot {
            axes: RefCell::new(HashMap::new()),
            allocator: bumpalo::Bump::new(),
         }
    }

    pub fn add_data<T, F, B>(&'a self, data: &[T], mut f: F) -> &'a [B]
    where
        F: FnMut(&T) -> B,
    {
        self.allocator.alloc_slice_fill_with(data.len(), move |i| f(&data[i]))
    }

    pub fn plot_axes(&'a self, row: u32, col: u32, ax: Axes2D<'a>) {
        self.axes.borrow_mut().insert((row, col), ax);
    }

    pub fn plot<I, A>(&'a self, row: u32, col: u32, iter: I)
    where
        I: Iterator<Item=(f64, A)>,
        A: Plottable,
    {
        let data: Vec<(f64, A)> = iter.collect();
        let ax = Plottable::plot(&self, &data);
        self.plot_axes(row, col, ax);
    }

    pub fn figure<F, B>(&'a self, f: F) -> B where F: FnOnce(Figure<'a>) -> B {
        let mut axes = Vec::new();
        let mut maxrow = 1;
        let mut maxcol = 1;
        for (row, col) in self.axes.borrow().keys() {
            if row + 1 > maxrow {
                maxrow = row + 1;
            }
            if col + 1 > maxcol {
                maxcol = col + 1;
            }
        }
        for row in 0..maxrow {
            for col in 0..maxcol {
                axes.push(self.axes.borrow_mut().remove(&(row, col)));
            }
        }

        f(Figure::new().subplots(maxrow, maxcol, axes))
    }

    pub fn show(&'a self) -> std::io::Result<()> {
        self.figure(|fig| {
            let mut mpl = rustplotlib::backend::Matplotlib::new()?;
            mpl.set_style("ggplot")?;
            fig.apply(&mut mpl)?;
            mpl.show()?;
            mpl.wait()?;
            Ok(())
        })
    }
}

impl Plottable for f64 {
    fn plot<'a>(plot: &'a Plot<'a>, data: &[(f64, Self)]) -> Axes2D<'a> {
        let x = plot.add_data(data, |t| t.0);
        let y = plot.add_data(data, |t| t.1);
        Axes2D::new()
            .xlabel("t")
            .add(Line2D::new("value").data(x, y))
    }
}

impl Plottable for num::Complex<f64> {
    fn plot<'a>(plot: &'a Plot<'a>, data: &[(f64, Self)]) -> Axes2D<'a> {
        let x = plot.add_data(data, |t| t.0);
        let re = plot.add_data(data, |t| t.1.re);
        let im = plot.add_data(data, |t| t.1.im);
        Axes2D::new()
            .xlabel("t")
            .add(Line2D::new("Re").data(x, re))
            .add(Line2D::new("Im").data(x, im))
            .legend("lower right")
    }
}
