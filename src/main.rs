use conplot::RGB8;
use conplot::{Chart, ColorPlot, Plot, Shape};

fn main() {
    let points = [(-5.0, 3.0), (3.3, 2.0), (10.0, 6.0)];

    Chart::default().lineplot(&Shape::Lines(&points)).nice();

    // You can plot several functions on the same chart.
    // However the resolution of text displays is low, and the result might not be great.
    println!("\ny = cos(x), y = sin(x) / 2");
    Chart::new(180, 60, -5.0, 5.0)
        .linecolorplot(
            &Shape::Continuous(Box::new(|x| x.cos())),
            RGB8 {
                r: 255_u8,
                g: 0,
                b: 0,
            },
        )
        .linecolorplot(
            &Shape::Continuous(Box::new(|x| x.sin() / 2.0)),
            RGB8 { r: 0, g: 0, b: 255 },
        )
        .display();
}
