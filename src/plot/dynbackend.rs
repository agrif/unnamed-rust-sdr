use plotters::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use plotters::style::{FontDesc, RGBAColor, ShapeStyle, TextStyle};

// a Box<dyn Error> that can implement Error without overlapping
// impls for From
#[derive(Debug)]
pub struct DynError<'a> {
    inner: Box<dyn std::error::Error + Send + Sync + 'a>,
}

impl<'a> DynError<'a> {
    pub fn new<E: std::error::Error + Send + Sync + 'a>(err: E) -> Self {
        DynError {
            inner: Box::new(err),
        }
    }
}

impl<'a> std::fmt::Display for DynError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a> std::error::Error for DynError<'a> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }

    /*
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        self.inner.backtrace()
    }
     */

    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.inner.description()
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.inner.cause()
    }
}

// private helper for making DrawingErrorKind<DynError>
fn wrap_err<'a, T, E>(
    r: Result<T, DrawingErrorKind<E>>,
) -> Result<T, DrawingErrorKind<DynError<'a>>>
where
    E: std::error::Error + Send + Sync + 'a,
{
    match r {
        Ok(v) => Ok(v),
        Err(err) => Err(match err {
            DrawingErrorKind::DrawingError(inner) => {
                DrawingErrorKind::DrawingError(DynError::new(inner))
            }
            DrawingErrorKind::FontError(e) => DrawingErrorKind::FontError(e),
        }),
    }
}

// a type-erased, trait object compatible version of DrawingBackend
pub trait DynDrawingBackend<'a>: 'a {
    fn dyn_get_size(&self) -> (u32, u32);
    fn dyn_ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_present(&mut self) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_draw_pixel(
        &mut self,
        point: BackendCoord,
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_draw_line(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_draw_rect(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &ShapeStyle,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_draw_path(
        &mut self,
        path: Vec<BackendCoord>,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_draw_circle(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &ShapeStyle,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_fill_polygon(
        &mut self,
        vert: Vec<BackendCoord>,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_draw_text(
        &mut self,
        text: &str,
        style: &TextStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>>;
    fn dyn_estimate_text_size<'b>(
        &self,
        text: &str,
        font: &FontDesc<'b>,
    ) -> Result<(u32, u32), DrawingErrorKind<DynError<'a>>>;
    fn dyn_blit_bitmap<'b>(
        &mut self,
        pos: BackendCoord,
        dim: (u32, u32),
        src: &'b [u8],
    ) -> Result<(), DrawingErrorKind<DynError<'a>>>;
}

pub fn erase<'a, DB>(backend: DB) -> Box<dyn DynDrawingBackend<'a>>
where
    DB: DrawingBackend + 'a,
{
    Box::new(backend)
}

// blanket implementation
impl<'a, T> DynDrawingBackend<'a> for T
where
    T: DrawingBackend + 'a,
    T::ErrorType: 'a,
{
    fn dyn_get_size(&self) -> (u32, u32) {
        self.get_size()
    }
    fn dyn_ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.ensure_prepared())
    }
    fn dyn_present(&mut self) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.present())
    }
    fn dyn_draw_pixel(
        &mut self,
        point: BackendCoord,
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.draw_pixel(point, color))
    }
    fn dyn_draw_line(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.draw_line(from, to, style))
    }
    fn dyn_draw_rect(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &ShapeStyle,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.draw_rect(upper_left, bottom_right, style, fill))
    }
    fn dyn_draw_path(
        &mut self,
        path: Vec<BackendCoord>,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.draw_path(path, style))
    }
    fn dyn_draw_circle(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &ShapeStyle,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.draw_circle(center, radius, style, fill))
    }
    fn dyn_fill_polygon(
        &mut self,
        vert: Vec<BackendCoord>,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.fill_polygon(vert, style))
    }
    fn dyn_draw_text(
        &mut self,
        text: &str,
        style: &TextStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.draw_text(text, style, pos))
    }
    fn dyn_estimate_text_size<'b>(
        &self,
        text: &str,
        font: &FontDesc<'b>,
    ) -> Result<(u32, u32), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.estimate_text_size(text, font))
    }
    fn dyn_blit_bitmap<'b>(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &'b [u8],
    ) -> Result<(), DrawingErrorKind<DynError<'a>>> {
        wrap_err(self.blit_bitmap(pos, (iw, ih), src))
    }
}

