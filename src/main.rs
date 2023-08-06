use conplot::RGB8;
use conplot::{Chart, Data, Plot, Shape};

fn main() {
    let points = vec![(-5.0, 3.0), (3.3, 2.0), (10.0, 6.0)];

    Chart::default()
        .data(points.clone())
        .lineplot(Shape::Lines, Some(RGB8::new(200, 0, 0)))
        .nice();

    Chart::with_range(120, 50, 0.0, 5.0, 2.0, 6.0)
        .data(points)
        .lineplot(Shape::Lines, Some(RGB8::new_hex_str("#FF000B")))
        .nice();
}
