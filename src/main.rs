use conplot::{Chart, Data, Plot, Shape};

fn main() {
    let points = vec![(-5.0, 3.0), (3.3, 2.0), (10.0, 6.0)];

    Chart::with_range(70, 40, -5.0, 10.0, 0.0, 10.0)
        .lineplot(Shape::Lines, None)
        .data(points)
        .nice();
}
