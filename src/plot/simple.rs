use super::autorange::AutoRange;
use super::reimseries::ReImSeries;
use super::complexseries::ComplexSeries;

use std::ops::Range;
use std::fmt::Debug;
use plotters::prelude::*;
use plotters::coord::{AsRangedCoord, Shift};
use plotters::drawing::backend::BackendCoord;
use palette::Hsv;

pub struct Simple<'a, 'b, DB: DrawingBackend, X: Clone, Y: Clone> {
    auto: AutoRange<'a, 'b, DB, X, Y>,
    color_idx: usize,
    stroke_width: u32,
    draw_legend: bool,
    xlabel: Option<&'a str>,
    ylabel: Option<&'a str>,
}

impl<'a, 'b, DB, X, Y> Simple<'a, 'b, DB, X, Y>
where
    DB: DrawingBackend + 'a,
    X: Clone + Debug + num::Float + 'static,
    Y: Clone + Debug + num::Float + 'static,
    Range<X>: AsRangedCoord<Value=X>,
    Range<Y>: AsRangedCoord<Value=Y>,
{
    pub fn on(root: &'a DrawingArea<DB, Shift>) -> Self {
        let mut auto = AutoRange::on(root);
        auto.margin(5)
            .x_label_area_size(30)
            .y_label_area_size(60);
        Simple {
            auto: auto,
            color_idx: 0,
            stroke_width: 2,
            draw_legend: false,
            xlabel: None,
            ylabel: None,
        }
    }

    pub fn title(&mut self, title: &'a str) -> &mut Self {
        self.auto.caption(title, ("sans-serif", 30).into_font());
        self
    }

    pub fn xlabel(&mut self, xlabel: &'a str) -> &mut Self {
        self.xlabel = Some(xlabel);
        self
    }

    pub fn ylabel(&mut self, ylabel: &'a str) -> &mut Self {
        self.ylabel = Some(ylabel);
        self
    }

    pub fn generate_style(&mut self) -> ShapeStyle {
        let style: ShapeStyle = Palette99::pick(self.color_idx)
            .stroke_width(self.stroke_width);
        self.color_idx += 1;
        style
    }

    pub fn add_series<S, F, FE>(
        &mut self,
        series: S,
        label: Option<(&str, F)>,
    ) -> &mut Self
    where
        S: IntoIterator<Item=DynElement<'static, DB, (X, Y)>>,
        F: Fn(BackendCoord) -> FE + 'static,
        FE: IntoDynElement<'a, DB, BackendCoord>,
    {
        if let Some(anno) = label {
            let name = anno.0.to_owned();
            let el = anno.1;
            self.auto.add_series(series, move |a| {
                a.label(name).legend(el);
            });
            self.draw_legend = true;
        } else {
            self.auto.add_series(series, |_a| ());
        }
        self
    }

    pub fn add_line<I>(&mut self, data: I, label: Option<&str>) -> &mut Self
    where
        I: IntoIterator<Item=(X, Y)>,
    {
        let style = self.generate_style();
        let stylec = style.clone();
        let legend = label.map(|n| {
            (n, move |(x, y)|
             PathElement::new(vec![(x, y), (x + 20, y)], stylec.clone()))
        });
        self.add_series(LineSeries::new(data, style), legend)
    }

    pub fn add_reim<I>(&mut self, data: I, label: Option<&str>) -> &mut Self
    where
        I: IntoIterator<Item=(X, num::Complex<Y>)>,
    {
        let restyle = self.generate_style();
        let imstyle = self.generate_style();
        let restylec = restyle.clone();
        let imstylec = imstyle.clone();
        let legend = label.map(|n| {
            (n, move |(x, y)|
             EmptyElement::at((x, y))
             + PathElement::new(vec![(0, -4), (20, -4)], restylec.clone())
             + PathElement::new(vec![(0, 4), (20, 4)], imstylec.clone())
            )
        });
        self.add_series(ReImSeries::new(data, restyle, imstyle), legend)
    }

    pub fn add_complex<I>(&mut self, data: I, db: bool, label: Option<&str>)
                          -> &mut Self
    where
        I: IntoIterator<Item=(X, num::Complex<Y>)>,
    {
        let style = self.generate_style();
        let stylec = style.clone();
        let legend = label.map(|n| {
            (n, move |(x, y)|
             PathElement::new(vec![(x, y), (x + 20, y)], stylec.clone()))
        });
        self.add_series(ComplexSeries::new(data, db, style), legend)
    }

    pub fn add_complex_hue<I>(&mut self, data: I, db: bool, label: Option<&str>)
                              -> &mut Self
    where
        I: IntoIterator<Item=(X, num::Complex<Y>)>,
    {
        let sw = self.stroke_width;
        let legend = label.map(|n| {
            (n, move |(x, y)|
             EmptyElement::at((x, y))
             + PathElement::new(vec![(0, 0), (2, 0)],
                                ShapeStyle::from(&Hsv::new(0.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(2, 0), (4, 0)],
                                ShapeStyle::from(&Hsv::new(36.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(4, 0), (6, 0)],
                                ShapeStyle::from(&Hsv::new(72.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(6, 0), (8, 0)],
                                ShapeStyle::from(&Hsv::new(108.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(8, 0), (10, 0)],
                                ShapeStyle::from(&Hsv::new(144.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(10, 0), (12, 0)],
                                ShapeStyle::from(&Hsv::new(180.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(12, 0), (14, 0)],
                                ShapeStyle::from(&Hsv::new(216.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(14, 0), (16, 0)],
                                ShapeStyle::from(&Hsv::new(252.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(16, 0), (18, 0)],
                                ShapeStyle::from(&Hsv::new(288.0, 1.0, 1.0))
                                .stroke_width(sw))
             + PathElement::new(vec![(18, 0), (20, 0)],
                                ShapeStyle::from(&Hsv::new(324.0, 1.0, 1.0))
                                .stroke_width(sw))
            )
        });
        self.add_series(ComplexSeries::new_hue(data, db, self.stroke_width),
                        legend)
    }

    pub fn draw(&mut self)
                -> Result<&mut Self, DrawingAreaErrorKind<DB::ErrorType>>
    {
        let mut chart = self.auto.build(None, None)?;
        chart.configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .x_desc(self.xlabel.unwrap_or(""))
            .y_desc(self.ylabel.unwrap_or(""))
            .draw()?;

        self.auto.draw(&mut chart)?;

        if self.draw_legend {
            chart.configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .draw()?;
        }

        Ok(self)
    }
}

