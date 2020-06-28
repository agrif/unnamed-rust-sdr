use plotters::prelude::*;

pub struct ComplexSeries<DB: DrawingBackend, X, Y> {
    style: Option<ShapeStyle>,
    stroke_width: u32,
    db: bool,
    data: Vec<(X, num::Complex<Y>)>,
    point_phase: bool,
    point_idx: usize,
    point_size: u32,
    _phantom: std::marker::PhantomData<DB>,
}

impl<DB, X, Y> ComplexSeries<DB, X, Y>
where
    DB: DrawingBackend,
    X: Clone,
    Y: Clone,
{
    pub fn new<I, S>(iter: I, db: bool, style: S) -> Self
    where
        I: IntoIterator<Item=(X, num::Complex<Y>)>,
        S: Into<ShapeStyle>,
    {
        let style = style.into();
        ComplexSeries {
            stroke_width: style.stroke_width,
            style: Some(style),
            db,
            data: iter.into_iter().collect(),
            point_phase: false,
            point_size: 0,
            point_idx: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_hue<I>(iter: I, db: bool, stroke_width: u32) -> Self
    where
        I: IntoIterator<Item=(X, num::Complex<Y>)>,
    {
        ComplexSeries {
            style: None,
            stroke_width: stroke_width,
            db,
            data: iter.into_iter().collect(),
            point_phase: false,
            point_size: 0,
            point_idx: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn point_size(mut self, size: u32) -> Self {
        self.point_size = size;
        self
    }

    fn style(&self, v: num::Complex<Y>) -> ShapeStyle
    where
        Y: num::Float,
    {
        let mut hue = v.arg().to_degrees().to_f32().unwrap();
        if hue < 0.0 {
            hue += 360.0;
        }
        let s: ShapeStyle = (&palette::Hsv::new(hue, 1.0, 1.0)).into();
        s.stroke_width(self.stroke_width)
    }
}

impl<DB, X, Y> Iterator for ComplexSeries<DB, X, Y>
where
    DB: DrawingBackend,
    X: Clone + 'static,
    Y: Clone + num::Float + 'static,
{
    type Item = DynElement<'static, DB, (X, Y)>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.data.is_empty() {
            if self.point_idx < self.data.len() {
                let idx = self.point_idx;
                self.point_idx += 1;

                if self.point_phase {
                    // make points
                    let (x, y) = self.data[idx].clone();
                    let style = self.style.clone()
                        .unwrap_or_else(|| self.style(y));
                    let mut yn = y.norm();
                    if self.db {
                        yn = Y::from(20.0).unwrap() * yn.log10()
                    }
                    return Some(Circle::new((x, yn),
                                            self.point_size, style).into_dyn());
                } else {
                    // make lines
                    if idx < self.data.len() - 1 {
                        let (x1, y1) = self.data[idx].clone();
                        let mut y1n = y1.norm();
                        let (x2, y2) = self.data[idx + 1].clone();
                        let mut y2n = y2.norm();
                        let avg = (y1 + y2) / Y::from(2.0).unwrap();
                        let style = self.style.clone()
                            .unwrap_or_else(|| self.style(avg));
                        if self.db {
                            let twenty = Y::from(20.0).unwrap();
                            y1n = twenty * y1n.log10();
                            y2n = twenty * y2n.log10();
                        }
                        return Some(PathElement::new(vec![(x1, y1n), (x2, y2n)],
                                                     style).into_dyn());
                    }
                }
            }

            if self.point_phase || self.point_size == 0 {
                self.data.clear();
                return None;
            } else {
                self.point_idx = 0;
                self.point_phase = true;
                return self.next();
            }
        } else {
            None
        }
    }
}
