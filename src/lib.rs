use std::cmp;
use std::default::Default;

use drawille::Canvas as BrailleCanvas;
use drawille::PixelColor;

pub mod scale;

use scale::Scale;

pub struct RGB8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
    /// Draws a [line chart](https://en.wikipedia.org/wiki/Line_chart) of points connected by straight line segments.
    fn lineplot(&'a mut self, shape: Shape, color: Option<RGB8>) -> &'a mut Chart;
}

pub trait Data<'a> {
    fn data(&'a mut self, data_points: Vec<(f32, f32)>) -> &'a mut Chart;
}

impl<'a> Default for Chart {
    fn default() -> Self {
        Self::new(120, 60, -10.0, 10.0)
    }
}

impl<'a> Chart {
    /// Creates a new `Chart` object.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is less than 32.
    pub fn new(width: u32, height: u32, xmin: f32, xmax: f32) -> Self {
        if width < 32 {
            panic!("width should be more then 32, {} is provided", width);
        }

        if height < 32 {
            panic!("height should be more then 32, {} is provided", height);
        }

        Self {
            xmin,
            xmax,
            x_ranging: ChartRangeMethod::AutoRange,
            ymin: f32::INFINITY,
            ymax: f32::NEG_INFINITY,
            y_ranging: ChartRangeMethod::AutoRange,
            width,
            height,
            data_points: Vec::new(),
            appearance: Vec::new(),
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
            x_ranging: ChartRangeMethod::AutoRange,
            ymin,
            ymax,
            y_ranging: ChartRangeMethod::FixedRange,
            width,
            height,
            data_points: Vec::new(),
            appearance: Vec::new(),
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
        self.figures();
        self.axis();

        let mut frame = self.canvas.frame();
        if let Some(idx) = frame.find('\n') {
            frame.insert_str(idx, &format!(" {0:.1}", self.ymax));
            frame.push_str(&format!(
                " {0:.1}\n{1: <width$.1}{2:.1}\n",
                self.ymin,
                self.xmin,
                self.xmax,
                width = (self.width as usize) / 2 - 3
            ));
        }
        frame
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

    /// Show axis.
    pub fn axis(&mut self) {
        let x_scale = Scale::new(self.xmin..self.xmax, 0.0..self.width as f32);
        let y_scale = Scale::new(self.ymin..self.ymax, 0.0..self.height as f32);

        if self.xmin <= 0.0 && self.xmax >= 0.0 {
            self.vline(x_scale.linear(0.0) as u32);
        }
        if self.ymin <= 0.0 && self.ymax >= 0.0 {
            self.hline(y_scale.linear(0.0) as u32);
        }
    }

    // Show figures.
    pub fn figures(&mut self) {
        for (shape, color) in &self.appearance {
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
                        if let Some(color) = color {
                            let color = rgb_to_pixelcolor(color);
                            self.canvas.line_colored(x1, y1, x2, y2, color);
                        } else {
                            self.canvas.line(x1, y1, x2, y2);
                        }
                    }
                }
                Shape::Steps => {
                    for pair in points.windows(2) {
                        let (x1, y1) = pair[0];
                        let (x2, y2) = pair[1];

                        if let Some(color) = color {
                            let color = rgb_to_pixelcolor(color);
                            self.canvas.line_colored(x1, y2, x2, y2, color);
                            self.canvas.line_colored(x1, y1, x1, y2, color);
                        } else {
                            self.canvas.line(x1, y2, x2, y2);
                            self.canvas.line(x1, y1, x1, y2);
                        }
                    }
                }
                Shape::Bars => {
                    for pair in points.windows(2) {
                        let (x1, y1) = pair[0];
                        let (x2, y2) = pair[1];

                        if let Some(color) = color {
                            let color = rgb_to_pixelcolor(color);
                            self.canvas.line_colored(x1, y2, x2, y2, color);
                            self.canvas.line_colored(x1, y1, x1, y2, color);
                            self.canvas.line_colored(x1, self.height, x1, y1, color);
                            self.canvas.line_colored(x2, self.height, x2, y2, color);
                        } else {
                            self.canvas.line(x1, y2, x2, y2);
                            self.canvas.line(x1, y1, x1, y2);
                            self.canvas.line(x1, self.height, x1, y1);
                            self.canvas.line(x2, self.height, x2, y2);
                        }
                    }
                }
            }
        }
    }

    /// Return the frame.
    pub fn frame(&self) -> String {
        self.canvas.frame()
    }

    fn rescale(&mut self, shape: &Shape) {
        // rescale ymin and ymax
        let x_scale = Scale::new(self.xmin..self.xmax, 0.0..self.width as f32);

        let ys: Vec<_> = self
            .data_points
            .iter()
            .filter_map(|(x, y)| {
                if *x >= self.xmin && *x <= self.xmax {
                    Some(*y)
                } else {
                    None
                }
            })
            .collect();

        let ymax = *ys
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(cmp::Ordering::Equal))
            .unwrap_or(&0.0);
        let ymin = *ys
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap_or(cmp::Ordering::Equal))
            .unwrap_or(&0.0);

        self.ymin = f32::min(self.ymin, ymin);
        self.ymax = f32::max(self.ymax, ymax);
    }
}

impl<'a> Plot<'a> for Chart {
    fn lineplot(&'a mut self, shape: Shape, color: Option<RGB8>) -> &'a mut Chart {
        self.appearance.push((shape.clone(), color));
        if self.y_ranging == ChartRangeMethod::AutoRange {
            self.rescale(&shape);
        }
        self
    }
}

impl<'a> Data<'a> for Chart {
    fn data(&'a mut self, data_points: Vec<(f32, f32)>) -> &'a mut Chart {
        self.data_points = data_points;
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
