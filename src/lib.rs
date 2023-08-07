use std::default::Default;

use drawille::Canvas as BrailleCanvas;
use drawille::PixelColor;

pub mod scale;

use scale::Scale;

#[derive(Clone)]
pub struct RGB8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB8 {
    pub fn new(r: u8, g: u8, b: u8) -> RGB8 {
        RGB8 { r, g, b }
    }

    pub fn new_hex_str(hex_str: &str) -> RGB8 {
        fn str_to_u8(hex_str: &str) -> u8 {
            u8::from_str_radix(&hex_str, 16).unwrap()
        }
        if hex_str.chars().count() < 6 || hex_str.chars().count() > 7 {
            panic!("Length of hex string must be 6 or 7, {}", hex_str);
        }
        let hex_str = hex_str.replace("#", "");
        let r = str_to_u8(&hex_str[0..2]);
        let g = str_to_u8(&hex_str[2..4]);
        let b = str_to_u8(&hex_str[4..6]);
        RGB8 { r, g, b }
    }
}

/// How the chart will do the ranging on axes
#[derive(PartialEq)]
enum ChartRangeMethod {
    /// Automatically ranges based on input data
    AutoRange,
    /// Has a fixed range between the given min & max
    FixedRange,
}

/// Controls the drawing.
pub struct Chart {
    /// Canvas width in points.
    width: u32,
    /// Canvas height in points.
    height: u32,
    /// X-axis start value.
    xmin: f32,
    /// X-axis end value.
    xmax: f32,
    /// The type of x axis ranging used
    x_ranging: ChartRangeMethod,
    /// Y-axis start value (potentially calculated automatically).
    ymin: f32,
    /// Y-axis end value (potentially calculated automatically).
    ymax: f32,
    /// The type of y axis ranging used
    y_ranging: ChartRangeMethod,
    /// Data points to plot
    data_points: Vec<(f32, f32)>,
    /// Collection of shapes to be presented on the canvas.
    appearance: Vec<(Shape, Option<RGB8>)>,
    /// If true, show x and y axis.
    show_axis: bool,
    /// Function to apply to X-axis ticks.
    xtick: Option<Box<dyn Fn(f32) -> String>>,
    /// Function to apply to Y-axis ticks.
    ytick: Option<Box<dyn Fn(f32) -> String>>,
    /// Underlying canvas object.
    canvas: BrailleCanvas,
}

/// Specifies different kinds of plotted data.
#[derive(Clone)]
pub enum Shape {
    Points,
    Lines,
    Steps,
    Bars,
}

