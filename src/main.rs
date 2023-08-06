use conplot::RGB8;
use conplot::{Chart, Plot, Shape};

fn main() {
    let points = vec![(-5.0, 3.0), (3.3, 2.0), (10.0, 6.0)];

    Chart::with_range(32, 32, -10.0, 10.0, 0.0, 10.0)
        .data(points.clone())
        .lineplot(Shape::Lines, Some(RGB8::new(200, 0, 0)))
        .nice();

    Chart::default()
        .data(points)
        .lineplot(Shape::Steps, Some(RGB8::new_hex_str("#FF000B")))
        .nice();
}
