use plotters::prelude::*;
use plotters::chart::SeriesAnno;
use plotters::coord::{AsRangedCoord, Shift};
use plotters::element::{DynElement, PointCollection};
use std::ops::{Deref, DerefMut, Range};

pub struct AutoRange<'a, 'b, DB: DrawingBackend, X: Clone, Y: Clone> {
    chart: ChartBuilder<'a, 'b, DB>,
    xrange: Range<X>,
    yrange: Range<Y>,
    series: Vec<(Vec<DynElement<'static, DB, (X, Y)>>,
                 Box<dyn FnOnce(&mut SeriesAnno<'a, DB>)>)>,
}

impl<'a, 'b, DB, X, Y> Deref for AutoRange<'a, 'b, DB, X, Y>
where
    DB: DrawingBackend,
    X: Clone,
    Y: Clone,
{
    type Target = ChartBuilder<'a, 'b, DB>;
    fn deref(&self) -> &Self::Target {
        &self.chart
    }
}

impl<'a, 'b, DB, X, Y> DerefMut for AutoRange<'a, 'b, DB, X, Y>
where
    DB: DrawingBackend,
    X: Clone,
    Y: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chart
    }
}

impl<'a, 'b, DB, X, Y> AutoRange<'a, 'b, DB, X, Y>
where
    DB: DrawingBackend,
    X: num::Zero + Clone + PartialOrd,
    Y: num::Zero + Clone + PartialOrd,
{
    pub fn on(root: &'a DrawingArea<DB, Shift>) -> Self {
        AutoRange {
            chart: ChartBuilder::on(root),
            xrange: X::zero()..X::zero(),
            yrange: Y::zero()..Y::zero(),
            series: vec![],
        }
    }

    fn extend<A>(range: &mut Range<A>, value: &A)
    where
        A: PartialOrd + Clone,
    {
        if value < &range.start {
            range.start = value.clone();
        }
        if value > &range.end {
            range.end = value.clone();
        }
    }

    pub fn add_series<E, S, F>(&mut self, series: S, anno: F)
    where
        E: IntoDynElement<'static, DB, (X, Y)>,
        S: IntoIterator<Item=E>,
        F: FnOnce(&mut SeriesAnno<'a, DB>) + 'static,
    {
        let data: Vec<_> = series.into_iter().map(|e| e.into_dyn()).collect();
        for el in data.iter() {
            for (x, y) in el.point_iter() {
                Self::extend(&mut self.xrange, x);
                Self::extend(&mut self.yrange, y);
            }
        }
        self.series.push((data, Box::new(anno)));
    }

    pub fn build(&mut self, xrange: Option<Range<X>>, yrange: Option<Range<Y>>)
                 -> Result<ChartContext<'a, DB, RangedCoord<<Range<X> as AsRangedCoord>::CoordDescType, <Range<Y> as AsRangedCoord>::CoordDescType>>, DrawingAreaErrorKind<DB::ErrorType>>
    where
        Range<X>: AsRangedCoord,
        Range<Y>: AsRangedCoord,
    {
        let xrange = xrange.unwrap_or(self.xrange.clone());
        let yrange = yrange.unwrap_or(self.yrange.clone());
        self.chart.build_ranged(xrange, yrange)
    }

    pub fn draw(&mut self, chart: &mut ChartContext<'a, DB, RangedCoord<<Range<X> as AsRangedCoord>::CoordDescType, <Range<Y> as AsRangedCoord>::CoordDescType>>)
                -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
    where
        Range<X>: AsRangedCoord<Value=X>,
        Range<Y>: AsRangedCoord<Value=Y>,
    {
        for (data, annotate) in self.series.drain(..) {
            let ann = chart.draw_series(data.into_iter())?;
            annotate(ann);
        }
        Ok(())
    }
}