/// Provides an interface for drawing plots.
pub trait Plot<'a> {
    /// Sets the data points that will be plotted
    fn data(&'a mut self, data_points: Vec<(f32, f32)>) -> &'a mut Chart;
    /// Draws a [line chart](https://en.wikipedia.org/wiki/Line_chart) of points connected by straight line segments.
    fn lineplot(&'a mut self, shape: Shape, color: Option<RGB8>) -> &'a mut Chart;
    /// Hides the x and y axis.
    fn hide_axis(&'a mut self) -> &'a mut Chart;
}

impl<'a> Default for Chart {
    fn default() -> Self {
        Self::new(120, 60)
    }
}

impl<'a> Chart {
    /// Creates a new `Chart` object.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is less than 32.
    pub fn new(width: u32, height: u32) -> Self {
        if width < 32 {
            panic!("width should be more then 32, {} is provided", width);
        }

        if height < 32 {
            panic!("height should be more then 32, {} is provided", height);
        }

        Self {
            xmin: f32::INFINITY,
            xmax: f32::NEG_INFINITY,
            x_ranging: ChartRangeMethod::AutoRange,
            ymin: f32::INFINITY,
            ymax: f32::NEG_INFINITY,
            y_ranging: ChartRangeMethod::AutoRange,
            width,
            height,
            data_points: Vec::new(),
            appearance: Vec::new(),
            show_axis: true,
            xtick: None,
            ytick: None,
            canvas: BrailleCanvas::new(width, height),
        }
    }

    /// Creates a new `Chart` object with fixed y axis range.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is less than 32.
    pub fn with_range(width: u32, height: u32, xmin: f32, xmax: f32, ymin: f32, ymax: f32) -> Self {
        if width < 32 {
            panic!("width should be more then 32, {} is provided", width);
        }

        if height < 32 {
            panic!("height should be more then 32, {} is provided", height);
        }

        Self {
            xmin,
            xmax,
            x_ranging: ChartRangeMethod::FixedRange,
            ymin,
            ymax,
            y_ranging: ChartRangeMethod::FixedRange,
            width,
            height,
            data_points: Vec::new(),
            appearance: Vec::new(),
            show_axis: true,
            xtick: None,
            ytick: None,
            canvas: BrailleCanvas::new(width, height),
        }
    }

    /// Displays bounding rect.
    fn borders(&mut self) {
        let w = self.width;
        let h = self.height;

        self.vline(0);
        self.vline(w);
        self.hline(0);
        self.hline(h);
    }

    /// Draws vertical line.
    fn vline(&mut self, i: u32) {
        if i <= self.width {
            for j in 0..=self.height {
                if j % 3 == 0 {
                    self.canvas.set(i, j);
                }
            }
        }
    }

    /// Draws horizontal line.
    fn hline(&mut self, j: u32) {
        if j <= self.height {
            for i in 0..=self.width {
                if i % 3 == 0 {
                    self.canvas.set(i, self.height - j);
                }
            }
        }
    }

    pub fn to_string(&mut self) -> String {
        if self.show_axis {
            self.null_axis();
        }
        self.figures();

        let mut frame = self.canvas.frame();

        if self.show_axis {
            self.show_num_label(&mut frame);
        }
        frame
    }

    fn show_num_label(&mut self, frame: &mut String) {
        let xmin = self.format_xaxis_tick(self.xmin);
        let xmax = self.format_xaxis_tick(self.xmax);

        if let Some(idx) = frame.find('\n') {
            frame.insert_str(idx, &format!(" {}", self.format_yaxis_tick(self.ymax)));
            frame.push_str(&format!(
                " {0}\n{1: <width$}{2}\n",
                self.format_yaxis_tick(self.ymin),
                xmin,
                xmax,
                width = (self.width as usize) / 2 - xmax.chars().count(),
            ));
        }
    }

    /// Prints canvas content.
    pub fn display(&mut self) {
        println!("{}", self.to_string());
    }

    /// Prints canvas content with some additional visual elements (like borders).
    pub fn nice(&mut self) {
        self.borders();
        self.display();
    }

    /// Show axis at x = 0 and y = 0 if in view
    pub fn null_axis(&mut self) {
        let x_scale = Scale::new(self.xmin..self.xmax, 0.0..self.width as f32);
        let y_scale = Scale::new(self.ymin..self.ymax, 0.0..self.height as f32);

        if self.xmin <= 0.0 && self.xmax >= 0.0 {
            self.vline(x_scale.linear(0.0) as u32);
        }
        if self.ymin <= 0.0 && self.ymax >= 0.0 {
            self.hline(y_scale.linear(0.0) as u32);
        }
    }

    fn render_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: &Option<RGB8>) {
        if let Some(color) = color {
            let color = rgb_to_pixelcolor(color);
            self.canvas.line_colored(x1, y1, x2, y2, color);
        } else {
            self.canvas.line(x1, y1, x2, y2);
        }
    }

    fn figure(&mut self, shape: &Shape, color: &Option<RGB8>) {
        let x_scale = Scale::new(self.xmin..self.xmax, 0.0..self.width as f32);
        let y_scale = Scale::new(self.ymin..self.ymax, 0.0..self.height as f32);

        // translate (x, y) points into screen coordinates
        let points: Vec<_> = self
            .data_points
            .iter()
            .filter_map(|(x, y)| {
                let i = x_scale.linear(*x).round() as u32;
                let j = y_scale.linear(*y).round() as u32;
                if i <= self.width && j <= self.height {
                    Some((i, self.height - j))
                } else {
                    None
                }
            })
            .collect();

        // display segments
        match shape {
            Shape::Points => {
                for (x, y) in points {
                    self.canvas.set(x, y);
                }
            }
            Shape::Lines => {
                for pair in points.windows(2) {
                    let (x1, y1) = pair[0];
                    let (x2, y2) = pair[1];
                    self.render_line(x1, y1, x2, y2, color);
                }
            }
            Shape::Steps => {
                for pair in points.windows(2) {
                    let (x1, y1) = pair[0];
                    let (x2, y2) = pair[1];
                    self.render_line(x1, y2, x2, y2, color);
                    self.render_line(x1, y1, x1, y2, color);
                }
            }
            Shape::Bars => {
                for pair in points.windows(2) {
                    let (x1, y1) = pair[0];
                    let (x2, y2) = pair[1];

                    self.render_line(x1, y2, x2, y2, color);
                    self.render_line(x1, y1, x1, y2, color);
                    self.render_line(x1, self.height, x1, y1, color);
                    self.render_line(x2, self.height, x2, y2, color);
                }
            }
        }
    }

    // Show figures.
    pub fn figures(&mut self) {
        for (shape, color) in self.appearance.clone() {
            self.figure(&shape, &color)
        }
    }

    /// Return the frame.
    pub fn frame(&self) -> String {
        self.canvas.frame()
    }

    fn format_xaxis_tick(&self, value: f32) -> String {
        if let Some(ref f) = self.xtick {
            f(value)
        } else {
            format!("{:.1}", value)
        }
    }

    fn format_yaxis_tick(&self, value: f32) -> String {
        if let Some(ref f) = self.ytick {
            f(value)
        } else {
            format!("{:.1}", value)
        }
    }

    /// Sets the function for the X-axis ticks.
    pub fn set_xtick<F: 'static + Fn(f32) -> String>(mut self, f: F) -> Self {
        self.xtick = Some(Box::new(f));
        self
    }

    /// Sets the function for the Y-axis ticks.
    pub fn set_ytick<F: 'static + Fn(f32) -> String>(mut self, f: F) -> Self {
        self.ytick = Some(Box::new(f));
        self
    }

    fn rescale_x(&mut self) {
        let xmin = self
            .data_points
            .iter()
            .map(|&(x, _)| x)
            .fold(f32::INFINITY, |min_x, x| min_x.min(x));
        let xmax = self
            .data_points
            .iter()
            .map(|&(x, _)| x)
            .fold(f32::NEG_INFINITY, |max_x, x| max_x.max(x));

        self.xmin = f32::min(self.xmin, xmin);
        self.xmax = f32::max(self.xmax, xmax);
    }

    fn rescale_y(&mut self) {
        let ymin = self
            .data_points
            .iter()
            .map(|&(_, y)| y)
            .fold(f32::INFINITY, |min_y, y| min_y.min(y));
        let ymax = self
            .data_points
            .iter()
            .map(|&(_, y)| y)
            .fold(f32::NEG_INFINITY, |max_y, y| max_y.max(y));

        self.ymin = f32::min(self.ymin, ymin);
        self.ymax = f32::max(self.ymax, ymax);
    }
}

impl<'a> Plot<'a> for Chart {
    fn data(&'a mut self, data_points: Vec<(f32, f32)>) -> &'a mut Chart {
        self.data_points = data_points;
        self
    }

    fn lineplot(&'a mut self, shape: Shape, color: Option<RGB8>) -> &'a mut Chart {
        self.appearance.push((shape.clone(), color));
        if self.x_ranging == ChartRangeMethod::AutoRange {
            self.rescale_x();
        }
        if self.y_ranging == ChartRangeMethod::AutoRange {
            self.rescale_y();
        }
        self
    }

    fn hide_axis(&'a mut self) -> &'a mut Chart {
        self.show_axis = false;
        self
    }
}

fn rgb_to_pixelcolor(rgb: &RGB8) -> PixelColor {
    PixelColor::TrueColor {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
    }
}
