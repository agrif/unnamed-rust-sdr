use plotters::prelude::*;

pub struct ReImSeries<DB: DrawingBackend, X, Y> {
    restyle: ShapeStyle,
    imstyle: ShapeStyle,
    redata: Vec<(X, Y)>,
    imdata: Vec<(X, Y)>,
    point_idx: usize,
    point_size: u32,
    _phantom: std::marker::PhantomData<DB>,
}

impl<DB, X, Y> ReImSeries<DB, X, Y>
where
    DB: DrawingBackend,
    X: Clone,
    Y: Clone,
{
    pub fn new<I, RS, IS>(iter: I, restyle: RS, imstyle: IS) -> Self
    where
        I: IntoIterator<Item=(X, num::Complex<Y>)>,
        RS: Into<ShapeStyle>,
        IS: Into<ShapeStyle>,
    {
        let data = iter.into_iter();
        let size = data.size_hint().0;
        let mut redata = Vec::with_capacity(size);
        let mut imdata = Vec::with_capacity(size);
        for (x, y) in data {
            redata.push((x.clone(), y.re));
            imdata.push((x, y.im));
        }
        ReImSeries {
            restyle: restyle.into(),
            imstyle: imstyle.into(),
            redata: redata,
            imdata: imdata,
            point_size: 0,
            point_idx: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn point_size(mut self, size: u32) -> Self {
        self.point_size = size;
        self
    }
}

impl<DB, X, Y> Iterator for ReImSeries<DB, X, Y>
where
    DB: DrawingBackend,
    X: Clone + 'static,
    Y: Clone + 'static,
{
    type Item = DynElement<'static, DB, (X, Y)>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.redata.is_empty() {
            if self.point_size > 0 && self.point_idx < self.redata.len() {
                let idx = self.point_idx;
                self.point_idx += 1;
                let (x, ry) = self.redata[idx].clone();
                return Some(
                    Circle::new((x, ry), self.point_size, self.restyle.clone())
                        .into_dyn());
            }
            let mut redata = vec![];
            std::mem::swap(&mut self.redata, &mut redata);
            Some(PathElement::new(redata, self.restyle.clone()).into_dyn())
        } else if !self.imdata.is_empty() {
            if self.point_size > 0 && self.point_idx < self.imdata.len() {
                let idx = self.point_idx;
                self.point_idx += 1;
                let (x, iy) = self.imdata[idx].clone();
                return Some(
                    Circle::new((x, iy), self.point_size, self.imstyle.clone())
                        .into_dyn());
            }
            let mut imdata = vec![];
            std::mem::swap(&mut self.imdata, &mut imdata);
            Some(PathElement::new(imdata, self.imstyle.clone()).into_dyn())
        } else {
            None
        }
    }
}