// helper for styles
fn wrap_style<S: BackendStyle>(style: &S) -> ShapeStyle {
    let dynstyle: ShapeStyle = (&style.as_color()).into();
    dynstyle.stroke_width(style.stroke_width())
}

macro_rules! impl_drawing_backend {
    ($($l:tt)*) => {
        impl<'a> DrawingBackend for $($l)* {
            type ErrorType = DynError<'a>;
            fn get_size(&self) -> (u32, u32) {
                (**self).dyn_get_size()
            }
            fn ensure_prepared(&mut self)
                               -> Result<(), DrawingErrorKind<Self::ErrorType>>
            {
                (**self).dyn_ensure_prepared()
            }
            fn present(&mut self)
                       -> Result<(), DrawingErrorKind<Self::ErrorType>>
            {
                (**self).dyn_present()
            }
            fn draw_pixel(&mut self, point: BackendCoord, color: &RGBAColor)
                          -> Result<(), DrawingErrorKind<Self::ErrorType>> {
                (**self).dyn_draw_pixel(point, color)
            }
            fn draw_line<S: BackendStyle>(&mut self, from: BackendCoord,
                                          to: BackendCoord, style: &S,
            ) -> Result<(), DrawingErrorKind<Self::ErrorType>>
            {
                (**self).dyn_draw_line(from, to, &wrap_style(style))
            }
            fn draw_rect<S: BackendStyle>(&mut self, upper_left: BackendCoord,
                                          bottom_right: BackendCoord, style: &S,
                                          fill: bool,
            ) -> Result<(), DrawingErrorKind<Self::ErrorType>>
            {
                (**self).dyn_draw_rect(upper_left, bottom_right, &wrap_style(style), fill)
            }
            fn draw_path<S: BackendStyle, I: IntoIterator<Item=BackendCoord>>(
                &mut self, path: I, style: &S,
            ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
                (**self).dyn_draw_path(path.into_iter().collect(), &wrap_style(style))
            }
            fn draw_circle<S: BackendStyle>(&mut self, center: BackendCoord, radius: u32, style: &S, fill: bool) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
                (**self).dyn_draw_circle(center, radius, &wrap_style(style), fill)
            }
            fn fill_polygon<S: BackendStyle, I: IntoIterator<Item=BackendCoord>>(
                &mut self, vert: I, style: &S,
            ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
                (**self).dyn_fill_polygon(vert.into_iter().collect(), &wrap_style(style))
            }
            fn draw_text(&mut self, text: &str, style: &TextStyle, pos: BackendCoord) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
                (**self).dyn_draw_text(text, style, pos)
            }
            fn estimate_text_size<'b>(&self, text: &str, font: &FontDesc<'b>) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
                (**self).dyn_estimate_text_size(text, font)
            }
            fn blit_bitmap<'b>(&mut self, pos: BackendCoord, (iw, ih): (u32, u32), src: &'b [u8]) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
                (**self).dyn_blit_bitmap(pos, (iw, ih), src)
            }
        }
    }
}

impl_drawing_backend!(&mut (dyn DynDrawingBackend<'a>));
impl_drawing_backend!(&mut (dyn DynDrawingBackend<'a> + Send));
impl_drawing_backend!(&mut (dyn DynDrawingBackend<'a> + Sync));
impl_drawing_backend!(&mut (dyn DynDrawingBackend<'a> + Send + Sync));
impl_drawing_backend!(Box<dyn DynDrawingBackend<'a>>);
impl_drawing_backend!(Box<dyn DynDrawingBackend<'a> + Send>);
impl_drawing_backend!(Box<dyn DynDrawingBackend<'a> + Sync>);
impl_drawing_backend!(Box<dyn DynDrawingBackend<'a> + Send + Sync>);
