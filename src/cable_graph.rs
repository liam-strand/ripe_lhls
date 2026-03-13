use geo::{Distance, Geodesic, MultiLineString, Point};
use petgraph::algo::dijkstra;
use petgraph::graph::{NodeIndex, UnGraph};

pub struct CableGraph {
    graph: UnGraph<usize, f64>,
    points: Vec<(Point, NodeIndex)>,
}

impl CableGraph {
    pub fn new(cable_geo: &MultiLineString) -> Self {
        let mut graph = UnGraph::new_undirected();
        let mut points: Vec<(Point, NodeIndex)> = Vec::new();

        for line in &cable_geo.0 {
            for c in &line.0 {
                let p = Point::from(*c);
                if !points.iter().any(|(pt, _)| pt == &p) {
                    points.push((p, graph.add_node(0)));
                }
            }
        }

        for line in &cable_geo.0 {
            for i in 0..line.0.len() - 1 {
                let p1 = Point::from(line.0[i]);
                let p2 = Point::from(line.0[i + 1]);
                let idx1 = Self::get_node(&p1, &points).1;
                let idx2 = Self::get_node(&p2, &points).1;
                let d = Geodesic.distance(p1, p2);
                graph.add_edge(idx1, idx2, d);
            }
        }

        Self { graph, points }
    }

    fn node<'a>(&'a self, p: &'a Point) -> &'a (Point, NodeIndex) {
        Self::get_node(p, &self.points)
    }

    fn get_node<'a>(p: &'a Point, points: &'a [(Point, NodeIndex)]) -> &'a (Point, NodeIndex) {
        points
            .iter()
            .min_by(|(pt1, _), (pt2, _)| {
                let d1 = Geodesic.distance(*pt1, *p);
                let d2 = Geodesic.distance(*pt2, *p);
                d1.partial_cmp(&d2).unwrap()
            })
            .unwrap()
    }

    pub fn traverse(&self, start: &Point, end: &Point) -> Option<f64> {
        let start_node = self.node(start);
        let end_node = self.node(end);
        let start_dist = Geodesic.distance(*start, start_node.0);
        let end_dist = Geodesic.distance(*end, end_node.0);

        let binding = dijkstra(&self.graph, start_node.1, Some(end_node.1), |e| *e.weight());
        let dist = binding.get(&end_node.1)?;
        Some(start_dist + dist + end_dist)
    }
}
